use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use crate::common::WriteOpts;

#[derive(Args)]
pub struct CorrectArgs {
    pub file: PathBuf,
    pub statement: String,
    #[arg(long)]
    pub topics: Option<String>,
    #[arg(long)]
    pub source: Option<String>,
    #[arg(long, default_value = "2.0")]
    pub boost: f64,
    #[arg(long)]
    pub json: bool,
    #[command(flatten)]
    pub write_opts: WriteOpts,
}

pub fn run(args: CorrectArgs) -> Result<()> {
    let mut mem = crate::common::open_memory_rw(&args.file, &args.write_opts)?;
    let mut opts = memvid_core::PutOptions::builder();
    opts = opts.tag("memvid.role", "correction");
    opts = opts.tag("memvid.boost", &args.boost.to_string());
    if let Some(ref source) = args.source {
        opts = opts.tag("memvid.source", source);
    }
    if let Some(ref topics) = args.topics {
        for topic in topics.split(',') {
            opts = opts.label(topic.trim());
        }
    }
    let options = opts.build();
    let frame_id = mem.put_bytes_with_options(args.statement.as_bytes(), options)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    mem.commit().map_err(|e| anyhow::anyhow!("{e}"))?;
    if args.json {
        println!("{{\"frame_id\":{frame_id}}}");
    } else {
        println!("Correction stored as frame {frame_id}");
    }
    Ok(())
}
