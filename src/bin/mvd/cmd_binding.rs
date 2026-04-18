use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct BindingArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: BindingArgs) -> Result<()> {
    let mem = crate::common::open_memory_ro(&args.file)?;
    let stats = mem.stats().map_err(|e| anyhow::anyhow!("{e}"))?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&stats)?);
    } else {
        println!("Binding info for {}", args.file.display());

    }
    Ok(())
}
