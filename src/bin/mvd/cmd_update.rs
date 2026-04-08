use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use crate::common::WriteOpts;

#[derive(Args)]
pub struct UpdateArgs {
    pub file: PathBuf,
    #[arg(long = "frame-id")]
    pub frame_id: u64,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub json: bool,
    #[command(flatten)]
    pub write_opts: WriteOpts,
}

pub fn run(args: UpdateArgs) -> Result<()> {
    let mut mem = crate::common::open_memory_rw(&args.file, &args.write_opts)?;

    let payload = if let Some(ref input_path) = args.input {
        Some(std::fs::read(input_path)
            .with_context(|| format!("Failed to read input: {}", input_path.display()))?)
    } else {
        None
    };

    let options = memvid_core::PutOptions::builder().build();
    mem.update_frame(args.frame_id, payload, options, None)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    mem.commit().map_err(|e| anyhow::anyhow!("{e}"))?;

    if args.json {
        println!("{{\"updated\":true,\"frame_id\":{}}}", args.frame_id);
    } else {
        println!("Frame {} updated", args.frame_id);
    }
    Ok(())
}
