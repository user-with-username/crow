use anyhow::Result;
use clap::Args;

#[derive(Args)]
pub struct TestArgs {
    /// Build tests in release mode (like `cargo test --release`)
    #[arg(short, long)]
    pub release: bool,

    /// Number of parallel jobs
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,

    /// Name of the specific package to test
    #[arg(long)]
    pub bin: Option<String>,

    /// Arguments to pass to each test executable
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

pub struct TestCommand {
    args: TestArgs,
}

impl TestCommand {
    pub fn new(args: TestArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let profile = if self.args.release { "release" } else { "test" };
        let project = crate::commands::run::build_project(profile, self.args.jobs, self.args.bin)?;
        project.test(&self.args.args)
    }
}