use anyhow::{bail, Context, Result};
use git2::{FetchOptions, RemoteCallbacks, Repository};
use tempfile::TempDir;

pub fn auth_callbacks(token: &str) -> RemoteCallbacks<'static> {
    let token = token.to_string();
    let mut cb = RemoteCallbacks::new();
    cb.credentials(move |_url, username, _allowed| {
        git2::Cred::userpass_plaintext(username.unwrap_or("x-access-token"), &token)
    });
    cb
}

pub fn auth_fetch_opts(token: &str) -> FetchOptions<'static> {
    let mut opts = FetchOptions::new();
    opts.remote_callbacks(auth_callbacks(token));
    opts
}

pub fn clone_registry(registry_url: &str, token: &str) -> Result<(TempDir, Repository)> {
    let temp_dir = TempDir::new()?;
    let clone_path = temp_dir.path().join("registry");

    let mut builder = git2::build::RepoBuilder::new();
    builder.fetch_options(auth_fetch_opts(token));
    builder
        .clone(registry_url, &clone_path)
        .with_context(|| format!("Failed to clone registry from {registry_url}"))?;

    let repo = Repository::open(&clone_path)?;

    {
        let mut remote = repo.find_remote("origin")?;
        let mut opts = auth_fetch_opts(token);
        remote
            .fetch(
                &["refs/heads/*:refs/remotes/origin/*"],
                Some(&mut opts),
                None,
            )
            .with_context(|| format!("Failed to fetch branches from registry {registry_url}"))?;
    }

    Ok((temp_dir, repo))
}

pub fn checkout_default_branch(repo: &Repository) -> Result<String> {
    let default_ref = find_default_branch(repo)?;

    let commit = if let Ok(head_ref) = repo.find_reference(&default_ref) {
        repo.reference_to_annotated_commit(&head_ref)?
    } else {
        let remote_ref = default_ref.replace("refs/heads/", "refs/remotes/origin/");
        let head_ref = repo.find_reference(&remote_ref)?;
        repo.reference_to_annotated_commit(&head_ref)?
    };

    let commit_obj = repo.find_commit(commit.id())?;
    let tree = commit_obj.tree()?;
    repo.checkout_tree(&tree.into_object(), None)?;
    repo.set_head(&default_ref)?;

    let short_name = default_ref.trim_start_matches("refs/heads/").to_string();
    Ok(short_name)
}

pub fn commit_and_push(
    repo: &Repository,
    branch_name: &str,
    message: &str,
    author_name: &str,
    token: &str,
) -> Result<()> {
    let mut index = repo.index()?;
    index.add_all(["packages"].iter(), git2::IndexAddOption::DEFAULT, None)?;
    index.write()?;

    let tree_oid = index.write_tree()?;
    let tree = repo.find_tree(tree_oid)?;
    let parent_commit = repo.head()?.peel_to_commit()?;
    let sig = git2::Signature::now(author_name, "crow@localhost")?;

    repo.commit(Some("HEAD"), &sig, &sig, message, &tree, &[&parent_commit])?;

    let mut push_opts = git2::PushOptions::new();
    push_opts.remote_callbacks(auth_callbacks(token));
    let mut remote = repo.find_remote("origin")?;

    let refspec = format!("refs/heads/{branch_name}:refs/heads/{branch_name}");
    remote
        .push(&[&refspec], Some(&mut push_opts))
        .with_context(|| format!("Failed to push branch {branch_name}"))?;

    Ok(())
}

pub fn find_default_branch(repo: &Repository) -> Result<String> {
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
                let branch = target.replace("refs/remotes/origin/", "refs/heads/");
                if repo.find_reference(&branch).is_ok() {
                    return Ok(branch);
                }
            }
        }
    }

    for candidate in &["refs/heads/main", "refs/heads/master", "refs/heads/trunk"] {
        if repo.find_reference(candidate).is_ok() {
            return Ok(candidate.to_string());
        }
    }

    for candidate in &["refs/remotes/origin/main", "refs/remotes/origin/master"] {
        if repo.find_reference(candidate).is_ok() {
            let local = candidate.replace("refs/remotes/origin/", "refs/heads/");
            return Ok(local);
        }
    }

    bail!("failed to find default branch in registry");
}
