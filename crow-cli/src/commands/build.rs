use super::*;
use crow_core::Config;
use crow_utils::Environment;
use std::path::PathBuf;

pub trait ProjectBuilder {
    fn build_project(
        &self,
        profile: &str,
        jobs: Option<usize>,
        verbose: bool,
        global_deps: bool,
        logger: &'static Logger,
    ) -> anyhow::Result<std::path::PathBuf>;
}

#[derive(Args)]
pub struct BuildCommand {
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

impl ProjectBuilder for BuildCommand {
    fn build_project(
        &self,
        profile: &str,
        jobs: Option<usize>,
        verbose: bool,
        global_deps: bool,
        logger: &'static Logger,
    ) -> Result<PathBuf> {
        let config = Config::load("crow.toml")?;
        let build_system = crow_core::build_system::BuildSystem::new(
            config,
            profile,
            verbose,
            global_deps,
            logger,
        )?;
        build_system.build(jobs)
    }
}

impl Command for BuildCommand {
    fn execute(&self, logger: &'static Logger) -> Result<()> {
        crow_utils::logger::QUIET_MODE.store(
            Environment::parse_quiet_mode_env(self.quiet),
            std::sync::atomic::Ordering::Relaxed,
        );
        let global_deps = Environment::parse_global_deps_env(self.global_deps);
        self.build_project(&self.profile, self.jobs, self.verbose, global_deps, logger)?;
        Ok(())
    }
}
