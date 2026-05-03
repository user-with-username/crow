use anyhow::Result;
use clap::Args;
use crow_utils::status;
use std::fs;
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
        let src_dir = current_dir.join("src");
        let main_cpp = src_dir.join("main.cpp");

        if !self.args.quiet {
            if config_path.exists() {
                if !self.prompt_overwrite("crow.toml")? {
                    status!("Aborted", "Initialization cancelled");
                    return Ok(());
                }
            }
            
            if main_cpp.exists() && !self.prompt_overwrite("main.cpp")? {
                status!("Aborted", "Initialization cancelled");
                return Ok(());
            }
        }

        // Create project structure
        if !src_dir.exists() {
            fs::create_dir_all(&src_dir)?;
        }
        
        templates::write_main_cpp(&src_dir)?;
        templates::write_crow_toml(&config_path, &package_name, true)?;
        templates::write_gitignore(&current_dir, false)?;

        status!("Initialized", "binary (application) `{}` package", package_name);
        Ok(())
    }

    fn prompt_overwrite(&self, filename: &str) -> Result<bool> {
        use std::io::Write;
        print!("{} already exists. Overwrite? [y/N] ", filename);
        std::io::stdout().flush()?;
        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;
        Ok(input.trim().eq_ignore_ascii_case("y"))
    }
}