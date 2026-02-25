use anyhow::Result;
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
        let (config, manifest_dir) = CrowConfig::load()?;
        let project = Project::new(config, manifest_dir, "dev")?;

        let target_dir = project.target_dir();

        if target_dir.exists() {
            status!("Cleaning", "{}", target_dir.display());
            project.clean()?;
            
            status!("Deleted", "target directory");
        } else {
            status!("Finished", "nothing to clean");
        }

        Ok(())
    }
}