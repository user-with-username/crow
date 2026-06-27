pub mod add;
pub mod bench;
pub mod build;
pub mod check;
pub mod clean;
pub mod delete;
pub mod fmt;
pub mod init;
pub mod metadata;
pub mod new;
pub mod publish;
pub mod run;
pub mod self_update;
pub mod test;

use anyhowed::Result;
use clap::Subcommand;

#[derive(Subcommand)]
pub enum Command {
    /// Create a new Crow package
    New(new::NewArgs),

    /// Initialize a new Crow package in an existing directory
    Init(init::InitArgs),

    /// Add a dependency from the registry
    Add(add::AddArgs),

    /// Check the current project for errors without linking
    Check(check::CheckArgs),

    /// Compile the current project
    Build(build::BuildArgs),

    /// Run the current project
    Run(run::RunArgs),

    /// Build and run the test binary (`test` profile; like `cargo test`)
    Test(test::TestArgs),

    /// Build and run the benchmark binary (`bench` profile; like `cargo bench`)
    Bench(bench::BenchArgs),

    /// Clean target directory of the current project
    Clean(clean::CleanArgs),

    /// Publish a library package to the registry
    Publish(publish::PublishArgs),

    /// Delete a library package from the registry
    Delete(delete::DeleteArgs),

    /// Update crow to the latest release
    SelfUpdate(self_update::SelfUpdateArgs),

    /// Show all info
    Metadata(metadata::MetadataArgs),

    /// Format all files in the project
    Fmt(fmt::FmtArgs),
}

impl Command {
    pub fn execute(self) -> Result<()> {
        match self {
            Self::New(args) => new::NewCommand::new(args).execute(),
            Self::Init(args) => init::InitCommand::new(args).execute(),
            Self::Add(args) => add::AddCommand::new(args).execute(),
            Self::Check(args) => check::CheckCommand::new(args).execute(),
            Self::Build(args) => build::BuildCommand::new(args).execute(),
            Self::Run(args) => run::RunCommand::new(args).execute(),
            Self::Test(args) => test::TestCommand::new(args).execute(),
            Self::Bench(args) => bench::BenchCommand::new(args).execute(),
            Self::Clean(args) => clean::CleanCommand::new(args).execute(),
            Self::Publish(args) => publish::PublishCommand::new(args).execute(),
            Self::Delete(args) => delete::DeleteCommand::new(args).execute(),
            Self::SelfUpdate(args) => self_update::SelfUpdateCommand::new(args).execute(),
            Self::Metadata(args) => metadata::MetadataCommand::new(args).execute(),
            Self::Fmt(args) => fmt::FmtCommand::new(args).execute(),
        }
    }
}
