use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct VerifySingleFileArgs {
    pub file: PathBuf,
}

pub fn run(args: VerifySingleFileArgs) -> Result<()> {
    let dir = args.file.parent().unwrap_or_else(|| std::path::Path::new("."));
    let stem = args.file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let mut extra_files = Vec::new();
    for entry in std::fs::read_dir(dir)? {
        let entry = entry?;
        let name = entry.file_name();
        let name_str = name.to_string_lossy();
        if name_str.starts_with(stem) && entry.path() != args.file {
            extra_files.push(entry.path());
        }
    }
    if extra_files.is_empty() {
        println!("✓ No auxiliary files found alongside {}", args.file.display());
    } else {
        println!("⚠ Found {} auxiliary files:", extra_files.len());
        for f in &extra_files {
            println!("  {}", f.display());
        }
    }
    Ok(())
}
