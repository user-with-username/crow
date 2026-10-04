use anyhowed::{Context, Result};
use std::{
    env::{self, var_os},
    path::{Path, PathBuf},
};

pub const DEFAULT_REGISTRY_URL: &str = "https://github.com/user-with-username/crow-registry";

pub struct Environment;

impl Environment {
    pub const TARGET_DIR_VAR: &'static str = "CROW_TARGET_DIR";
    pub const REGISTRY_VAR: &'static str = "CROW_REGISTRY";
    pub const HOME_VAR: &'static str = "CROW_HOME";
    pub const INSTALL_ROOT_VAR: &'static str = "CROW_INSTALL_ROOT";
    pub const JOBS_VAR: &'static str = "CROW_JOBS";

    pub fn build_jobs() -> usize {
        env::var(Self::JOBS_VAR)
            .ok()
            .and_then(|v| v.trim().parse::<usize>().ok())
            .filter(|n| *n > 0)
            .unwrap_or_else(|| {
                std::thread::available_parallelism()
                    .map(|n| (n.get() / 2).max(1))
                    .unwrap_or(1)
            })
    }

    pub fn target_dir() -> PathBuf {
        var_os(Self::TARGET_DIR_VAR)
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target"))
    }

    pub fn github_token() -> Result<String> {
        env::var("GITHUB_TOKEN").map_err(|_| {
            anyhowed::anyhow!(
                "GITHUB_TOKEN environment variable is required.\n\
                 Create a token with 'repo' scope at https://github.com/settings/tokens"
            )
        })
    }

    pub fn registry_url() -> String {
        env::var(Self::REGISTRY_VAR).unwrap_or_else(|_| DEFAULT_REGISTRY_URL.to_string())
    }

    pub fn user_home() -> Option<PathBuf> {
        var_os("USERPROFILE")
            .or_else(|| var_os("HOME"))
            .map(PathBuf::from)
    }

    pub fn crow_home() -> Result<PathBuf> {
        if let Some(home) = var_os(Self::HOME_VAR) {
            return Ok(PathBuf::from(home));
        }

        let user_home = Self::user_home().with_context(|| {
            format!(
                "Cannot determine the home directory, set {} or use `--root`",
                Self::HOME_VAR
            )
        })?;

        Ok(user_home.join(".crow"))
    }

    pub fn install_bin_dir(root: Option<&Path>) -> Result<PathBuf> {
        if let Some(root) = root {
            return Ok(root.join("bin"));
        }

        if let Some(root) = var_os(Self::INSTALL_ROOT_VAR) {
            return Ok(PathBuf::from(root).join("bin"));
        }

        Ok(Self::crow_home()?.join("bin"))
    }

    pub fn is_in_path(dir: &Path) -> bool {
        let normalize = |p: &Path| p.canonicalize().unwrap_or_else(|_| p.to_path_buf());
        let target = normalize(dir);

        var_os("PATH")
            .map(|paths| env::split_paths(&paths).any(|p| normalize(&p) == target))
            .unwrap_or(false)
    }
}
