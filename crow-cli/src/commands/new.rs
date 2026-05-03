use anyhow::Result;
use clap::Args;
use crow_utils::status;
use std::fs;
use std::path::PathBuf;
use crate::templates;

#[derive(Args)]
pub struct NewArgs {
    /// Package name
    pub name: String,

    /// Create package in current directory (skip creating new directory)
    #[arg(short = 'n', long)]
    pub no_directory: bool,
}

pub struct NewCommand {
    args: NewArgs,
}

impl NewCommand {
    pub fn new(args: NewArgs) -> Self {
        Self { args }
    }

    pub fn execute(self) -> Result<()> {
        let package_name = &self.args.name;

        if !self.is_valid_package_name(package_name) {
            anyhow::bail!(
                "Invalid package name `{}`. Package names must start with a letter and contain \
                 only alphanumeric characters, hyphens, or underscores.",
                package_name
            );
        }

        let project_dir = if self.args.no_directory {
            std::env::current_dir()?
        } else {
            let dir = PathBuf::from(package_name);
            if dir.exists() {
                anyhow::bail!("Directory `{}` already exists", package_name);
            }
            dir
        };

        if !self.args.no_directory && !project_dir.exists() {
            fs::create_dir_all(&project_dir)?;
        }

        templates::create_project_structure(&project_dir, package_name, true)?;

        if !self.args.no_directory {
            status!("Created", "binary (application) `{}` package", package_name);
        } else {
            status!("Created", "binary (application) `{}` package in current directory", package_name);
        }

        Ok(())
    }

    fn is_valid_package_name(&self, name: &str) -> bool {
        if name.is_empty() {
            return false;
        }

        let first_char = name.chars().next().unwrap();
        if !first_char.is_ascii_alphabetic() {
            return false;
        }

        name.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
    }
}