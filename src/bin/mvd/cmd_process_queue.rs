use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ProcessQueueArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub status: bool,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: ProcessQueueArgs) -> Result<()> {
    let mem = crate::common::open_memory_ro(&args.file)?;
    let stats = mem.stats().map_err(|e| anyhow::anyhow!("{e}"))?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!("Queue status for {}", args.file.display());
        println!("  Frames: {}", stats.frame_count);
    }
    Ok(())
}
