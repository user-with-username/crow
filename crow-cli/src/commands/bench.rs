use anyhowed::Result;
use clap::Args;

#[derive(Args)]
pub struct BenchArgs {
    /// Build benchmarks in release mode (like `cargo bench --release`)
    #[arg(short, long)]
    pub release: bool,

    /// Number of parallel jobs
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,

    /// Name of the specific package to benchmark
    #[arg(long)]
    pub bin: Option<String>,

    /// Named target or conditional target (e.g., client, "cfg(windows)")
    #[arg(long)]
    pub target: Option<String>,

    /// Arguments to pass to each test executable
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
        let profile = if self.args.release {
            "release"
        } else {
            "bench"
        };
        let project = crate::commands::run::build_project(profile, self.args.jobs, self.args.bin, self.args.target.as_deref())?;
        project.bench(&self.args.args)
    }
}