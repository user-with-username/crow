use anyhowed::Result;
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
        let current_dir = std::env::current_dir()?;
        let (config, manifest_dir) = CrowConfig::find_in_tree(&current_dir)?;
        let project = Project::new(config, manifest_dir, "dev", None, None)?;

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
