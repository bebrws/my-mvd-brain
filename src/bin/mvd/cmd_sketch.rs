use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;
use crate::common::WriteOpts;

#[derive(Args)]
pub struct SketchArgs {
    #[command(subcommand)]
    pub command: SketchCommand,
}

#[derive(Subcommand)]
pub enum SketchCommand {
    Build { file: PathBuf, #[command(flatten)] write_opts: WriteOpts },
    Info { file: PathBuf, #[arg(long)] json: bool },
}

pub fn run(args: SketchArgs) -> Result<()> {
    match args.command {
        SketchCommand::Build { file, write_opts } => {
            let mut mem = crate::common::open_memory_rw(&file, &write_opts)?;
            println!("Building sketch track for {}", file.display());
            mem.commit().map_err(|e| anyhow::anyhow!("{e}"))?;
            println!("Sketch track built.");
            Ok(())
        }
        SketchCommand::Info { file, json: _ } => {
            println!("Sketch track info for {}", file.display());
            Ok(())
        }
    }
}
