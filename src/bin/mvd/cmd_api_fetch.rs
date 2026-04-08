use anyhow::{Context, Result};
use clap::Args;
use std::path::PathBuf;
use crate::common::WriteOpts;

#[derive(Args)]
pub struct ApiFetchArgs {
    pub file: PathBuf,
    pub config: PathBuf,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub json: bool,
    #[command(flatten)]
    pub write_opts: WriteOpts,
}

pub fn run(args: ApiFetchArgs) -> Result<()> {
    let config_text = std::fs::read_to_string(&args.config)
        .with_context(|| format!("Failed to read config: {}", args.config.display()))?;
    let config: serde_json::Value = serde_json::from_str(&config_text)
        .context("Failed to parse fetch config JSON")?;

    if args.dry_run {
        println!("Dry run — would fetch and ingest from config: {}", args.config.display());
        println!("Config: {}", serde_json::to_string_pretty(&config)?);
        return Ok(());
    }

    eprintln!("api-fetch: Remote content fetching requires network access.");
    eprintln!("This command reads a fetch configuration and ingests the results.");
    Ok(())
}
