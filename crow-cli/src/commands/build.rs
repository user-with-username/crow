use anyhowed::Result;
use clap::Args;
use crow_core::{config::Workspace, Project};
use std::path::PathBuf;

#[derive(Args)]
pub struct BuildArgs {
    #[arg(short, long)]
    pub release: bool,
    #[arg(long)]
    pub target: Option<String>,
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,
    #[arg(long)]
    pub bin: Option<String>,
    #[arg(short = 'p', long, default_value = "debug")]
    pub profile: String,
}

pub struct BuildCommand {
    args: BuildArgs,
}

impl BuildCommand {
    pub fn new(args: BuildArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let workspace = Workspace::load()?;

        let profile_name = if self.args.release {
            "release"
        } else {
            &self.args.profile
        };

        let members_to_build: Vec<_> = if let Some(bin_name) = &self.args.bin {
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

        if members_to_build.is_empty() {
            if self.args.bin.is_some() {
                anyhowed::bail!(
                    "No binary package named `{}` found",
                    self.args.bin.as_ref().unwrap()
                );
            } else {
                anyhowed::bail!("No packages found to build");
            }
        }

        for (member_config, member_root) in members_to_build {
            self.build_package(member_config, member_root, profile_name)?;
        }

        Ok(())
    }

    fn build_package(
        &self,
        _config: crow_core::CrowConfig,
        root: PathBuf,
        profile_name: &str,
    ) -> Result<()> {
        let _project = Project::build(&root, profile_name, self.args.jobs)?;
        Ok(())
    }
}
