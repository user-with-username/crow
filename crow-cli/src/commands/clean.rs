use super::*;
use crow_utils::Environment;

pub trait ProjectCleaner {
    fn clean_project(&self, clean_all: bool, logger: &'static Logger) -> anyhow::Result<()>;
}

#[derive(Args)]
pub struct CleanCommand {
    /// Also clean dependencies cache
    #[arg(long)]
    pub all: bool,
    /// Suppress output
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,
}

impl ProjectCleaner for CleanCommand {
    fn clean_project(&self, clean_all: bool, logger: &'static Logger) -> Result<()> {
        let build_dir = Environment::build_dir();

        if build_dir.exists() {
            std::fs::remove_dir_all(&build_dir)?;
            logger.success(&format!(
                "Cleaned build artifacts in '{}'.",
                build_dir.display()
            ));
        } else {
            logger.dim(&format!(
                "No build artifacts found in '{}'.",
                build_dir.display()
            ));
        }

        if clean_all {
            let deps_dir = Environment::deps_dir(false);
            if deps_dir.exists() {
                logger.warn(&format!(
                    "Cleaning dependency cache in '{}'...",
                    deps_dir.display()
                ));
                std::fs::remove_dir_all(&deps_dir)?;
                logger.success(&format!(
                    "Cleaned dependency cache in '{}'.",
                    deps_dir.display()
                ));
            } else {
                logger.dim(&format!(
                    "No dependency cache found in '{}'.",
                    deps_dir.display()
                ));
            }

            let deps_dir_global = Environment::deps_dir(true);
            if deps_dir_global.exists() {
                logger.warn(&format!(
                    "Cleaning global dependency cache in '{}'...",
                    deps_dir_global.display()
                ));
                std::fs::remove_dir_all(&deps_dir_global)?;
                logger.success(&format!(
                    "Cleaned global dependency cache in '{}'.",
                    deps_dir_global.display()
                ));
            } else {
                logger.dim(&format!(
                    "No global dependency cache found in '{}'.",
                    deps_dir_global.display()
                ));
            }
        } else {
            logger.dim("Skipping dependency cache clean. Use `crow clean --all` to remove it.");
        }
        Ok(())
    }
}

impl Command for CleanCommand {
    fn execute(&self, logger: &'static Logger) -> Result<()> {
        crow_utils::logger::QUIET_MODE.store(
            Environment::parse_quiet_mode_env(self.quiet),
            std::sync::atomic::Ordering::Relaxed,
        );
        self.clean_project(self.all, logger)
    }
}
