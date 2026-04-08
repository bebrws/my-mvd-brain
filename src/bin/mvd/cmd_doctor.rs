use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct DoctorArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub rebuild_time_index: bool,
    #[arg(long)]
    pub rebuild_lex_index: bool,
    #[arg(long)]
    pub rebuild_vec_index: bool,
    #[arg(long)]
    pub vacuum: bool,
    #[arg(long)]
    pub dry_run: bool,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: DoctorArgs) -> Result<()> {
    let options = memvid_core::DoctorOptions {
        rebuild_time_index: args.rebuild_time_index,
        rebuild_lex_index: args.rebuild_lex_index,
        rebuild_vec_index: args.rebuild_vec_index,
        vacuum: args.vacuum,
        dry_run: args.dry_run,
        quiet: false,
    };
    let report = memvid_core::Memvid::doctor(&args.file, options)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("Doctor Report: {:?}", report.status);
        for finding in &report.findings {
            println!("  [{:?}] {:?}: {}", finding.severity, finding.code, finding.message);
        }
        for phase in &report.phases {
            println!("  Phase {:?}: {:?} ({} ms)", phase.phase, phase.status,
                phase.duration_ms.unwrap_or(0));
        }
    }
    Ok(())
}
