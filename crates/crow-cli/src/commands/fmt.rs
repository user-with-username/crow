use anyhowed::Result;
use clap::Args;
use crow_core::{CrowConfig, Project};

#[derive(Args)]
pub struct FmtArgs {
    /// Check formatting without writing changes
    #[arg(long)]
    pub check: bool,
    /// Named target or conditional target (e.g., client, "cfg(windows)")
    #[arg(long)]
    pub target: Option<String>,
}

pub struct FmtCommand {
    args: FmtArgs,
}

impl FmtCommand {
    pub fn new(args: FmtArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let (loaded_config, manifest_dir) =
            CrowConfig::load_from(&current_dir, false, self.args.target.as_deref())?;
        let project = Project::new(loaded_config, manifest_dir, "dev", None)?;

        project.fmt(!self.args.check)?;

        Ok(())
    }
}
