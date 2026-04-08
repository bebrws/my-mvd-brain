use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct StateArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub entity: Option<String>,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: StateArgs) -> Result<()> {
    println!("Entity state for {}", args.file.display());
    if let Some(ref entity) = args.entity { println!("  Entity: {entity}"); }
    Ok(())
}
