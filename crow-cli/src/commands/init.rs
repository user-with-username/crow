use anyhow::Result;
use clap::Args;
use crow_utils::status;
use dialoguer::console::style;
use dialoguer::{theme::ColorfulTheme, Confirm};

use crate::templates;

#[derive(Args)]
pub struct InitArgs {
    /// Initialize in current directory without asking
    #[arg(short, long)]
    pub quiet: bool,
}

pub struct InitCommand {
    args: InitArgs,
}

impl InitCommand {
    pub fn new(args: InitArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let current_dir = std::env::current_dir()?;
        let package_name = current_dir
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("project")
            .to_string();

        let config_path = current_dir.join("crow.toml");
        let main_cpp = current_dir.join("src/main.cpp");

        let overwrite = if !self.args.quiet && (config_path.exists() || main_cpp.exists()) {
            println!("{}", style("Looks like the project there is already exists").yellow());
            
            let confirmed = Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Overwrite existing files?")
                .default(false)
                .wait_for_newline(false)
                .interact_opt()?;
            
            print!("\x1B[2A\x1B[2K\r\x1B[2K\r");
            
            match confirmed {
                Some(true) => true,
                _ => {
                    status!("Aborted", "Initialization cancelled");
                    return Ok(());
                }
            }
        } else {
            true
        };

        templates::create_project_structure(&current_dir, &package_name, overwrite)?;

        status!(
            "Initialized",
            "binary (application) `{}` package",
            package_name
        );
        Ok(())
    }
}