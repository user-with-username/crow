use anyhow::{Context, Result};
use clap::Args;
use crow_core::{CrowConfig, Project};
use crow_utils::status;

#[derive(Args)]
pub struct CleanArgs {}

pub struct CleanCommand {
    _args: CleanArgs,
}

impl CleanCommand {
    pub fn new(args: CleanArgs) -> Self {
        Self { _args: args }
    }

    pub fn execute(self) -> Result<()> {
        let config = CrowConfig::load()?;
        let project = Project::new(config, "dev")?;

        let profile_dir = project.profile_dir();
        let target_dir = profile_dir
            .parent()
            .context("Profile directory has no parent. How is it possible?")?;

        if target_dir.exists() {
            status!("Cleaning", "{}", target_dir.display());
            std::fs::remove_dir_all(target_dir)?;
            status!("Deleted", "target directory");
        }

        Ok(())
    }
}