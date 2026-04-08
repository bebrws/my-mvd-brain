use anyhow::{Context, Result};
use clap::Args;
use std::io::Read;
use std::path::PathBuf;
use crate::common::WriteOpts;

#[derive(Args)]
pub struct PutManyArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[command(flatten)]
    pub write_opts: WriteOpts,
}

pub fn run(args: PutManyArgs) -> Result<()> {
    let input_data = if let Some(ref path) = args.input {
        std::fs::read_to_string(path)
            .with_context(|| format!("Failed to read input file: {}", path.display()))?
    } else {
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf).context("Failed to read stdin")?;
        buf
    };

    let items: Vec<serde_json::Value> = serde_json::from_str(&input_data)
        .context("Failed to parse batch JSON input")?;

    let mut mem = crate::common::open_memory_rw(&args.file, &args.write_opts)?;
    let mut inserted = 0u64;

    for item in &items {
        let text = item.get("text").and_then(|v| v.as_str()).unwrap_or("");
        let mut opts = memvid_core::PutOptions::builder();
        if let Some(uri) = item.get("uri").and_then(|v| v.as_str()) {
            opts = opts.uri(uri);
        }
        if let Some(title) = item.get("title").and_then(|v| v.as_str()) {
            opts = opts.title(title);
        }
        let options = opts.build();
        mem.put_bytes_with_options(text.as_bytes(), options)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .with_context(|| format!("Failed to put item {inserted}"))?;
        inserted += 1;
    }

    mem.commit().map_err(|e| anyhow::anyhow!("{e}"))?;

    if args.json {
        println!("{{\"inserted\":{inserted}}}");
    } else {
        println!("Inserted {inserted} frames into {}", args.file.display());
    }
    Ok(())
}
