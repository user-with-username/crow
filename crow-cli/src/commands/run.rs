use anyhowed::{Context, Result};
use clap::Args;
use crow_core::{config::Workspace, Project};
use crow_utils::status;
use std::process::Command;

#[derive(Args)]
pub struct RunArgs {
    /// Build in release mode
    #[arg(short, long)]
    pub release: bool,

    /// Number of parallel jobs
    #[arg(short = 'j', long)]
    pub jobs: Option<usize>,

    /// Build profile (debug, release, test, bench)
    #[arg(short = 'p', long, default_value = "debug")]
    pub profile: String,

    /// Name of the specific binary to run
    #[arg(long)]
    pub bin: Option<String>,

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
        let profile_name = if self.args.release {
            "release".to_owned()
        } else {
            self.args.profile.clone()
        };

        let project = build_project(&profile_name, self.args.jobs, self.args.bin)?;
        execute_project_binary(project, &self.args.args)
    }
}

pub(crate) fn build_project(
    profile_name: &str,
    jobs: Option<usize>,
    bin: Option<String>,
) -> Result<Project> {
    let workspace = Workspace::load()?;
    let binary_members = workspace.binary_members();

    if binary_members.is_empty() {
        anyhowed::bail!("No binary packages found");
    }

    let (_, selected_root) = match &bin {
        Some(name) => workspace
            .find_member_by_name(name)
            .map(|(cfg, root)| (cfg.clone(), root.clone()))
            .with_context(|| format!("No binary package named `{}` found", name))?,
        None => {
            if binary_members.len() == 1 {
                let (cfg, root) = binary_members[0];
                (cfg.clone(), root.clone())
            } else {
                let names: Vec<String> = binary_members
                    .iter()
                    .filter_map(|(cfg, _)| cfg.package.as_ref().map(|p| p.name.clone()))
                    .collect();
                anyhowed::bail!(
                    "Multiple binary packages available. Use `--bin` to specify one.\nAvailable binaries: {}",
                    names.join(", ")
                );
            }
        }
    };

    Project::build(&selected_root, profile_name, jobs, false)
}

fn execute_project_binary(project: Project, trailing: &[String]) -> Result<()> {
    let executable = project.output_path();

    if !executable.exists() {
        anyhowed::bail!(
            "Executable '{}' not found. Did the build succeed?",
            executable.display()
        );
    }

    status!("Running", "`{}`", executable.display());

    let mut cmd = Command::new(executable);
    if !trailing.is_empty() {
        cmd.args(trailing);
    }

    let status = cmd.status().with_context(|| "Failed to start executable")?;

    if !status.success() {
        anyhowed::bail!(
            "Process exited with non-zero status (code: {})",
            status.code().unwrap_or(-1)
        );
    }

    Ok(())
}
