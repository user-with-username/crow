use anyhowed::Result;
use clap::Args;
use crow_core::{CrowConfig, Project};

#[derive(Args)]
pub struct FmtArgs {}

pub struct FmtCommand {
    _args: FmtArgs,
}

impl FmtCommand {
    pub fn new(args: FmtArgs) -> Self {
        Self { _args: args }
    }

    pub fn execute(self) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let (loaded_config, manifest_dir) = CrowConfig::load_from(&current_dir, false)?;
        let project = Project::new(loaded_config, manifest_dir, "dev", None)?;
        project.fmt()?;

        Ok(())
    }
}
