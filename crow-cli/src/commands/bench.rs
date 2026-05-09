use anyhow::Result;
use clap::Args;

use super::run::run_with_profile;

#[derive(Args)]
pub struct BenchArgs {
    /// Number of parallel jobs
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,

    /// Name of the specific binary to run
    #[arg(long)]
    pub bin: Option<String>,

    /// Arguments to pass to the executable
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

pub struct BenchCommand {
    args: BenchArgs,
}

impl BenchCommand {
    pub fn new(args: BenchArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        run_with_profile("bench", self.args.jobs, self.args.bin, self.args.args)
    }
}
