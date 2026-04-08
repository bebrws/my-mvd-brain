use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct WhenArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub limit: Option<u64>,
    #[arg(long)]
    pub reverse: bool,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: WhenArgs) -> Result<()> {
    let mut mem = crate::common::open_memory_ro(&args.file)?;
    let mut builder = memvid_core::TimelineQuery::builder()
        .reverse(args.reverse);
    if let Some(limit) = args.limit {
        if let Some(nz) = std::num::NonZeroU64::new(limit) {
            builder = builder.limit(nz);
        }
    }
    let query = builder.build();
    let entries = mem.timeline(query).map_err(|e| anyhow::anyhow!("{e}"))?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&entries)?);
    } else {
        for entry in &entries {
            println!("#{:<6} {} {}", entry.frame_id, entry.timestamp, entry.preview);
        }
        if entries.is_empty() {
            println!("No frames matched the temporal query.");
        }
    }
    Ok(())
}
