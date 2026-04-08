use anyhow::{Context, Result};
use clap::Args;
use memvid_core::PutOptions;
use std::io::Read;
use std::path::PathBuf;
use crate::common::WriteOpts;

#[derive(Args)]
pub struct PutArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub input: Option<PathBuf>,
    #[arg(long)]
    pub uri: Option<String>,
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long = "tag")]
    pub tags: Vec<String>,
    #[arg(long = "label")]
    pub labels: Vec<String>,
    #[arg(long)]
    pub embedding: bool,
    #[arg(long)]
    pub dedup: bool,
    #[command(flatten)]
    pub write_opts: WriteOpts,
}

pub fn run(args: PutArgs) -> Result<()> {
    let payload = if let Some(ref input_path) = args.input {
        std::fs::read(input_path)
            .with_context(|| format!("Failed to read input file: {}", input_path.display()))?
    } else {
        let mut buf = Vec::new();
        std::io::stdin().read_to_end(&mut buf)
            .context("Failed to read from stdin")?;
        buf
    };

    let mut mem = crate::common::open_memory_rw(&args.file, &args.write_opts)?;

    let mut opts = PutOptions::builder();
    if let Some(ref uri) = args.uri {
        opts = opts.uri(uri);
    }
    if let Some(ref title) = args.title {
        opts = opts.title(title);
    }
    for tag in &args.tags {
        if let Some((key, value)) = tag.split_once('=') {
            opts = opts.tag(key, value);
        }
    }
    for label in &args.labels {
        opts = opts.label(label);
    }

    let options = opts.build();
    let frame_id = mem.put_bytes_with_options(&payload, options)
        .map_err(|e| anyhow::anyhow!("{e}"))
        .context("Failed to put frame")?;
    mem.commit().map_err(|e| anyhow::anyhow!("{e}"))?;

    if args.json {
        println!("{{\"frame_id\":{frame_id}}}");
    } else {
        println!("Frame {frame_id} added to {}", args.file.display());
    }
    Ok(())
}
