use anyhow::{bail, Context, Result};
use dirs::home_dir;
use git2::{FetchOptions, Repository};
use once_cell::sync::Lazy;
use semver::Version;
use serde::Deserialize;
use std::path::PathBuf;

use crate::dependency::version_req::VersionReq as CrowVersionReq;

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
            .with_context(|| format!("failed to open registry repo at {}", registry_dir.display()))?;

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
            let repo = Repository::init_bare(&registry_dir)
                .with_context(|| {
                    format!("failed to init bare registry repo at {}", registry_dir.display())
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
        let commit = repo
            .revparse_single("origin/HEAD")
            .or_else(|_| repo.revparse_single("origin/master"))
            .or_else(|_| repo.revparse_single("origin/main"))
            .context("failed to find default branch in registry")?
            .peel_to_commit()
            .context("failed to peel to commit")?;
        Ok(commit)
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
                let versions: Vec<String> =
                    available.iter().map(|(v, _)| v.to_string()).collect();
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
            format!(
                "invalid registry entry for `{dep_name}` v{chosen_version} at {blob_path}"
            )
        })?;

        Ok(RegistryCoordinates {
            git_url: entry.git,
            commit: entry.commit,
            version: version_str.clone(),
        })
    }
}

fn url_hash(url: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    h.finish()
}