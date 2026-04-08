use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct PlanArgs {
    #[command(subcommand)]
    pub command: PlanCommand,
}

#[derive(Subcommand)]
pub enum PlanCommand {
    /// Show current plan/tier
    Show { file: PathBuf, #[arg(long)] json: bool },
    /// Sync plan from API
    Sync { file: PathBuf },
}

pub fn run(args: PlanArgs) -> Result<()> {
    match args.command {
        PlanCommand::Show { file, json: _ } => {
            let mem = crate::common::open_memory_ro(&file)?;
            let stats = mem.stats().map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Plan for {}: {:?}", file.display(), stats.tier);
            println!("  Capacity: {}", crate::common::format_bytes(stats.capacity_bytes));
            Ok(())
        }
        PlanCommand::Sync { file: _ } => {
            eprintln!("plan sync: requires API server (not available in open-source mvd).");
            Ok(())
        }
    }
}
