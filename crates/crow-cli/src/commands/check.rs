use anyhowed::Result;
use clap::Args;
use crow_core::{config::Workspace, Project};

#[derive(Args)]
pub struct CheckArgs {
    #[arg(short, long)]
    pub release: bool,
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,
    #[arg(long)]
    pub bin: Option<String>,
    #[arg(short = 'p', long, default_value = "debug")]
    pub profile: String,
    /// Named target or conditional target (e.g., client, "cfg(windows)")
    #[arg(long)]
    pub target: Option<String>,
}

pub struct CheckCommand {
    args: CheckArgs,
}

impl CheckCommand {
    pub fn new(args: CheckArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let workspace = Workspace::load_with_target(self.args.target.as_deref())?;

        let profile_name = if self.args.release {
            "release"
        } else {
            &self.args.profile
        };

        let members_to_check: Vec<_> = if let Some(bin_name) = &self.args.bin {
            workspace
                .binary_members()
                .into_iter()
                .filter(|(cfg, _)| cfg.package.as_ref().map(|p| &p.name) == Some(bin_name))
                .map(|(cfg, root)| (cfg.clone(), root.clone()))
                .collect()
        } else {
            workspace
                .members()
                .map(|(cfg, root)| (cfg.clone(), root.clone()))
                .collect()
        };

        if members_to_check.is_empty() {
            if let Some(bin) = &self.args.bin {
                anyhowed::bail!("No binary package named `{}` found", bin);
            } else {
                anyhowed::bail!("No packages found to check");
            }
        }

        for (_config, root) in members_to_check {
            Project::build(
                &root,
                profile_name,
                self.args.jobs,
                true,
                self.args.target.as_deref(),
            )?;
        }

        Ok(())
    }
}
