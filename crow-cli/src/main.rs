use clap::Parser;
use crow_utils::helpers::*;
use std::path::PathBuf;

mod commands;
mod templates;
use commands::Command;

#[derive(Parser)]
#[command(name = "crow")]
#[command(about = "A C/C++ build tool inspired by Cargo", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    command: Command,

    #[arg(short = 'C', long, global = true)]
    pub directory: Option<PathBuf>,

    #[arg(short, long, global = true)]
    pub quiet: bool,
}

impl Cli {
    pub fn execute(self) -> anyhowed::Result<()> {
        if let Some(dir) = &self.directory {
            change_directory(dir)?;
        }

        self.command.execute()
    }
}

fn main() -> anyhowed::Result<()> {
    let cli = Cli::parse();
    cli.execute()
}
