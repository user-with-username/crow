use anyhowed::{Context, Result};
use dirs::home_dir;
use git2::{FetchOptions, Repository, SubmoduleUpdateOptions};
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

        status!("   Downloading", "{}", dep_name);

        let repo = Repository::open(dep_dir)
            .with_context(|| format!("failed to open git repository at {}", dep_dir.display()))?;

        let mut remote = repo
            .find_remote("origin")
            .context("failed to find remote 'origin'")?;

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.depth(1);
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

        self.update_submodules(&repo, true)?;

        Ok(commit_id.to_string())
    }

    fn clone_new_repo(&self, dep_dir: &PathBuf, git_url: &str, dep_name: &str) -> Result<String> {
        use crow_utils::status;
        use git2::build::CheckoutBuilder;

        status!("   Downloading", "{}", dep_name);

        let repo = Repository::init(dep_dir)
            .with_context(|| format!("failed to initialize repository at {}", dep_dir.display()))?;

        let mut remote = repo
            .remote("origin", git_url)
            .with_context(|| format!("failed to add remote origin for {git_url}"))?;

        let mut fetch_opts = FetchOptions::new();
        fetch_opts.depth(1);
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

        self.update_submodules(&repo, true)?;

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

        status!("   Downloading", "{}", dep_name);

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

        self.update_submodules(&repo, true)?;

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

        status!("Checking out", "{}", dep_name);

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

        self.update_submodules(&repo, true)?;

        Ok(oid.to_string())
    }

    fn update_submodules(&self, repo: &Repository, recursive: bool) -> Result<()> {
        let submodules = repo.submodules()?;

        let parent_remote = repo
            .find_remote("origin")
            .or_else(|_| repo.find_remote("upstream"))
            .ok();
        let parent_url = parent_remote.as_ref().and_then(|r| r.url());

        for mut sub in submodules {
            let name = sub.name().unwrap_or("unknown").to_string();

            sub.init(false)
                .with_context(|| format!("failed to init submodule '{name}'"))?;

            let mut fetch_opts = FetchOptions::new();
            fetch_opts.depth(1);
            fetch_opts.update_fetchhead(true);

            let mut update_opts = SubmoduleUpdateOptions::new();
            update_opts.fetch(fetch_opts);

            let mut cb = git2::build::CheckoutBuilder::new();
            cb.force();
            update_opts.checkout(cb);

            if let Err(e) = sub.update(true, Some(&mut update_opts)) {
                if e.code() == git2::ErrorCode::NotFound || e.class() == git2::ErrorClass::Odb {
                    let sub_repo = sub.open().with_context(|| {
                        format!("failed to open broken submodule '{name}' for manual recovery")
                    })?;

                    let target_id = sub.index_id().with_context(|| {
                        format!("submodule '{name}' doesn't specify a target commit id")
                    })?;

                    let raw_url = sub
                        .url()
                        .with_context(|| format!("submodule '{name}' has no remote URL"))?;

                    // resolves "../bloom.git" to "https://github.com/boostorg/bloom"
                    let absolute_url = if raw_url.starts_with("../") {
                        if let Some(p_url) = parent_url {
                            let base = match p_url.rfind('/') {
                                Some(idx) => &p_url[..idx],
                                None => p_url,
                            };
                            let sub_clean = raw_url.trim_start_matches("../");
                            format!("{}/{}", base, sub_clean)
                        } else {
                            anyhowed::bail!("Submodule '{name}' uses relative URL, but parent repository remote URL is unknown");
                        }
                    } else {
                        raw_url.to_string()
                    };

                    let mut remote = sub_repo.remote_anonymous(&absolute_url)
                        .with_context(|| format!("failed to create anonymous remote for '{name}' using URL '{absolute_url}'"))?;

                    let mut manual_fetch_opts = FetchOptions::new();

                    remote.fetch(
                        &[target_id.to_string()],
                        Some(&mut manual_fetch_opts),
                        None,
                    ).with_context(|| format!("manual fallback fetch failed for submodule '{name}' at commit {target_id} (URL: {absolute_url})"))?;

                    let obj = sub_repo.find_object(target_id, None)?;
                    let mut manual_cb = git2::build::CheckoutBuilder::new();
                    manual_cb.force();

                    sub_repo.checkout_tree(&obj, Some(&mut manual_cb))?;
                    sub_repo.set_head_detached(target_id)?;
                } else {
                    return Err(e).with_context(|| format!("failed to update submodule '{name}'"));
                }
            }

            if recursive {
                let sub_repo = sub.open().with_context(|| {
                    format!("failed to open submodule repo '{name}' for recursive update")
                })?;
                self.update_submodules(&sub_repo, true)?;
            }
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
