use anyhow::{Context, Result};
use clap::Args;
use memvid_core::Memvid;
use std::path::PathBuf;

#[derive(Args)]
pub struct CreateArgs {
    /// Path to the memory file to create
    pub file: PathBuf,
}

pub fn run(args: CreateArgs) -> Result<()> {
    let mem = Memvid::create(&args.file)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .with_context(|| format!("Failed to create memory: {}", args.file.display()))?;

    let stats = mem.stats().map_err(|e| anyhow::anyhow!("{e}"))?;
    println!("Created memory: {}", args.file.display());
    println!("  Frames:    {}", stats.frame_count);
    println!("  Size:      {}", crate::common::format_bytes(stats.size_bytes));
    println!("  Lex index: {}", stats.has_lex_index);
    println!("  Vec index: {}", stats.has_vec_index);
    Ok(())
}
