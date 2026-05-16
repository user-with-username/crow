use anyhow::{Context, Result};
use std::collections::HashMap;
use std::fs;

use crow_utils::environment::Environment;

use super::git_ops::{checkout_default_branch, clone_registry, commit_and_push};
use super::github::GithubRepo;
use super::fetcher::{RegistryEntry, RegistryIndex};

pub fn publish(
    project_root: &std::path::PathBuf,
    package_name: &str,
    version: &str,
    registry_url: &str,
) -> Result<bool> {
    let token = Environment::github_token()?;

    let repo = git2::Repository::open(project_root)
        .with_context(|| format!("Not a git repository: {}", project_root.display()))?;
    let git_url = repo
        .find_remote("origin")
        .context("No remote 'origin' found in repository")?
        .url()
        .context("Remote 'origin' has no URL")?
        .to_string();
    let commit_sha = repo
        .head()
        .context("No HEAD commit found")?
        .target()
        .context("HEAD is not a commit (maybe detached?)")?
        .to_string();

    let (_temp_dir, registry_repo) = clone_registry(registry_url, &token)?;
    let base_branch = checkout_default_branch(&registry_repo)?;

    let packages_dir = registry_repo
        .workdir()
        .unwrap()
        .join("packages");
    let index_file = packages_dir.join(format!("{package_name}.toml"));

    let mut versions: HashMap<String, RegistryEntry> = if index_file.exists() {
        let existing_content = fs::read_to_string(&index_file)?;
        let existing_index: RegistryIndex = toml::from_str(&existing_content).with_context(|| {
            format!("Failed to parse existing registry index for {package_name}")
        })?;

        if existing_index.versions.contains_key(version) {
            let existing_entry = &existing_index.versions[version];
            if existing_entry.git == git_url && existing_entry.commit == commit_sha {
                return Ok(false);
            }

            anyhow::bail!(
                "Version {version} of package '{package_name}' is already published with a different commit.\n\
                 Existing: git = \"{}\", commit = \"{}\"\n\
                 New: git = \"{}\", commit = \"{}\"",
                existing_entry.git,
                existing_entry.commit,
                git_url,
                commit_sha
            );
        }
        existing_index.versions
    } else {
        HashMap::new()
    };

    versions.insert(
        version.to_string(),
        RegistryEntry {
            git: git_url.clone(),
            commit: commit_sha.clone(),
        },
    );

    let publish_branch = format!("publish/{package_name}/{version}");
    let head_commit = registry_repo.head()?.peel_to_commit()?;
    let branch = registry_repo.branch(&publish_branch, &head_commit, false)?;
    let branch_ref = branch.get();
    registry_repo.checkout_tree(&branch_ref.peel_to_tree()?.into_object(), None)?;
    registry_repo.set_head(branch_ref.name().unwrap())?;

    fs::create_dir_all(&packages_dir)?;
    let index = RegistryIndex { versions };
    let content = toml::to_string_pretty(&index)?;
    fs::write(&index_file, &content)?;

    let push_result = commit_and_push(
        &registry_repo,
        &publish_branch,
        &format!("publish: {package_name} v{version}"),
        "crow-publish",
        &token,
    );

    if let Err(e) = push_result {
        let msg = e.to_string();
        if msg.contains("non-fastforward") || msg.contains("already exists") {
            anyhow::bail!(
                "Version {version} of package '{package_name}' has already been published \
                 (branch '{publish_branch}' already exists in registry)."
            );
        }
        return Err(e.context("Failed to push to registry"));
    }

    GithubRepo::from_url(registry_url)?.create_pr(
        &token,
        &format!("publish: {package_name} v{version}"),
        &publish_branch,
        &base_branch,
        &format!(
            "Publish `{package_name}` version `{version}`\n\n\
             - Git URL: {git_url}\n\
             - Commit: `{commit_sha}`\n\n\
             Auto-merge will be triggered if CI passes."
        ),
        "crow-publish",
    )?;

    Ok(true)
}
