use super::*;
use crate::commands::build::ProjectBuilder;
use crow_core::Config;
use crow_utils::{Environment, logger::{Logger, LogLevel}};

pub trait ProjectRunner {
    fn run_project(
        &self,
        profile: &str,
        no_build: bool,
        jobs: Option<usize>,
        verbose: bool,
        global_deps: bool,
        logger: &Logger,
    ) -> anyhow::Result<()>;
}

#[derive(Args)]
pub struct RunCommand {
    /// Skip build step and run existing executable
    #[arg(long, short = 'n')]
    pub no_build: bool,
    /// Build profile to use
    #[arg(long, default_value = "debug")]
    pub profile: String,
    /// Number of parallel jobs
    #[arg(short, long)]
    pub jobs: Option<usize>,
    /// Enable verbose output
    #[arg(short, long)]
    pub verbose: bool,
    /// Use global dependencies cache
    #[arg(long, default_value_t = false)]
    pub global_deps: bool,
    /// Suppress output
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,
}

impl ProjectRunner for RunCommand {
    fn run_project(
        &self,
        profile: &str,
        no_build: bool,
        jobs: Option<usize>,
        verbose: bool,
        global_deps: bool,
        logger: &Logger,
    ) -> Result<()> {
        let exe_path = if !no_build {
            BuildCommand {
                profile: profile.to_string(),
                jobs,
                verbose,
                global_deps,
                quiet: self.quiet,
            }
            .build_project(profile, jobs, verbose, global_deps, logger)?
        } else {
            let config = Config::load("crow.toml")?;
            let (package_config, _, _) = crow_core::build_system::BuildSystem::resolve_config(
                &config, profile, false, logger.clone(),
            )?;
            let exe_name = package_config.name;
            let path = Environment::build_dir().join(profile).join(&exe_name);
            if !path.exists() {
                anyhow::bail!("Executable not found at '{}'. Run `crow build --profile {}` first or remove --no-build.", path.display(), profile);
            }
            path
        };

        logger.log(LogLevel::Success, format!(
            "Running `{}` (profile: {})",
            exe_path.display(),
            profile
        ), 1);
        std::process::Command::new(&exe_path).status()?;
        Ok(())
    }
}

impl Command for RunCommand {
    fn execute(&self, logger: &mut Logger) -> Result<()> {
        logger.quiet(Environment::quiet_mode(self.quiet));
        let global_deps = Environment::global_deps(self.global_deps);
        self.run_project(
            &self.profile,
            self.no_build,
            self.jobs,
            self.verbose,
            global_deps,
            logger,
        )
    }
}