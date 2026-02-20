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
        let config = CrowConfig::load()?;
        let project = Project::new(config, "dev")?;

        let profile_dir = project.profile_dir();

        println!("{}", profile_dir.display());

        if profile_dir.exists() {
            status!("Cleaning", "{}", profile_dir.display());
            std::fs::remove_dir_all(&profile_dir)?;
            status!("Deleted", "profile directory");
        }

        Ok(())
    }
}
