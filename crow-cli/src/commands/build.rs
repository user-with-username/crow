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
        target: Option<&str>,
        logger: &Logger,
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

    /// Build specific target (binary or library name)
    #[arg(long)]
    pub target: Option<String>,
}

impl ProjectBuilder for BuildCommand {
    fn build_project(
        &self,
        profile: &str,
        jobs: Option<usize>,
        verbose: bool,
        global_deps: bool,
        target: Option<&str>,
        logger: &Logger,
    ) -> anyhow::Result<PathBuf> {
        let mut logger = logger.clone();
        logger.verbose(verbose);

        let config = Config::load("crow.toml")?;
        let build_system = crow_core::build_system::BuildSystem::new(
            config,
            profile,
            global_deps,
            logger.clone(),
        )?;

        if let Some(t) = target {
            build_system.build_target(t, jobs)
        } else {
            build_system.build(jobs)
        }
    }
}

impl Command for BuildCommand {
    fn execute(&self, logger: &mut Logger) -> anyhow::Result<()> {
        logger.quiet(Environment::quiet_mode(self.quiet));

        let global_deps = Environment::global_deps(self.global_deps);
        self.build_project(
            &self.profile,
            self.jobs,
            self.verbose,
            global_deps,
            self.target.as_deref(),
            logger,
        )?;
        Ok(())
    }
}
