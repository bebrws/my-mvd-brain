use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ViewArgs {
    pub file: PathBuf,
    #[arg(long = "frame-id")]
    pub frame_id: u64,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: ViewArgs) -> Result<()> {
    let mut mem = crate::common::open_memory_ro(&args.file)?;
    let text = mem.frame_text_by_id(args.frame_id)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    if args.json {
        let obj = serde_json::json!({
            "frame_id": args.frame_id,
            "content": text,
            "length": text.len(),
        });
        println!("{}", serde_json::to_string_pretty(&obj)?);
    } else {
        println!("{text}");
    }
    Ok(())
}
