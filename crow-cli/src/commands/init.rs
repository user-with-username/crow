use super::*;
use anyhow::Context;
use crow_utils::logger::{LogLevel, Logger};
use crow_utils::Environment;
use std::io::Write;
use tera::Tera;

pub trait ProjectInitializer {
    fn init_project(&self, name: &str, logger: &mut Logger) -> anyhow::Result<()>;
}

#[derive(Args)]
pub struct InitCommand {
    /// Name of the project to create
    pub name: String,
    /// Suppress output
    #[arg(short, long, default_value_t = false)]
    pub quiet: bool,
}

impl ProjectInitializer for InitCommand {
    fn init_project(&self, name: &str, logger: &mut Logger) -> Result<()> {
        let project_dir = std::path::PathBuf::from(name);

        if project_dir.exists() {
            logger.log(
                LogLevel::Warn,
                &format!("Destination '{}' already exists.", name),
                1,
            );
            if !logger.quiet {
                print!("Do you want to overwrite it? (y/N): ");
                std::io::stdout().flush()?;
            }

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.eq_ignore_ascii_case("y") {
                std::fs::remove_dir_all(&project_dir)?;
            } else {
                logger.log(LogLevel::Dim, "Aborting project initialization", ());
                return Ok(());
            }
        }

        std::fs::create_dir(&project_dir)?;
        let src_dir = project_dir.join("src");
        std::fs::create_dir(&src_dir)?;

        let mut tera = Tera::default();
        tera.add_raw_template(
            "main.cpp.tera",
            include_str!("../../templates/main.cpp.tera"),
        )
        .with_context(|| "No main.cpp template")?;
        tera.add_raw_template(
            "crow.toml.tera",
            include_str!("../../templates/crow.toml.tera"),
        )
        .with_context(|| "No config template")?;

        let mut context = tera::Context::new();
        context.insert("project_name", name);

        let main_cpp = tera
            .render("main.cpp.tera", &context)
            .context("Oops no templates")?;
        let crow_toml = tera
            .render("crow.toml.tera", &context)
            .context("No config template? oops, download from sources")?;

        std::fs::write(src_dir.join("main.cpp"), main_cpp)?;
        std::fs::write(project_dir.join("crow.toml"), crow_toml)?;

        logger.log(
            LogLevel::Success,
            &format!("Created new package `{}`", name),
            1,
        );
        Ok(())
    }
}

impl Command for InitCommand {
    fn execute(&self, logger: &mut Logger) -> Result<()> {
        logger.quiet(Environment::quiet_mode(self.quiet));
        self.init_project(&self.name, logger)
    }
}
