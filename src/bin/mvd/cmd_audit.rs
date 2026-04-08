use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct AuditArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub out: Option<PathBuf>,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: AuditArgs) -> Result<()> {
    let mem = crate::common::open_memory_ro(&args.file)?;
    let stats = mem.stats().map_err(|e| anyhow::anyhow!("{e}"))?;
    let report = format!(
        "Audit Report for {}\n\
         Frames: {}\n\
         Active: {}\n\
         Size: {}\n\
         Lex index: {}\n\
         Vec index: {}\n\
         Time index: {}",
        args.file.display(),
        stats.frame_count,
        stats.active_frame_count,
        crate::common::format_bytes(stats.size_bytes),
        stats.has_lex_index,
        stats.has_vec_index,
        stats.has_time_index,
    );
    if let Some(ref out_path) = args.out {
        std::fs::write(out_path, &report)?;
        println!("Audit report written to {}", out_path.display());
    } else {
        println!("{report}");
    }
    Ok(())
}
