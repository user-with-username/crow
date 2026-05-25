use anyhowed::{Context, Result};
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
    let branch_name = find_default_branch(repo)?;
    let local_ref = format!("refs/heads/{branch_name}");
    let remote_ref = format!("refs/remotes/origin/{branch_name}");

    let tracking = repo
        .find_reference(&remote_ref)
        .with_context(|| format!("Remote-tracking ref '{remote_ref}' not found"))?;
    let commit_id = tracking.peel_to_commit()?.id();

    if repo.find_reference(&local_ref).is_err() {
        repo.reference(
            &local_ref,
            commit_id,
            false,
            &format!("branch: Created from {remote_ref}"),
        )
        .with_context(|| format!("Failed to create local branch '{branch_name}'"))?;
    }

    let commit_obj = repo.find_commit(commit_id)?;
    repo.checkout_tree(&commit_obj.tree()?.into_object(), None)?;
    repo.set_head(&local_ref)?;

    Ok(branch_name)
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

    let mut callbacks = auth_callbacks(token);
    callbacks.push_update_reference(|refname, status| {
        if let Some(status) = status {
            eprintln!("Push rejected for {}: {}", refname, status);
            if status.contains("permission") || status.contains("403") {
                panic!("Permission denied. Try using a Personal Access Token with 'repo' scope");
            }
        }
        Ok(())
    });
    push_opts.remote_callbacks(callbacks);

    let mut remote = repo.find_remote("origin")?;

    let refspec = format!("refs/heads/{branch_name}:refs/heads/{branch_name}");
    remote
        .push(&[&refspec], Some(&mut push_opts))
        .with_context(|| format!("Failed to push branch {branch_name}"))?;

    Ok(())
}

pub fn find_default_branch(repo: &Repository) -> Result<String> {
    if let Some(name) = query_remote_head(repo) {
        let remote_ref = format!("refs/remotes/origin/{name}");
        if repo.find_reference(&remote_ref).is_ok() {
            return Ok(name);
        }
    }

    if let Ok(r) = repo.find_reference("refs/remotes/origin/HEAD") {
        if let Some(target) = r.symbolic_target() {
            // target: "refs/remotes/origin/main"
            if let Some(name) = target.strip_prefix("refs/remotes/origin/") {
                if !name.is_empty() {
                    return Ok(name.to_string());
                }
            }
        }
    }

    let mut names: Vec<String> = repo
        .references()
        .into_iter()
        .flatten()
        .flatten()
        .filter_map(|r| {
            let name = r.name()?.to_string();
            let branch = name.strip_prefix("refs/remotes/origin/")?.to_string();
            (!branch.is_empty() && branch != "HEAD").then_some(branch)
        })
        .collect();

    names.sort_unstable();

    names.into_iter().next().with_context(|| {
        "cannot determine default branch: remote did not advertise HEAD \
         and no remote-tracking branches found"
    })
}

fn query_remote_head(repo: &Repository) -> Option<String> {
    let mut remote = repo.find_remote("origin").ok()?;
    remote.connect(git2::Direction::Fetch).ok()?;
    let buf = remote.default_branch().ok()?;
    let full_ref = buf.as_str()?; // "refs/heads/main"
    remote.disconnect().ok();

    let name = full_ref
        .strip_prefix("refs/heads/")
        .unwrap_or(full_ref)
        .to_string();

    if name.is_empty() {
        None
    } else {
        Some(name)
    }
}
