use std::{env::var_os, path::PathBuf};

pub struct Environment;

impl Environment {
    pub fn target_dir() -> std::path::PathBuf {
        var_os("CROW_TARGET_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("target"))
    }

    pub fn github_token() -> anyhow::Result<String> {
        std::env::var("GITHUB_TOKEN").map_err(|_| {
            anyhow::anyhow!(
                "GITHUB_TOKEN environment variable is required.\n\
                 Create a token with 'repo' scope at https://github.com/settings/tokens"
            )
        })
    }
}
