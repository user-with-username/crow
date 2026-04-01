use serde::Deserialize;
use std::path::PathBuf;
use anyhow::{Context, Result};
use crate::config::CrowConfig;

#[derive(Deserialize, Debug, Default, Clone)]
pub struct WorkspaceConfig {
    /// List of relative paths to workspace members
    pub members: Vec<String>,
}

/// A workspace that can contain one or more packages.
/// For a non‑virtual manifest, the workspace contains a single package.
pub struct Workspace {
    pub root: PathBuf,
    members: Vec<(CrowConfig, PathBuf)>,
}

impl Workspace {
    /// Creates a new workspace from a loaded config and its root directory.
    /// If the config is virtual, it loads all member packages.
    /// If it is a regular package, it creates a workspace with that single package.
    pub fn from_config(config: CrowConfig, root: PathBuf) -> Result<Self> {
        let members = if config.is_virtual() {
            let workspace = config.workspace.as_ref()
                .context("virtual manifest missing [workspace]")?;
            let mut members = Vec::new();
            for member_path in &workspace.members {
                let member_full_path = root.join(member_path);
                let (member_config, _) = CrowConfig::load_from(&member_full_path.join("crow.toml"))?;
                members.push((member_config, member_full_path));
            }
            members
        } else {
            // Clone root so it can be used again for the struct field
            vec![(config, root.clone())]
        };

        Ok(Self { root, members })
    }

    /// Loads the workspace from the current directory (searching upwards for `crow.toml`).
    pub fn load() -> Result<Self> {
        let (config, root) = CrowConfig::load()?;
        Self::from_config(config, root)
    }

    /// Returns an iterator over all member packages (config + root path).
    pub fn members(&self) -> impl Iterator<Item = &(CrowConfig, PathBuf)> {
        self.members.iter()
    }

    /// Returns the member packages that are binaries.
    pub fn binary_members(&self) -> Vec<&(CrowConfig, PathBuf)> {
        self.members
            .iter()
            .filter(|(cfg, _)| {
    cfg.package
        .as_ref()
        .map(|p| p.r#type.is_bin())
        .unwrap_or(false)
})

            .collect()
    }

    /// Finds a member by package name.
    pub fn find_member_by_name(&self, name: &str) -> Option<&(CrowConfig, PathBuf)> {
        self.members
            .iter()
            .find(|(cfg, _)| {
                cfg.package
                    .as_ref()
                    .map(|p| p.name.as_str()) == Some(name)
            })
    }

    /// Returns the number of members in the workspace.
    pub fn len(&self) -> usize {
        self.members.len()
    }

    /// Returns true if the workspace has no members.
    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}