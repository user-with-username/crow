use anyhow::{bail, Context, Result};
use dirs::home_dir;
use git2::{FetchOptions, Repository};
use once_cell::sync::Lazy;
use reqwest::blocking::Client;
use semver::Version;
use serde::Deserialize;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::TempDir;
use url::Url;

use crate::dependency::version_req::VersionReq as CrowVersionReq;
use crow_utils::environment::Environment;

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryEntry {
    pub git: String,
    pub commit: String,
}

#[derive(Debug, Clone)]
pub struct RegistryCoordinates {
    pub git_url: String,
    pub commit: String,
    pub version: String,
}

pub struct RegistryFetcher {
    cache_root: PathBuf,
}

static REGISTRY_FETCHER: Lazy<RegistryFetcher> = Lazy::new(|| {
    let cache_root = home_dir()
        .expect("cannot find home directory")
        .join(".crow")
        .join("registry_cache");
    std::fs::create_dir_all(&cache_root).expect("failed to create registry cache directory");
    RegistryFetcher { cache_root }
});

impl RegistryFetcher {
    pub fn global() -> &'static Self {
        &REGISTRY_FETCHER
    }

    fn ensure_registry_repo(&self, registry_url: &str) -> Result<Repository> {
        let hash = url_hash(registry_url);
        let registry_dir = self.cache_root.join(format!("registry-{:016x}", hash));

        if registry_dir.join("HEAD").exists() || registry_dir.join(".git").exists() {
            let repo = if registry_dir.join(".git").exists() {
                Repository::open(&registry_dir)
            } else {
                Repository::open_bare(&registry_dir)
            }
            .with_context(|| {
                format!("failed to open registry repo at {}", registry_dir.display())
            })?;

            {
                let mut remote = repo
                    .find_remote("origin")
                    .context("failed to find remote 'origin' in registry repo")?;

                let mut fetch_opts = FetchOptions::new();
                remote
                    .fetch(
                        &["refs/heads/*:refs/remotes/origin/*"],
                        Some(&mut fetch_opts),
                        None,
                    )
                    .with_context(|| format!("failed to fetch registry from {registry_url}"))?;
            }

            Ok(repo)
        } else {
            std::fs::create_dir_all(&registry_dir)?;
            let repo = Repository::init_bare(&registry_dir).with_context(|| {
                format!(
                    "failed to init bare registry repo at {}",
                    registry_dir.display()
                )
            })?;

            repo.remote("origin", registry_url)
                .with_context(|| format!("failed to add remote {registry_url}"))?;

            {
                let mut remote = repo
                    .find_remote("origin")
                    .context("failed to find remote 'origin'")?;

                let mut fetch_opts = FetchOptions::new();
                remote
                    .fetch(
                        &["refs/heads/*:refs/remotes/origin/*"],
                        Some(&mut fetch_opts),
                        None,
                    )
                    .with_context(|| format!("failed to fetch registry from {registry_url}"))?;
            }

            Ok(repo)
        }
    }

    fn head_commit(repo: &Repository) -> Result<git2::Commit<'_>> {
        let default_branch = Self::find_default_branch(repo)?;

        let commit = repo
            .find_reference(&default_branch)?
            .peel_to_commit()
            .context("failed to peel to commit")?;
        Ok(commit)
    }

    fn find_default_branch(repo: &Repository) -> Result<String> {
        if let Ok(head) = repo.find_reference("HEAD") {
            if let Some(target) = head.symbolic_target() {
                if target.starts_with("refs/heads/") {
                    return Ok(target.to_string());
                }
            }
        }

        if let Ok(origin_head) = repo.find_reference("refs/remotes/origin/HEAD") {
            if let Some(target) = origin_head.symbolic_target() {
                if target.starts_with("refs/remotes/origin/") {
                    let branch_name = target.replace("refs/remotes/origin/", "refs/heads/");
                    if repo.find_reference(&branch_name).is_ok() {
                        return Ok(branch_name);
                    }
                    if repo.find_reference(&target).is_ok() {
                        let local_branch = target.replace("refs/remotes/origin/", "refs/heads/");
                        return Ok(local_branch);
                    }
                }
            }
        }

        for branch_name in &["refs/heads/main", "refs/heads/master", "refs/heads/trunk"] {
            if repo.find_reference(branch_name).is_ok() {
                return Ok(branch_name.to_string());
            }
        }

        for branch_name in &["refs/remotes/origin/main", "refs/remotes/origin/master"] {
            if repo.find_reference(branch_name).is_ok() {
                let local_branch = branch_name.replace("refs/remotes/origin/", "refs/heads/");
                return Ok(local_branch);
            }
        }

        bail!("failed to find default branch in registry");
    }

    fn read_blob(&self, repo: &Repository, path: &str) -> Result<Vec<u8>> {
        let commit = Self::head_commit(repo)?;
        let tree = commit.tree().context("failed to get commit tree")?;

        let entry = tree
            .get_path(std::path::Path::new(path))
            .with_context(|| format!("path `{path}` not found in registry tree"))?;

        let blob = repo
            .find_blob(entry.id())
            .with_context(|| format!("failed to read blob for `{path}`"))?;

        Ok(blob.content().to_vec())
    }

    fn list_versions(&self, repo: &Repository, dep_name: &str) -> Result<Vec<(Version, String)>> {
        let commit = Self::head_commit(repo)?;
        let tree = commit.tree().context("failed to get commit tree")?;

        let package_path = format!("packages/{dep_name}");
        let subtree_entry = tree
            .get_path(std::path::Path::new(&package_path))
            .with_context(|| {
                format!("package `{dep_name}` not found in registry (looked under {package_path})")
            })?;

        let subtree = repo
            .find_tree(subtree_entry.id())
            .with_context(|| format!("failed to read tree for `{package_path}`"))?;

        let mut versions = Vec::new();
        for entry in subtree.iter() {
            let name = match entry.name() {
                Some(n) => n,
                None => continue,
            };
            if !name.ends_with(".toml") {
                continue;
            }
            let stem = &name[..name.len() - 5];
            if let Ok(v) = Version::parse(stem) {
                versions.push((v, stem.to_string()));
            }
        }

        Ok(versions)
    }

    pub fn resolve_str(
        &self,
        dep_name: &str,
        version_req_str: &str,
        registry_url: &str,
    ) -> Result<RegistryCoordinates> {
        let req = CrowVersionReq::parse(version_req_str)?;
        self.resolve(dep_name, &req, registry_url)
    }

    pub fn resolve(
        &self,
        dep_name: &str,
        version_req: &CrowVersionReq,
        registry_url: &str,
    ) -> Result<RegistryCoordinates> {
        let repo = self.ensure_registry_repo(registry_url)?;

        let mut available = self.list_versions(&repo, dep_name)?;

        if available.is_empty() {
            bail!("registry has no versions for package `{dep_name}`");
        }

        available.sort_by(|a, b| b.0.cmp(&a.0));

        let (chosen_version, version_str) = available
            .iter()
            .find(|(v, _)| version_req.matches(&v.to_string()))
            .with_context(|| {
                let versions: Vec<String> = available.iter().map(|(v, _)| v.to_string()).collect();
                format!(
                    "no version of `{dep_name}` satisfies `{}` in registry `{registry_url}`\n\
                     available versions: {}",
                    version_req.as_str(),
                    versions.join(", ")
                )
            })?;

        let blob_path = format!("packages/{dep_name}/{chosen_version}.toml");
        let raw_bytes = self.read_blob(&repo, &blob_path).with_context(|| {
            format!("failed to read registry entry for `{dep_name}` v{chosen_version}")
        })?;

        let raw_str = std::str::from_utf8(&raw_bytes).with_context(|| {
            format!("registry entry for `{dep_name}` v{chosen_version} is not valid UTF-8")
        })?;

        let entry: RegistryEntry = toml::from_str(raw_str).with_context(|| {
            format!("invalid registry entry for `{dep_name}` v{chosen_version} at {blob_path}")
        })?;

        Ok(RegistryCoordinates {
            git_url: entry.git,
            commit: entry.commit,
            version: version_str.clone(),
        })
    }

    pub fn publish(
        project_root: &PathBuf,
        package_name: &str,
        version: &str,
        registry_url: &str,
    ) -> Result<bool> {
        let token = Environment::github_token()?;

        let repo = git2::Repository::open(project_root)
            .with_context(|| format!("Not a git repository: {}", project_root.display()))?;

        let remote = repo
            .find_remote("origin")
            .context("No remote 'origin' found in repository")?;
        let git_url = remote
            .url()
            .context("Remote 'origin' has no URL")?
            .to_string();

        let head = repo
            .head()
            .context("No HEAD commit found. Make sure you have at least one commit")?;
        let commit_oid = head
            .target()
            .context("HEAD is not a commit (maybe detached?)")?;
        let commit_sha = commit_oid.to_string();

        let temp_dir = TempDir::new()?;
        let registry_clone_path = temp_dir.path().join("registry");

        let mut callbacks = git2::RemoteCallbacks::new();
        let token_clone = token.clone();
        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            git2::Cred::userpass_plaintext(username_from_url.unwrap_or("x-access-token"), &token_clone)
        });

        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_opts);
        builder
            .clone(registry_url, &registry_clone_path)
            .with_context(|| format!("Failed to clone registry from {}", registry_url))?;

        let registry_repo = git2::Repository::open(&registry_clone_path)?;

        {
            let mut remote = registry_repo.find_remote("origin")?;
            let mut fetch_opts = git2::FetchOptions::new();
            let mut cb = git2::RemoteCallbacks::new();
            let token_cb_clone = token.clone();
            cb.credentials(move |_url, username_from_url, _allowed_types| {
                git2::Cred::userpass_plaintext(
                    username_from_url.unwrap_or("x-access-token"),
                    &token_cb_clone,
                )
            });
            fetch_opts.remote_callbacks(cb);

            remote
                .fetch(
                    &["refs/heads/*:refs/remotes/origin/*"],
                    Some(&mut fetch_opts),
                    None,
                )
                .with_context(|| {
                    format!("Failed to fetch branches from registry {}", registry_url)
                })?;
        }

        let default_branch_ref = Self::find_default_branch(&registry_repo)?;
        let branch_name = default_branch_ref.trim_start_matches("refs/heads/");

        let commit = if let Ok(head_ref) = registry_repo.find_reference(&default_branch_ref) {
            registry_repo.reference_to_annotated_commit(&head_ref)?
        } else {
            let remote_ref = default_branch_ref.replace("refs/heads/", "refs/remotes/origin/");
            let head_ref = registry_repo.find_reference(&remote_ref)?;
            registry_repo.reference_to_annotated_commit(&head_ref)?
        };

        let commit_obj = registry_repo.find_commit(commit.id())?;
        let tree = commit_obj.tree()?;
        registry_repo.checkout_tree(&tree.into_object(), None)?;
        registry_repo.set_head(&default_branch_ref)?;

        let publish_branch = format!("publish/{}/{}", package_name, version);
        let head_commit = registry_repo.head()?.peel_to_commit()?;
        let branch = registry_repo.branch(&publish_branch, &head_commit, false)?;
        let branch_ref = branch.get();
        let branch_tree = branch_ref.peel_to_tree()?;
        registry_repo.checkout_tree(&branch_tree.into_object(), None)?;
        registry_repo.set_head(branch_ref.name().unwrap())?;

        let package_dir = registry_clone_path.join("packages").join(package_name);
        fs::create_dir_all(&package_dir)?;
        let version_file = package_dir.join(format!("{}.toml", version));

        let content = format!("git = \"{}\"\ncommit = \"{}\"\n", git_url, commit_sha);

        if version_file.exists() {
            let existing = fs::read_to_string(&version_file)?;
            if existing.trim() == content.trim() {
                return Ok(false);
            } else {
                anyhow::bail!(
                    "Version {} already published with different commit.\n\
                     Existing: {}\n\
                     New: {}",
                    version,
                    existing.trim(),
                    content.trim()
                );
            }
        }

        fs::write(&version_file, content)?;

        let mut index = registry_repo.index()?;
        index.add_all(["packages"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        let tree_oid = index.write_tree()?;
        let tree = registry_repo.find_tree(tree_oid)?;
        let parent_commit = registry_repo.head()?.peel_to_commit()?;
        let signature = git2::Signature::now("crow-publish", "crow@localhost")?;
        let commit_oid = registry_repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &format!("publish: {} v{}", package_name, version),
            &tree,
            &[&parent_commit],
        )?;
        let _commit_hash = commit_oid.to_string();

        let mut push_callbacks = git2::RemoteCallbacks::new();
        let token_push_clone = token.clone();
        push_callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            git2::Cred::userpass_plaintext(username_from_url.unwrap_or("x-access-token"), &token_push_clone)
        });

        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(push_callbacks);
        let mut remote = registry_repo.find_remote("origin")?;
        remote.push(
            &[&format!(
                "refs/heads/{}:refs/heads/{}",
                publish_branch, publish_branch
            )],
            Some(&mut push_opts),
        )?;

        let parsed_url = Url::parse(registry_url)
            .with_context(|| format!("Invalid registry URL: {}", registry_url))?;

        let path_segments: Vec<&str> = parsed_url.path().trim_matches('/').split('/').collect();
        if path_segments.len() < 2 {
            anyhow::bail!(
                "Invalid registry URL format: expected github.com/owner/repo, got {}",
                registry_url
            );
        }
        let owner = path_segments[0];
        let repo_name = path_segments[1].trim_end_matches(".git");

        let api_url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo_name);
        let client = Client::new();
        let pr_body = format!(
            "Publish `{}` version `{}`\n\n- Git URL: {}\n- Commit: `{}`\n\nAuto-merge will be triggered if CI passes.",
            package_name, version, git_url, commit_sha
        );

        let payload = json!({
            "title": format!("publish: {} v{}", package_name, version),
            "head": publish_branch,
            "base": branch_name,
            "body": pr_body,
        });

        let response = client
            .post(&api_url)
            .header("Authorization", format!("token {}", token))
            .header("User-Agent", "crow-publish")
            .json(&payload)
            .send()
            .with_context(|| format!("Failed to send PR request to {}", api_url))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text()?;
            if status == 422 && error_text.contains("A pull request already exists") {
                return Ok(true);
            }
            anyhow::bail!("GitHub API error ({}): {}", status, error_text);
        }

        Ok(true)
    }

    pub fn delete(package_name: &str, version: Option<&str>, registry_url: &str) -> Result<()> {
        let token = Environment::github_token()?;

        let temp_dir = TempDir::new()?;
        let registry_clone_path = temp_dir.path().join("registry");

        let mut callbacks = git2::RemoteCallbacks::new();
        let token_clone = token.clone();
        callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            git2::Cred::userpass_plaintext(username_from_url.unwrap_or("x-access-token"), &token_clone)
        });

        let mut fetch_opts = git2::FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        let mut builder = git2::build::RepoBuilder::new();
        builder.fetch_options(fetch_opts);
        builder
            .clone(registry_url, &registry_clone_path)
            .with_context(|| format!("Failed to clone registry from {}", registry_url))?;

        let registry_repo = git2::Repository::open(&registry_clone_path)?;

        {
            let mut remote = registry_repo.find_remote("origin")?;
            let mut fetch_opts = git2::FetchOptions::new();
            let mut cb = git2::RemoteCallbacks::new();
            let token_cb_clone = token.clone();
            cb.credentials(move |_url, username_from_url, _allowed_types| {
                git2::Cred::userpass_plaintext(
                    username_from_url.unwrap_or("x-access-token"),
                    &token_cb_clone,
                )
            });
            fetch_opts.remote_callbacks(cb);

            remote
                .fetch(
                    &["refs/heads/*:refs/remotes/origin/*"],
                    Some(&mut fetch_opts),
                    None,
                )
                .with_context(|| {
                    format!("Failed to fetch branches from registry {}", registry_url)
                })?;
        }

        let default_branch_ref = Self::find_default_branch(&registry_repo)?;
        let branch_name = default_branch_ref.trim_start_matches("refs/heads/");

        let commit = if let Ok(head_ref) = registry_repo.find_reference(&default_branch_ref) {
            registry_repo.reference_to_annotated_commit(&head_ref)?
        } else {
            let remote_ref = default_branch_ref.replace("refs/heads/", "refs/remotes/origin/");
            let head_ref = registry_repo.find_reference(&remote_ref)?;
            registry_repo.reference_to_annotated_commit(&head_ref)?
        };

        let commit_obj = registry_repo.find_commit(commit.id())?;
        let tree = commit_obj.tree()?;
        registry_repo.checkout_tree(&tree.into_object(), None)?;
        registry_repo.set_head(&default_branch_ref)?;

        let package_dir = registry_clone_path.join("packages").join(package_name);

        if !package_dir.exists() {
            anyhow::bail!("Package '{}' not found in registry", package_name);
        }

        let (files_to_delete, _description) = if let Some(ver) = version {
            let version_file = package_dir.join(format!("{}.toml", ver));
            if !version_file.exists() {
                anyhow::bail!(
                    "Version '{}' of package '{}' not found in registry",
                    ver,
                    package_name
                );
            }
            (vec![version_file], format!("{} v{}", package_name, ver))
        } else {
            let mut files = Vec::new();
            if let Ok(entries) = fs::read_dir(&package_dir) {
                for entry in entries.flatten() {
                    if entry.path().extension().map_or(false, |ext| ext == "toml") {
                        files.push(entry.path());
                    }
                }
            }

            if files.is_empty() {
                anyhow::bail!("No versions found for package '{}'", package_name);
            }

            (files, format!("{} (all versions)", package_name))
        };

        let delete_branch = format!("delete/{}/{}", package_name, version.unwrap_or("all"));

        let head_commit = registry_repo.head()?.peel_to_commit()?;
        registry_repo.branch(&delete_branch, &head_commit, false)?;

        let branch_ref = registry_repo.find_reference(&format!("refs/heads/{}", delete_branch))?;
        let branch_tree = branch_ref.peel_to_tree()?;
        registry_repo.checkout_tree(&branch_tree.into_object(), None)?;
        registry_repo.set_head(branch_ref.name().unwrap())?;

        for file in &files_to_delete {
            fs::remove_file(file)
                .with_context(|| format!("Failed to delete {}", file.display()))?;
        }

        if version.is_none() {
            if package_dir.exists() {
                let remaining: Vec<_> =
                    fs::read_dir(&package_dir)?.filter_map(|e| e.ok()).collect();
                if remaining.is_empty() {
                    fs::remove_dir(&package_dir)?;
                }
            }
        }

        let mut index = registry_repo.index()?;
        index.add_all(["packages"].iter(), git2::IndexAddOption::DEFAULT, None)?;
        index.write()?;
        let tree_oid = index.write_tree()?;
        let tree = registry_repo.find_tree(tree_oid)?;
        let parent_commit = registry_repo.head()?.peel_to_commit()?;
        let signature = git2::Signature::now("crow-delete", "crow@localhost")?;

        let commit_message = if let Some(ver) = version {
            format!("delete: {} v{}", package_name, ver)
        } else {
            format!("delete: {} (all versions)", package_name)
        };

        registry_repo.commit(
            Some("HEAD"),
            &signature,
            &signature,
            &commit_message,
            &tree,
            &[&parent_commit],
        )?;

        let mut push_callbacks = git2::RemoteCallbacks::new();
        let token_push_clone = token.clone();
        push_callbacks.credentials(move |_url, username_from_url, _allowed_types| {
            git2::Cred::userpass_plaintext(username_from_url.unwrap_or("x-access-token"), &token_push_clone)
        });

        let mut push_opts = git2::PushOptions::new();
        push_opts.remote_callbacks(push_callbacks);
        let mut remote = registry_repo.find_remote("origin")?;
        remote.push(
            &[&format!(
                "refs/heads/{}:refs/heads/{}",
                delete_branch, delete_branch
            )],
            Some(&mut push_opts),
        )?;

        let parsed_url = Url::parse(registry_url)
            .with_context(|| format!("Invalid registry URL: {}", registry_url))?;

        let path_segments: Vec<&str> = parsed_url.path().trim_matches('/').split('/').collect();
        if path_segments.len() < 2 {
            anyhow::bail!(
                "Invalid registry URL format: expected github.com/owner/repo, got {}",
                registry_url
            );
        }
        let owner = path_segments[0];
        let repo_name = path_segments[1].trim_end_matches(".git");

        let api_url = format!("https://api.github.com/repos/{}/{}/pulls", owner, repo_name);
        let client = Client::new();

        let pr_body = if let Some(ver) = version {
            format!(
                "Delete `{}` version `{}`\n\nThis PR removes version {} of package {}.",
                package_name, ver, ver, package_name
            )
        } else {
            format!(
                "Delete `{}` (all versions)\n\nThis PR removes all versions of package {}.",
                package_name, package_name
            )
        };

        let pr_title = if let Some(ver) = version {
            format!("delete: {} v{}", package_name, ver)
        } else {
            format!("delete: {} (all versions)", package_name)
        };

        let payload = json!({
            "title": pr_title,
            "head": delete_branch,
            "base": branch_name,
            "body": pr_body,
        });

        let response = client
            .post(&api_url)
            .header("Authorization", format!("token {}", token))
            .header("User-Agent", "crow-delete")
            .json(&payload)
            .send()
            .with_context(|| format!("Failed to send PR request to {}", api_url))?;

        let status = response.status();
        if !status.is_success() {
            let error_text = response.text()?;
            if status == 422 && error_text.contains("A pull request already exists") {
                return Ok(());
            }
            anyhow::bail!("GitHub API error ({}): {}", status, error_text);
        }

        Ok(())
    }
}

fn url_hash(url: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    h.finish()
}