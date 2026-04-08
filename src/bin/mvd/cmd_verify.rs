use anyhow::Result;
use clap::Args;
use memvid_core::VerificationStatus;
use std::path::PathBuf;

#[derive(Args)]
pub struct VerifyArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub deep: bool,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: VerifyArgs) -> Result<()> {
    let report = memvid_core::Memvid::verify(&args.file, args.deep)
        .map_err(|e| anyhow::anyhow!("{e}"))?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&report)?);
    } else {
        println!("Verification: {:?}", report.overall_status);
        for check in &report.checks {
            let symbol = match check.status {
                VerificationStatus::Passed => "✓",
                VerificationStatus::Failed => "✗",
                VerificationStatus::Skipped => "○",
            };
            println!("  {symbol} {}: {}", check.name, check.details.as_deref().unwrap_or("ok"));
        }
    }
    Ok(())
}
