use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use crate::common::WriteOpts;

#[derive(Args)]
pub struct DeleteArgs {
    pub file: PathBuf,
    #[arg(long = "frame-id")]
    pub frame_id: u64,
    #[arg(long)]
    pub json: bool,
    #[command(flatten)]
    pub write_opts: WriteOpts,
}

pub fn run(args: DeleteArgs) -> Result<()> {
    let mut mem = crate::common::open_memory_rw(&args.file, &args.write_opts)?;
    mem.delete_frame(args.frame_id).map_err(|e| anyhow::anyhow!("{e}"))?;
    mem.commit().map_err(|e| anyhow::anyhow!("{e}"))?;
    if args.json {
        println!("{{\"deleted\":true,\"frame_id\":{}}}", args.frame_id);
    } else {
        println!("Frame {} deleted", args.frame_id);
    }
    Ok(())
}
