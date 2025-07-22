use super::*;
use crow_utils::logger::{LogLevel, Logger};
use crow_utils::Environment;

pub trait ProjectCleaner {
    fn clean_project(&self, clean_all: bool, logger: &Logger) -> anyhow::Result<()>;
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
    fn clean_project(&self, clean_all: bool, logger: &Logger) -> Result<()> {
        let build_dir = Environment::build_dir();

        if build_dir.exists() {
            std::fs::remove_dir_all(&build_dir)?;
            logger.log(
                LogLevel::Success,
                &format!("Cleaned build artifacts in '{}'.", build_dir.display()),
                (),
            );
        } else {
            logger.log(
                LogLevel::Dim,
                &format!("No build artifacts found in '{}'.", build_dir.display()),
                (),
            );
        }

        if clean_all {
            let deps_dir = Environment::deps_dir(false);
            if deps_dir.exists() {
                logger.log(
                    LogLevel::Warn,
                    &format!("Cleaning dependency cache in '{}'...", deps_dir.display()),
                    (),
                );
                std::fs::remove_dir_all(&deps_dir)?;
                logger.log(
                    LogLevel::Success,
                    &format!("Cleaned dependency cache in '{}'.", deps_dir.display()),
                    (),
                );
            } else {
                logger.log(
                    LogLevel::Dim,
                    &format!("No dependency cache found in '{}'.", deps_dir.display()),
                    (),
                );
            }

            let deps_dir_global = Environment::deps_dir(true);
            if deps_dir_global.exists() {
                logger.log(
                    LogLevel::Warn,
                    &format!(
                        "Cleaning global dependency cache in '{}'...",
                        deps_dir_global.display()
                    ),
                    (),
                );
                std::fs::remove_dir_all(&deps_dir_global)?;
                logger.log(
                    LogLevel::Success,
                    &format!(
                        "Cleaned global dependency cache in '{}'.",
                        deps_dir_global.display()
                    ),
                    (),
                );
            } else {
                logger.log(
                    LogLevel::Dim,
                    &format!(
                        "No global dependency cache found in '{}'.",
                        deps_dir_global.display()
                    ),
                    (),
                );
            }
        } else {
            logger.log(
                LogLevel::Dim,
                "Skipping dependency cache clean. Use `crow clean --all` to remove it.",
                (),
            );
        }
        Ok(())
    }
}

impl Command for CleanCommand {
    fn execute(&self, logger: &mut Logger) -> Result<()> {
        logger.quiet(Environment::quiet_mode(self.quiet));
        self.clean_project(self.all, logger)
    }
}
