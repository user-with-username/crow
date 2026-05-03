use anyhow::{Context, Result};
use git2::{FetchOptions, Repository};
use std::path::PathBuf;

#[derive(Debug)]
pub struct GitDependencyFetcher {
    cache_root: PathBuf,
}

impl GitDependencyFetcher {
    pub fn new(cache_root: PathBuf) -> Self {
        Self { cache_root }
    }

    pub fn fetch(&self, dep_name: &str, git_url: &str) -> Result<(PathBuf, String)> {
        let hash = {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            git_url.hash(&mut hasher);
            hasher.finish()
        };
        let dep_dir = self.cache_root.join(format!("{}-{:016x}", dep_name, hash));

        if dep_dir.join(".git").exists() {
            let rev = self.update_existing_repo(&dep_dir, git_url, dep_name)?;
            Ok((dep_dir, rev))
        } else {
            let rev = self.clone_new_repo(&dep_dir, git_url, dep_name)?;
            Ok((dep_dir, rev))
        }
    }

    fn update_existing_repo(
        &self,
        dep_dir: &PathBuf,
        git_url: &str,
        dep_name: &str,
    ) -> Result<String> {
        use crow_utils::status;
        use git2::build::CheckoutBuilder;

        status!("Downloading", "{}", dep_name);

        let repo = Repository::open(dep_dir)
            .with_context(|| format!("failed to open git repository at {}", dep_dir.display()))?;

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
            .with_context(|| format!("failed to fetch from {git_url}"))?;

        let commit = repo
            .revparse_single("origin/HEAD")
            .or_else(|_| repo.revparse_single("origin/master"))
            .or_else(|_| repo.revparse_single("origin/main"))
            .with_context(|| format!("failed to find default branch for {git_url}"))?;

        let commit_id = commit.id();
        let obj = repo.find_object(commit_id, None)?;
        repo.checkout_tree(&obj, Some(CheckoutBuilder::new().force()))
            .with_context(|| format!("failed to checkout commit {commit_id}"))?;

        repo.set_head("refs/remotes/origin/HEAD")
            .or_else(|_| repo.set_head("refs/remotes/origin/master"))
            .or_else(|_| repo.set_head("refs/remotes/origin/main"))?;

        Ok(commit_id.to_string())
    }

    fn clone_new_repo(&self, dep_dir: &PathBuf, git_url: &str, dep_name: &str) -> Result<String> {
        use crow_utils::status;
        use git2::build::CheckoutBuilder;

        status!("Downloading", "{}", dep_name);

        let repo = Repository::init(dep_dir)
            .with_context(|| format!("failed to initialize repository at {}", dep_dir.display()))?;

        let mut remote = repo
            .remote("origin", git_url)
            .with_context(|| format!("failed to add remote origin for {git_url}"))?;

        let mut fetch_opts = FetchOptions::new();
        remote
            .fetch(
                &["refs/heads/*:refs/remotes/origin/*"],
                Some(&mut fetch_opts),
                None,
            )
            .with_context(|| format!("failed to fetch from {git_url}"))?;

        let commit = repo
            .revparse_single("origin/HEAD")
            .or_else(|_| repo.revparse_single("origin/master"))
            .or_else(|_| repo.revparse_single("origin/main"))
            .with_context(|| format!("failed to find default branch for {git_url}"))?;

        let commit_id = commit.id();
        let obj = repo.find_object(commit_id, None)?;
        repo.checkout_tree(&obj, Some(CheckoutBuilder::new().force()))
            .with_context(|| format!("failed to checkout commit {commit_id}"))?;

        repo.set_head("refs/remotes/origin/HEAD")
            .or_else(|_| repo.set_head("refs/remotes/origin/master"))
            .or_else(|_| repo.set_head("refs/remotes/origin/main"))?;

        Ok(commit_id.to_string())
    }
}
