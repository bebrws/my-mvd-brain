use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct NudgeArgs {
    pub file: PathBuf,
}

pub fn run(args: NudgeArgs) -> Result<()> {
    println!("Nudge sent to writer of {}", args.file.display());
    Ok(())
}
