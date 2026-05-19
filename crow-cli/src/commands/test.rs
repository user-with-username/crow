use anyhow::{Context, Result};
use clap::Args;
use crow_core::{config::Workspace, Project};

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

        let workspace = Workspace::load()?;
        let binary_members = workspace.binary_members();

        if binary_members.is_empty() {
            anyhow::bail!("No binary packages found to test");
        }

        let (_, selected_root) = match &self.args.bin {
            Some(name) => workspace
                .find_member_by_name(name)
                .map(|(cfg, root)| (cfg.clone(), root.clone()))
                .with_context(|| format!("No binary package named `{name}` found"))?,
            None => {
                if binary_members.len() == 1 {
                    let (cfg, root) = binary_members[0];
                    (cfg.clone(), root.clone())
                } else {
                    let names: Vec<String> = binary_members
                        .iter()
                        .filter_map(|(cfg, _)| cfg.package.as_ref().map(|p| p.name.clone()))
                        .collect();
                    anyhow::bail!(
                        "Multiple binary packages available. Use `--bin` to specify one.\nAvailable binaries: {}",
                        names.join(", ")
                    );
                }
            }
        };

        let project = Project::build(&selected_root, profile, self.args.jobs)?;
        project.test(&self.args.args)
    }
}
