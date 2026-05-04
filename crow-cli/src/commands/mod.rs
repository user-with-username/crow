pub mod build;
pub mod clean;
pub mod init;
pub mod new;
pub mod run;

use anyhow::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Create a new Crow package
    New(new::NewArgs),

    /// Initialize a new Crow package in an existing directory
    Init(init::InitArgs),

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
            Self::New(args) => new::NewCommand::new(args).execute(),
            Self::Init(args) => init::InitCommand::new(args).execute(),
            Self::Build(args) => build::BuildCommand::new(args).execute(),
            Self::Run(args) => run::RunCommand::new(args).execute(),
            Self::Clean(args) => clean::CleanCommand::new(args).execute(),
        }
    }
}
