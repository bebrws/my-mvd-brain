use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct WhoArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: WhoArgs) -> Result<()> {
    let _mem = crate::common::open_memory_ro(&args.file)?;
    println!("Lock info for {}", args.file.display());
    Ok(())
}
