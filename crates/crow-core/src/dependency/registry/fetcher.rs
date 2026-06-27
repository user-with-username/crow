use anyhowed::{bail, Context, Result};
use dirs::home_dir;
use git2::{FetchOptions, Repository};
use once_cell::sync::Lazy;
use semver::{Version, VersionReq};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use crate::dependency::gitty::git_ops::find_default_branch;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryEntry {
    pub git: String,
    pub commit: String,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegistryIndex {
    pub versions: HashMap<String, RegistryEntry>,
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

    pub fn resolve_str(
        &self,
        dep_name: &str,
        version_req_str: &str,
        registry_url: &str,
    ) -> Result<RegistryCoordinates> {
        let req = VersionReq::parse(version_req_str)?;
        self.resolve(dep_name, &req, registry_url)
    }

    pub fn resolve(
        &self,
        dep_name: &str,
        version_req: &VersionReq,
        registry_url: &str,
    ) -> Result<RegistryCoordinates> {
        let repo = self.ensure_registry_repo(registry_url)?;

        let index = self.read_index(&repo, dep_name)?;
        if index.versions.is_empty() {
            bail!("registry has no versions for package `{dep_name}`");
        }

        let mut available: Vec<_> = index
            .versions
            .iter()
            .filter_map(|(v, entry)| Version::parse(v).ok().map(|ver| (ver, v, entry)))
            .collect();
        available.sort_by(|a, b| b.0.cmp(&a.0));

        let (_chosen_version, version_str, entry) = available
            .iter()
            .find(|(v, _, _)| version_req.matches(&v))
            .with_context(|| {
                let versions: Vec<String> =
                    available.iter().map(|(_, v, _)| v.to_string()).collect();
                format!(
                    "no version of `{dep_name}` satisfies `{}` in registry `{registry_url}`\n\
                     available versions: {}",
                    version_req.to_string(),
                    versions.join(", ")
                )
            })?;

        Ok(RegistryCoordinates {
            git_url: entry.git.clone(),
            commit: entry.commit.clone(),
            version: version_str.to_string(),
        })
    }

    pub fn delete(package_name: &str, version: Option<&str>, registry_url: &str) -> Result<()> {
        super::delete::delete(package_name, version, registry_url)
    }

    pub fn publish(
        project_root: &std::path::PathBuf,
        package_name: &str,
        version: &str,
        registry_url: &str,
    ) -> Result<bool> {
        super::publish::publish(project_root, package_name, version, registry_url)
    }

    fn ensure_registry_repo(&self, registry_url: &str) -> Result<Repository> {
        let registry_dir = self.cache_dir(registry_url);

        let repo = if registry_dir.join("HEAD").exists() || registry_dir.join(".git").exists() {
            let repo = if registry_dir.join(".git").exists() {
                Repository::open(&registry_dir)
            } else {
                Repository::open_bare(&registry_dir)
            }
            .with_context(|| {
                format!("failed to open registry repo at {}", registry_dir.display())
            })?;
            repo
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
            repo
        };

        if Self::registry_fetch_is_stale(&registry_dir) {
            self.fetch_origin(&repo, registry_url)?;
            Self::touch_registry_fetch_marker(&registry_dir)?;
        }
        Ok(repo)
    }

    const REGISTRY_FETCH_INTERVAL: Duration = Duration::from_secs(300);

    fn registry_fetch_is_stale(registry_dir: &PathBuf) -> bool {
        let marker = registry_dir.join(".last-fetch");
        let Ok(meta) = std::fs::metadata(&marker) else {
            return true;
        };
        let Ok(modified) = meta.modified() else {
            return true;
        };
        modified
            .elapsed()
            .map(|age| age > Self::REGISTRY_FETCH_INTERVAL)
            .unwrap_or(true)
    }

    fn touch_registry_fetch_marker(registry_dir: &PathBuf) -> Result<()> {
        std::fs::write(registry_dir.join(".last-fetch"), "")?;
        Ok(())
    }

    fn fetch_origin(&self, repo: &Repository, registry_url: &str) -> Result<()> {
        let mut remote = repo
            .find_remote("origin")
            .context("failed to find remote 'origin' in registry repo")?;
        let mut opts = FetchOptions::new();
        remote
            .fetch(
                &["refs/heads/*:refs/remotes/origin/*"],
                Some(&mut opts),
                None,
            )
            .with_context(|| format!("failed to fetch registry from {registry_url}"))?;
        Ok(())
    }

    fn cache_dir(&self, registry_url: &str) -> PathBuf {
        self.cache_root
            .join(format!("registry-{:016x}", url_hash(registry_url)))
    }

    fn head_commit(repo: &Repository) -> Result<git2::Commit<'_>> {
        let branch = find_default_branch(repo)?;
        let ref_name = if repo.is_bare() {
            format!("refs/remotes/origin/{branch}")
        } else {
            format!("refs/heads/{branch}")
        };
        repo.find_reference(&ref_name)?
            .peel_to_commit()
            .context("failed to peel to commit")
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

    fn read_index(&self, repo: &Repository, dep_name: &str) -> Result<RegistryIndex> {
        let blob_path = format!("packages/{dep_name}.toml");
        let raw_bytes = self
            .read_blob(&repo, &blob_path)
            .with_context(|| format!("failed to read registry index for `{dep_name}`"))?;

        let raw_str = std::str::from_utf8(&raw_bytes)
            .with_context(|| format!("registry index for `{dep_name}` is not valid UTF-8"))?;

        let index: RegistryIndex = toml::from_str(raw_str)
            .with_context(|| format!("invalid registry index for `{dep_name}` at {blob_path}"))?;

        Ok(index)
    }
}

fn url_hash(url: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    h.finish()
}
