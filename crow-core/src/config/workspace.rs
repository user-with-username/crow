use crate::config::CrowConfig;
use anyhowed::{Context, Result};
use serde::Deserialize;
use std::path::{Path, PathBuf};

#[derive(Deserialize, Debug, Default, Clone)]
pub struct WorkspaceConfig {
    pub members: Vec<String>,
}

pub struct Workspace {
    pub root: PathBuf,
    members: Vec<(CrowConfig, PathBuf)>,
}

impl Workspace {
    pub fn from_config(config: CrowConfig, root: PathBuf) -> Result<Self> {
        let members = if config.is_virtual() {
            let workspace = config
                .workspace
                .as_ref()
                .context("virtual manifest missing [workspace]")?;
            let mut members = Vec::new();
            for member_path in &workspace.members {
                let member_full_path = root.join(member_path);
                let (member_config, _) = CrowConfig::load_from(&member_full_path, false)?;
                members.push((member_config, member_full_path));
            }
            members
        } else {
            vec![(config, root.clone())]
        };

        Ok(Self { root, members })
    }

    pub fn load_from(dir: &Path) -> Result<Self> {
        let (config, root) = CrowConfig::load_from(dir, false)?;
        Self::from_config(config, root)
    }

    pub fn load() -> Result<Self> {
        let current_dir = std::env::current_dir().context("failed to get current directory")?;
        let (config, root) = CrowConfig::find_in_tree(&current_dir)?;
        Self::from_config(config, root)
    }

    pub fn members(&self) -> impl Iterator<Item = &(CrowConfig, PathBuf)> {
        self.members.iter()
    }

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

    pub fn find_member_by_name(&self, name: &str) -> Option<&(CrowConfig, PathBuf)> {
        self.members
            .iter()
            .find(|(cfg, _)| cfg.package.as_ref().map(|p| p.name.as_str()) == Some(name))
    }

    pub fn len(&self) -> usize {
        self.members.len()
    }

    pub fn is_empty(&self) -> bool {
        self.members.is_empty()
    }
}
