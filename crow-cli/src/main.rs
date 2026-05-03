use clap::Parser;
use crow_utils::{helpers::*, warning};
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

    #[arg(long, global = true)]
    pub frozen: bool,

    #[arg(long, global = true)]
    pub locked: bool,

    #[arg(short, long, global = true)]
    pub quiet: bool,
}

impl Cli {
    pub fn execute(self) -> anyhow::Result<()> {
        if let Some(dir) = &self.directory {
            change_directory(dir)?;
        }

        if self.frozen || self.locked {
            warning!("`--frozen` and `--locked` are not supported yet");
        }

        self.command.execute()
    }
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    cli.execute()
}
