use anyhow::{Context, Result};
use git2::{FetchOptions, RemoteCallbacks, Repository};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

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

        let resolving_printed = Arc::new(AtomicBool::new(false));

        if dep_dir.join(".git").exists() {
            let rev = self.update_existing_repo(&dep_dir, git_url, resolving_printed)?;
            Ok((dep_dir, rev))
        } else {
            let rev = self.clone_new_repo(&dep_dir, git_url, resolving_printed)?;
            Ok((dep_dir, rev))
        }
    }

    fn update_existing_repo(
        &self,
        dep_dir: &PathBuf,
        git_url: &str,
        resolving_printed: Arc<AtomicBool>,
    ) -> Result<String> {
        use git2::build::CheckoutBuilder;

        let repo = Repository::open(dep_dir).with_context(|| {
            format!("failed to open git repository at {}", dep_dir.display())
        })?;
        
        self.disable_ssl_verify(&repo)?;

        let mut remote = repo
            .find_remote("origin")
            .context("failed to find remote 'origin'")?;

        let mut callbacks = RemoteCallbacks::new();
        let resolving_printed_clone = resolving_printed.clone();
        callbacks.transfer_progress(move |stats| {
            if stats.received_objects() == stats.total_objects() && stats.total_objects() > 0 {
                if !resolving_printed_clone.swap(true, Ordering::Relaxed) {}
            } else {
                resolving_printed_clone.store(false, Ordering::Relaxed);
            }
            true
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        remote
            .fetch(
                &["refs/heads/*:refs/remotes/origin/*"],
                Some(&mut fetch_opts),
                None,
            )
            .with_context(|| format!("failed to fetch from {git_url}"))?;

        eprintln!();

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

    fn clone_new_repo(
        &self,
        dep_dir: &PathBuf,
        git_url: &str,
        resolving_printed: Arc<AtomicBool>,
    ) -> Result<String> {
        use git2::build::CheckoutBuilder;

        let mut callbacks = RemoteCallbacks::new();
        let resolving_printed_clone = resolving_printed.clone();
        callbacks.transfer_progress(move |stats| {
            if stats.received_objects() == stats.total_objects() && stats.total_objects() > 0 {
                if !resolving_printed_clone.swap(true, Ordering::Relaxed) {
                    eprint!("Resolving deltas...");
                }
            } else {
                resolving_printed_clone.store(false, Ordering::Relaxed);
            }
            true
        });

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.remote_callbacks(callbacks);

        let repo = Repository::init(dep_dir).with_context(|| {
            format!("failed to initialize repository at {}", dep_dir.display())
        })?;

        let mut remote = repo
            .remote("origin", git_url)
            .with_context(|| format!("failed to add remote origin for {git_url}"))?;

        remote
            .fetch(
                &["refs/heads/*:refs/remotes/origin/*"],
                Some(&mut fetch_opts),
                None,
            )
            .with_context(|| format!("failed to fetch from {git_url}"))?;

        eprintln!();

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