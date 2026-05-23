use anyhowed::{Context, Result};
use std::collections::HashMap;
use std::fs;

use crow_utils::environment::Environment;
use crow_utils::status;

use super::fetcher::{RegistryEntry, RegistryIndex};
use super::git_ops::{checkout_default_branch, clone_registry, commit_and_push};
use super::github::GithubRepo;

pub fn delete(package_name: &str, version: Option<&str>, registry_url: &str) -> Result<()> {
    let token = Environment::github_token()?;

    let (_temp_dir, registry_repo) = clone_registry(registry_url, &token)?;
    let base_branch = checkout_default_branch(&registry_repo)?;

    let packages_dir = registry_repo.workdir().unwrap().join("packages");
    let index_file = packages_dir.join(format!("{package_name}.toml"));

    if !index_file.exists() {
        anyhowed::bail!("Package '{package_name}' not found in registry");
    }

    let existing_content = fs::read_to_string(&index_file)?;
    let mut versions: HashMap<String, RegistryEntry> = {
        let index: RegistryIndex = toml::from_str(&existing_content)
            .with_context(|| format!("Failed to parse registry index for {package_name}"))?;
        index.versions
    };

    let versions_to_delete: Vec<String> = match version {
        Some(ver) => {
            if !versions.contains_key(ver) {
                anyhowed::bail!("Version '{ver}' of package '{package_name}' not found in registry");
            }
            vec![ver.to_string()]
        }
        None => versions.keys().cloned().collect(),
    };

    for ver in &versions_to_delete {
        versions.remove(ver);
    }

    let delete_branch = format!("delete/{}/{}", package_name, version.unwrap_or("all"));
    let head_commit = registry_repo.head()?.peel_to_commit()?;
    registry_repo.branch(&delete_branch, &head_commit, false)?;

    let branch_ref = registry_repo.find_reference(&format!("refs/heads/{delete_branch}"))?;
    registry_repo.checkout_tree(&branch_ref.peel_to_tree()?.into_object(), None)?;
    registry_repo.set_head(branch_ref.name().unwrap())?;

    if versions.is_empty() {
        fs::remove_file(&index_file)
            .with_context(|| format!("Failed to delete {}", index_file.display()))?;
    } else {
        let index = RegistryIndex { versions };
        let content = toml::to_string_pretty(&index)?;
        fs::write(&index_file, &content)?;
    }

    let commit_message = match version {
        Some(ver) => format!("delete: {package_name} v{ver}"),
        None => format!("delete: {package_name} (all versions)"),
    };

    let push_result = commit_and_push(
        &registry_repo,
        &delete_branch,
        &commit_message,
        "crow-delete",
        &token,
    );

    if let Err(e) = push_result {
        let msg = e.to_string();
        if msg.contains("non-fastforward") || msg.contains("already exists") {
            let repo_info = GithubRepo::from_url(registry_url)?;
            let prs_url = format!(
                "https://github.com/{}/{}/pulls?q=head:{}",
                repo_info.owner, repo_info.repo, delete_branch
            );
            status!("Note", "A pull request may already exist: {}", prs_url);
            anyhowed::bail!(
                "Branch '{delete_branch}' already exists in registry; a PR for this deletion likely already exists."
            );
        }
        return Err(e.context("Failed to push to registry"));
    }

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
