use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct StatsArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: StatsArgs) -> Result<()> {
    let mem = crate::common::open_memory_ro(&args.file)?;
    let stats = mem.stats().map_err(|e| anyhow::anyhow!("{e}"))?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!("Memory: {}", args.file.display());
        println!("  Total frames:       {}", stats.frame_count);
        println!("  Active frames:      {}", stats.active_frame_count);
        println!("  File size:          {}", crate::common::format_bytes(stats.size_bytes));
        println!("  Payload bytes:      {}", crate::common::format_bytes(stats.payload_bytes));
        println!("  Logical bytes:      {}", crate::common::format_bytes(stats.logical_bytes));
        println!("  Compression ratio:  {:.1}%", stats.compression_ratio_percent);
        println!("  Space savings:      {:.1}%", stats.savings_percent);
        println!("  Capacity:           {}", crate::common::format_bytes(stats.capacity_bytes));
        println!("  Remaining capacity: {}", crate::common::format_bytes(stats.remaining_capacity_bytes));
        println!("  Utilisation:        {:.1}%", stats.storage_utilisation_percent);

        println!("  Lex index:          {}", stats.has_lex_index);
        println!("  Vec index:          {}", stats.has_vec_index);
        println!("  Vectors:            {}", stats.vector_count);
        println!("  CLIP images:        {}", stats.clip_image_count);
        println!("  Time index:         {}", stats.has_time_index);
        println!("  WAL bytes:          {}", crate::common::format_bytes(stats.wal_bytes));
    }
    Ok(())
}
