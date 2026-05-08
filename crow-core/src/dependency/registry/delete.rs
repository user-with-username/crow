use anyhow::{Context, Result};
use std::fs;

use crow_utils::environment::Environment;

use super::git_ops::{checkout_default_branch, clone_registry, commit_and_push};
use super::github::GithubRepo;

pub fn delete(package_name: &str, version: Option<&str>, registry_url: &str) -> Result<()> {
    let token = Environment::github_token()?;

    let (_temp_dir, registry_repo) = clone_registry(registry_url, &token)?;
    let base_branch = checkout_default_branch(&registry_repo)?;

    let package_dir = registry_repo
        .workdir()
        .unwrap()
        .join("packages")
        .join(package_name);

    if !package_dir.exists() {
        anyhow::bail!("Package '{package_name}' not found in registry");
    }

    let files_to_delete: Vec<std::path::PathBuf> = match version {
        Some(ver) => {
            let path = package_dir.join(format!("{ver}.toml"));
            if !path.exists() {
                anyhow::bail!("Version '{ver}' of package '{package_name}' not found in registry");
            }
            vec![path]
        }
        None => {
            let files: Vec<_> = fs::read_dir(&package_dir)?
                .flatten()
                .filter(|e| e.path().extension().map_or(false, |ext| ext == "toml"))
                .map(|e| e.path())
                .collect();
            if files.is_empty() {
                anyhow::bail!("No versions found for package '{package_name}'");
            }
            files
        }
    };

    let delete_branch = format!("delete/{}/{}", package_name, version.unwrap_or("all"));
    let head_commit = registry_repo.head()?.peel_to_commit()?;
    registry_repo.branch(&delete_branch, &head_commit, false)?;

    let branch_ref = registry_repo.find_reference(&format!("refs/heads/{delete_branch}"))?;
    registry_repo.checkout_tree(&branch_ref.peel_to_tree()?.into_object(), None)?;
    registry_repo.set_head(branch_ref.name().unwrap())?;

    for file in &files_to_delete {
        fs::remove_file(file).with_context(|| format!("Failed to delete {}", file.display()))?;
    }

    if version.is_none() {
        if let Ok(mut entries) = fs::read_dir(&package_dir) {
            if entries.next().is_none() {
                fs::remove_dir(&package_dir)?;
            }
        }
    }

    let commit_message = match version {
        Some(ver) => format!("delete: {package_name} v{ver}"),
        None => format!("delete: {package_name} (all versions)"),
    };

    commit_and_push(
        &registry_repo,
        &delete_branch,
        &commit_message,
        "crow-delete",
        &token,
    )?;

    let pr_title = commit_message.clone();
    let pr_body = match version {
        Some(ver) => format!(
            "Delete `{package_name}` version `{ver}`\n\n\
             This PR removes version {ver} of package {package_name}."
        ),
        None => format!(
            "Delete `{package_name}` (all versions)\n\n\
             This PR removes all versions of package {package_name}."
        ),
    };

    GithubRepo::from_url(registry_url)?.create_pr(
        &token,
        &pr_title,
        &delete_branch,
        &base_branch,
        &pr_body,
        "crow-delete",
    )?;

    Ok(())
}
