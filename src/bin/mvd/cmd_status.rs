use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct StatusArgs {
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub dir: Option<PathBuf>,
}

pub fn run(args: StatusArgs) -> Result<()> {
    let search_dir = args.dir.unwrap_or_else(|| std::env::current_dir().unwrap_or_default());
    let mut mv2_files = Vec::new();
    if search_dir.is_dir() {
        for entry in std::fs::read_dir(&search_dir)? {
            let entry = entry?;
            if entry.path().extension().map_or(false, |ext| ext == "mv2") {
                mv2_files.push(entry.path());
            }
        }
    }
    if args.json {
        let status = serde_json::json!({ "directory": search_dir.display().to_string(), "mv2_files": mv2_files.len() });
        println!("{}", serde_json::to_string_pretty(&status)?);
    } else {
        println!("mvd status");
        println!("  Directory: {}", search_dir.display());
        println!("  MV2 files: {}", mv2_files.len());
        for f in &mv2_files { println!("    {}", f.display()); }
    }
    Ok(())
}
