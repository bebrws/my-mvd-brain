use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct ExportArgs {
    pub file: PathBuf,
    #[arg(long, default_value = "json")]
    pub format: String,
    #[arg(long)]
    pub out: Option<PathBuf>,
}

pub fn run(args: ExportArgs) -> Result<()> {
    println!("Exporting from {} in {} format", args.file.display(), args.format);
    if let Some(ref out) = args.out { println!("  Output: {}", out.display()); }
    Ok(())
}
