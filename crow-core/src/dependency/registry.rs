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

    pub fn ensure_registry(&self, registry_url: &str) -> Result<PathBuf> {
        let hash = url_hash(registry_url);
        let registry_dir = self.cache_root.join(format!("registry-{:016x}", hash));

        if registry_dir.join(".git").exists() {
            self.update_registry(&registry_dir, registry_url)
                .with_context(|| format!("failed to update registry at {registry_url}"))?;
        } else {
            self.clone_registry(&registry_dir, registry_url)
                .with_context(|| format!("failed to clone registry from {registry_url}"))?;
        }

        Ok(registry_dir)
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
        let registry_dir = self.ensure_registry(registry_url)?;
        let package_dir = registry_dir.join("packages").join(dep_name);

        if !package_dir.exists() {
            bail!(
                "package `{dep_name}` not found in registry `{registry_url}`\n\
                 (looked in {})",
                package_dir.display()
            );
        }

        let mut available: Vec<(Version, PathBuf, String)> = std::fs::read_dir(&package_dir)
            .with_context(|| format!("cannot read registry package dir {}", package_dir.display()))?
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let path = e.path();
                if path.extension().and_then(|s| s.to_str()) != Some("toml") {
                    return None;
                }
                let stem = path.file_stem()?.to_str()?.to_string();
                let version = Version::parse(&stem).ok()?;
                Some((version, path, stem))
            })
            .collect();

        if available.is_empty() {
            bail!("registry has no versions for package `{dep_name}`");
        }

        available.sort_by(|a, b| b.0.cmp(&a.0));

        let (chosen_version, chosen_path, version_str) = available
            .iter()
            .find(|(v, _, _)| version_req.matches(&v.to_string()))
            .with_context(|| {
                let versions: Vec<String> = available.iter().map(|(v, _, _)| v.to_string()).collect();
                format!(
                    "no version of `{dep_name}` satisfies `{}` in registry `{registry_url}`\n\
                     available versions: {}",
                    version_req.as_str(),
                    versions.join(", ")
                )
            })?;

        let entry: RegistryEntry = {
            let raw = std::fs::read_to_string(chosen_path).with_context(|| {
                format!(
                    "failed to read registry entry at {}",
                    chosen_path.display()
                )
            })?;
            toml::from_str(&raw).with_context(|| {
                format!(
                    "invalid registry entry for `{dep_name}` v{chosen_version} at {}",
                    chosen_path.display()
                )
            })?
        };

        Ok(RegistryCoordinates {
            git_url: entry.git,
            commit: entry.commit,
            version: version_str.clone(),
        })
    }

    fn clone_registry(&self, dir: &PathBuf, url: &str) -> Result<()> {
        use git2::build::CheckoutBuilder;

        if let Some(parent) = dir.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let repo = Repository::init(dir)
            .with_context(|| format!("failed to init registry repo at {}", dir.display()))?;

        let mut remote = repo
            .remote("origin", url)
            .with_context(|| format!("failed to add remote {url}"))?;

        let mut fetch_opts = FetchOptions::new();
        remote
            .fetch(
                &["refs/heads/*:refs/remotes/origin/*"],
                Some(&mut fetch_opts),
                None,
            )
            .with_context(|| format!("failed to fetch registry from {url}"))?;

        let commit = repo
            .revparse_single("origin/HEAD")
            .or_else(|_| repo.revparse_single("origin/master"))
            .or_else(|_| repo.revparse_single("origin/main"))
            .context("failed to find default branch in registry")?;

        let obj = repo.find_object(commit.id(), None)?;
        repo.checkout_tree(&obj, Some(CheckoutBuilder::new().force()))
            .context("failed to checkout registry")?;

        repo.set_head("refs/remotes/origin/HEAD")
            .or_else(|_| repo.set_head("refs/remotes/origin/master"))
            .or_else(|_| repo.set_head("refs/remotes/origin/main"))?;

        Ok(())
    }

    fn update_registry(&self, dir: &PathBuf, url: &str) -> Result<()> {
        use git2::build::CheckoutBuilder;

        let repo = Repository::open(dir)
            .with_context(|| format!("failed to open registry repo at {}", dir.display()))?;

        let mut remote = repo
            .find_remote("origin")
            .context("failed to find remote 'origin' in registry")?;

        let mut fetch_opts = FetchOptions::new();
        remote
            .fetch(
                &["refs/heads/*:refs/remotes/origin/*"],
                Some(&mut fetch_opts),
                None,
            )
            .with_context(|| format!("failed to fetch registry from {url}"))?;

        let commit = repo
            .revparse_single("origin/HEAD")
            .or_else(|_| repo.revparse_single("origin/master"))
            .or_else(|_| repo.revparse_single("origin/main"))
            .context("failed to find default branch in registry")?;

        let obj = repo.find_object(commit.id(), None)?;
        repo.checkout_tree(&obj, Some(CheckoutBuilder::new().force()))
            .context("failed to checkout registry")?;

        repo.set_head("refs/remotes/origin/HEAD")
            .or_else(|_| repo.set_head("refs/remotes/origin/master"))
            .or_else(|_| repo.set_head("refs/remotes/origin/main"))?;

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