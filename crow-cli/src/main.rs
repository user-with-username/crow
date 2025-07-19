use anyhow::Result;
use clap::Parser;
use commands::{Command, Commands};
use crow_utils::logger::LOGGER_INSTANCE;

mod commands;

#[derive(Parser)]
#[command(name = "crow", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    if let Err(e) = cli.command.execute(&LOGGER_INSTANCE) {
        LOGGER_INSTANCE.critical_error(&format!("{:?}", e));
        std::process::exit(1);
    }
    Ok(())
}