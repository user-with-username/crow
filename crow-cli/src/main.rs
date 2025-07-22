use anyhow::Result;
use clap::Parser;
use commands::{Command, Commands};
use crow_utils::logger::{LogLevel, Logger};

mod commands;

#[derive(Parser)]
#[command(name = "crow", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut logger = Logger::new();

    if let Err(e) = cli.command.execute(&mut logger) {
        logger.log(LogLevel::Error, &format!("{:?}", e), 0);
        std::process::exit(1);
    }
    Ok(())
}
