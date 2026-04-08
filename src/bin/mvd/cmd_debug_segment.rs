use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct DebugSegmentArgs {
    pub file: PathBuf,
    #[arg(long = "segment-id")]
    pub segment_id: u64,
    #[arg(long)]
    pub hex_dump: bool,
}

pub fn run(args: DebugSegmentArgs) -> Result<()> {
    println!("debug-segment: segment_id={} from {}", args.segment_id, args.file.display());
    eprintln!("Note: Direct segment dump not yet exposed in the public API");
    Ok(())
}
