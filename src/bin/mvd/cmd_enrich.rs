use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::common::WriteOpts;

#[derive(Args)]
pub struct EnrichArgs {
    pub file: PathBuf,
    #[arg(long, default_value = "rules")]
    pub engine: String,
    #[arg(long)]
    pub json: bool,
    #[command(flatten)]
    pub write_opts: WriteOpts,
}

pub fn run(args: EnrichArgs) -> Result<()> {
    let mem = crate::common::open_memory_rw(&args.file, &args.write_opts)?;
    let mem = Arc::new(Mutex::new(mem));
    println!("Enriching {} with engine: {}", args.file.display(), args.engine);
    let handle = memvid_core::start_enrichment_worker(Arc::clone(&mem), None);
    let stats = handle.stop_and_wait();
    // Commit after enrichment
    if let Ok(mut m) = mem.lock() {
        m.commit().map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    if args.json {
        println!("{{\"frames_processed\":{},\"errors\":{},\"embeddings_generated\":{}}}",
            stats.frames_processed, stats.errors, stats.embeddings_generated);
    } else {
        println!("Enrichment complete.");
        println!("  Frames processed: {}", stats.frames_processed);
        println!("  Errors:           {}", stats.errors);
        println!("  Embeddings:       {}", stats.embeddings_generated);
    }
    Ok(())
}
