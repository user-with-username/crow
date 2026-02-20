use anyhow::Result;
use clap::Args;
use crow_core::{CrowConfig, Project};
use crow_utils::status;
use std::process::Command;

#[derive(Args)]
pub struct RunArgs {
    /// Build in release mode
    #[arg(short, long)]
    pub release: bool,

    /// Build profile (debug, release, test, bench)
    #[arg(short = 'p', long, default_value = "debug")]
    pub profile: String,

    /// Arguments to pass to the executable
    #[arg(trailing_var_arg = true, allow_hyphen_values = true)]
    pub args: Vec<String>,
}

pub struct RunCommand {
    args: RunArgs,
}

impl RunCommand {
    pub fn new(args: RunArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let profile_name: &str = if self.args.release {
            "release"
        } else {
            &self.args.profile
        };

        let build_args = crate::commands::build::BuildArgs {
            release: self.args.release,
            target: None,
            jobs: None,
            bin: None,
            profile: self.args.profile.clone(),
        };

        crate::commands::build::BuildCommand::new(build_args).execute()?;

        self.run_executable(profile_name)
    }

    fn run_executable(&self, profile_name: &str) -> Result<()> {
        let config = CrowConfig::load()?;
        let project = Project::new(config, profile_name)?;
        let executable = project.output_path();

        if !std::path::Path::new(&executable).exists() {
            anyhow::bail!(
                "executable '{}' not found after build",
                executable.display()
            );
        }

        status!("Running", "`{}`", executable.display());

        let mut cmd = Command::new(format!("{}", executable.display()));

        if !self.args.args.is_empty() {
            cmd.args(&self.args.args);
        }

        let status = cmd
            .status()
            .map_err(|e| anyhow::anyhow!("failed to execute '{}': {}", executable.display(), e))?;

        if !status.success() {
            std::process::exit(status.code().unwrap_or(1));
        }

        Ok(())
    }
}
