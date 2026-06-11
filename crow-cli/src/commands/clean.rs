use anyhowed::Result;
use clap::Args;
use crow_core::{CrowConfig, Project};
use crow_utils::status;

#[derive(Args)]
pub struct CleanArgs {
    /// Named target or conditional target (e.g., client, "cfg(windows)")
    #[arg(long)]
    pub target: Option<String>,
}

pub struct CleanCommand {
    args: CleanArgs,
}

impl CleanCommand {
    pub fn new(args: CleanArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let (config, manifest_dir) =
            CrowConfig::find_in_tree(&current_dir, self.args.target.as_deref())?;
        let project = Project::new(config, manifest_dir, "dev", None)?;

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
