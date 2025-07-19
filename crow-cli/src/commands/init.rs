use super::*;
use anyhow::Context;
use std::io::Write;
use tera::Tera;

pub trait ProjectInitializer {
    fn init_project(&self, name: &str, logger: &'static Logger) -> anyhow::Result<()>;
}

#[derive(Args)]
pub struct InitCommand {
    /// Name of the project to create
    pub name: String,
}

impl ProjectInitializer for InitCommand {
    fn init_project(&self, name: &str, logger: &'static Logger) -> Result<()> {
        let project_dir = std::path::PathBuf::from(name);

        if project_dir.exists() {
            logger.warn(&format!("Destination '{}' already exists.", name));
            if !crow_utils::logger::QUIET_MODE.load(std::sync::atomic::Ordering::Relaxed) {
                print!("Do you want to overwrite it? (y/N): ");
                std::io::stdout().flush()?;
            }

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;
            let input = input.trim();

            if input.eq_ignore_ascii_case("y") {
                logger.warn(&format!("Overwriting existing directory '{}'...", name));
                std::fs::remove_dir_all(&project_dir)?;
            } else {
                logger.dim("Aborting project initialization");
                return Ok(());
            }
        }

        std::fs::create_dir(&project_dir)?;
        let src_dir = project_dir.join("src");
        std::fs::create_dir(&src_dir)?;

        let mut tera = Tera::default();
        tera.add_raw_template("main.cpp.tera", include_str!("../../templates/main.cpp.tera"))
            .with_context(|| "No main.cpp template")?;
        tera.add_raw_template("crow.toml.tera", include_str!("../../templates/crow.toml.tera"))
            .with_context(|| "No config template")?;

        let mut context = tera::Context::new();
        context.insert("project_name", name);

        let rendered_main_cpp = tera
            .render("main.cpp.tera", &context)
            .context("Oops no templates")?;
        let rendered_crow_toml = tera
            .render("crow.toml.tera", &context)
            .context("No config template? oops, download from sources")?;

        std::fs::write(src_dir.join("main.cpp"), rendered_main_cpp)?;
        std::fs::write(project_dir.join("crow.toml"), rendered_crow_toml)?;

        logger.success(&format!("Created new package `{}`", name));
        Ok(())
    }
}

impl Command for InitCommand {
    fn execute(&self, logger: &'static Logger) -> Result<()> {
        self.init_project(&self.name, logger)
    }
}
