use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct TicketsArgs {
    #[command(subcommand)]
    pub command: TicketsCommand,
}

#[derive(Subcommand)]
pub enum TicketsCommand {
    /// List tickets info
    List { file: PathBuf, #[arg(long)] json: bool },
    /// Sync tickets from the API
    Sync { file: PathBuf },
}

pub fn run(args: TicketsArgs) -> Result<()> {
    match args.command {
        TicketsCommand::List { file, json: _ } => {
            let mem = crate::common::open_memory_ro(&file)?;
            let stats = mem.stats().map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Ticket info for {}: Enterprise", file.display());
            Ok(())
        }
        TicketsCommand::Sync { file: _ } => {
            eprintln!("tickets sync: requires API server (not available in open-source mvd).");
            Ok(())
        }
    }
}
