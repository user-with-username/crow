pub mod build;
pub mod clean;
pub mod run;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Compile the current project
    Build(build::BuildArgs),
    /// Run the current project
    Run(run::RunArgs),
    /// Clean target directory of the current project
    Clean(clean::CleanArgs),
}

impl Command {
    pub fn execute(self) -> Result<()> {
        match self {
            Self::Build(args) => build::BuildCommand::new(args).execute(),
            Self::Run(args) => run::RunCommand::new(args).execute(),
            Self::Clean(args) => clean::CleanCommand::new(args).execute(),
        }
    }
}
