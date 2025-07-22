use anyhow::Result;
use clap::{Args, Subcommand};
use crow_utils::logger::Logger;

mod build;
mod clean;
mod init;
mod run;

pub use build::BuildCommand;
pub use clean::CleanCommand;
pub use init::InitCommand;
pub use run::RunCommand;

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new CRow project
    Init(InitCommand),
    /// Build the current project
    Build(BuildCommand),
    /// Clean build artifacts
    Clean(CleanCommand),
    /// Build and run the project
    Run(RunCommand),
}

pub trait Command {
    fn execute(&self, logger: &mut Logger) -> Result<()>;
}

impl Command for Commands {
    fn execute(&self, logger: &mut Logger) -> Result<()> {
        match self {
            Self::Init(cmd) => cmd.execute(logger),
            Self::Build(cmd) => cmd.execute(logger),
            Self::Clean(cmd) => cmd.execute(logger),
            Self::Run(cmd) => cmd.execute(logger),
        }
    }
}
