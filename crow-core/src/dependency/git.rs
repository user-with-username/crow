use anyhow::{Context, Result};
use dirs::home_dir;
use git2::{FetchOptions, Repository};
use once_cell::sync::Lazy;
use std::path::PathBuf;

#[derive(Debug)]
pub struct GitDependencyFetcher {
    cache_root: PathBuf,
}

static GIT_FETCHER: Lazy<GitDependencyFetcher> = Lazy::new(|| {
    let cache_root = home_dir()
        .expect("cannot find home directory")
        .join(".crow")
        .join("git_cache");
    std::fs::create_dir_all(&cache_root).expect("failed to create global git cache");
    GitDependencyFetcher { cache_root }
});

impl GitDependencyFetcher {
    pub fn global() -> &'static Self {
        &GIT_FETCHER
    }

    pub fn fetch(&self, dep_name: &str, git_url: &str) -> Result<(PathBuf, String)> {
        let dep_dir = self.dep_dir(dep_name, git_url);

        if dep_dir.join(".git").exists() {
            let rev = self.update_existing_repo(&dep_dir, git_url, dep_name)?;
            Ok((dep_dir, rev))
        } else {
            let rev = self.clone_new_repo(&dep_dir, git_url, dep_name)?;
            Ok((dep_dir, rev))
        }
    }

    pub fn fetch_commit(
        &self,
        dep_name: &str,
        git_url: &str,
        commit: &str,
    ) -> Result<(PathBuf, String)> {
        let short = &commit[..commit.len().min(16)];
        let dep_dir = self.dep_dir_with_suffix(dep_name, git_url, short);

        if dep_dir.join(".git").exists() {
            let repo = Repository::open(&dep_dir)
                .with_context(|| format!("failed to open {}", dep_dir.display()))?;
            let head = repo.head().ok().and_then(|h| h.peel_to_commit().ok());
            if let Some(c) = head {
                if c.id().to_string().starts_with(commit) {
                    return Ok((dep_dir, c.id().to_string()));
                }
            }
            let rev = self.checkout_commit(&dep_dir, git_url, dep_name, commit)?;
            return Ok((dep_dir, rev));
        }

        let rev = self.clone_at_commit(&dep_dir, git_url, dep_name, commit)?;
        Ok((dep_dir, rev))
    }

    fn dep_dir(&self, dep_name: &str, git_url: &str) -> PathBuf {
        let hash = url_hash(git_url);
        self.cache_root.join(format!("{}-{:016x}", dep_name, hash))
    }

    fn dep_dir_with_suffix(&self, dep_name: &str, git_url: &str, suffix: &str) -> PathBuf {
        let hash = url_hash(git_url);
        self.cache_root
            .join(format!("{}-{:016x}-{}", dep_name, hash, suffix))
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

    fn clone_at_commit(
        &self,
        dep_dir: &PathBuf,
        git_url: &str,
        dep_name: &str,
        commit: &str,
    ) -> Result<String> {
        use crow_utils::status;
        use git2::build::CheckoutBuilder;

        status!("Downloading", "{}", dep_name);

        let repo = Repository::init(dep_dir)
            .with_context(|| format!("failed to init repo at {}", dep_dir.display()))?;

        let mut remote = repo
            .remote("origin", git_url)
            .with_context(|| format!("failed to add remote for {git_url}"))?;

        let mut fetch_opts = FetchOptions::new();
        remote
            .fetch(
                &[
                    "refs/heads/*:refs/remotes/origin/*",
                    "refs/tags/*:refs/tags/*",
                ],
                Some(&mut fetch_opts),
                None,
            )
            .with_context(|| format!("failed to fetch from {git_url}"))?;

        let oid = git2::Oid::from_str(commit)
            .with_context(|| format!("invalid commit sha `{commit}`"))?;
        let obj = repo
            .find_object(oid, None)
            .with_context(|| format!("commit `{commit}` not found in {git_url}"))?;

        repo.checkout_tree(&obj, Some(CheckoutBuilder::new().force()))
            .with_context(|| format!("failed to checkout {commit}"))?;

        repo.set_head_detached(oid)
            .with_context(|| format!("failed to detach HEAD at {commit}"))?;

        Ok(oid.to_string())
    }

    fn checkout_commit(
        &self,
        dep_dir: &PathBuf,
        git_url: &str,
        dep_name: &str,
        commit: &str,
    ) -> Result<String> {
        use crow_utils::status;
        use git2::build::CheckoutBuilder;

        status!("Checking out", "{} @ {}", dep_name, &commit[..8.min(commit.len())]);

        let repo = Repository::open(dep_dir)
            .with_context(|| format!("failed to open {}", dep_dir.display()))?;

        let oid = git2::Oid::from_str(commit)
            .with_context(|| format!("invalid commit sha `{commit}`"))?;

        let obj = match repo.find_object(oid, None) {
            Ok(o) => o,
            Err(_) => {
                let mut remote = repo
                    .find_remote("origin")
                    .context("failed to find remote 'origin'")?;
                let mut fetch_opts = FetchOptions::new();
                remote
                    .fetch(
                        &[
                            "refs/heads/*:refs/remotes/origin/*",
                            "refs/tags/*:refs/tags/*",
                        ],
                        Some(&mut fetch_opts),
                        None,
                    )
                    .with_context(|| format!("failed to fetch from {git_url}"))?;
                repo.find_object(oid, None)
                    .with_context(|| format!("commit `{commit}` still not found after fetch"))?
            }
        };

        repo.checkout_tree(&obj, Some(CheckoutBuilder::new().force()))
            .with_context(|| format!("failed to checkout {commit}"))?;
        repo.set_head_detached(oid)?;

        Ok(oid.to_string())
    }
}

fn url_hash(url: &str) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    url.hash(&mut h);
    h.finish()
}