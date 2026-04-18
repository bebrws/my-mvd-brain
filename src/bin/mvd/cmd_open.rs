use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct OpenArgs {
    /// Path to the memory file to open
    pub file: PathBuf,
    /// Emit JSON instead of human-readable output
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: OpenArgs) -> Result<()> {
    let mem = crate::common::open_memory_ro(&args.file)?;
    let stats = mem.stats().map_err(|e| anyhow::anyhow!("{e}"))?;

    if args.json {
        let json = serde_json::to_string_pretty(&stats).context("Failed to serialize stats")?;
        println!("{json}");
    } else {
        println!("Memory: {}", args.file.display());
        println!("  Frames:     {}", stats.frame_count);
        println!("  Active:     {}", stats.active_frame_count);
        println!("  Size:       {}", crate::common::format_bytes(stats.size_bytes));

        println!("  Lex index:  {}", stats.has_lex_index);
        println!("  Vec index:  {}", stats.has_vec_index);
        println!("  Time index: {}", stats.has_time_index);
        println!("  CLIP index: {}", stats.has_clip_index);
    }
    Ok(())
}
