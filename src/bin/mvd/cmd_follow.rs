use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct FollowArgs {
    #[command(subcommand)]
    pub command: FollowCommand,
}

#[derive(Subcommand)]
pub enum FollowCommand {
    /// Traverse the entity graph
    Traverse { file: PathBuf, #[arg(long)] entity: String, #[arg(long, default_value = "2")] depth: usize },
    /// List entities
    Entities { file: PathBuf },
    /// Show graph statistics
    Stats { file: PathBuf, #[arg(long)] json: bool },
}

pub fn run(args: FollowArgs) -> Result<()> {
    match args.command {
        FollowCommand::Traverse { file, entity, depth } => {
            println!("Traversing from '{entity}' depth={depth} in {}", file.display());
            Ok(())
        }
        FollowCommand::Entities { file } => {
            println!("Entities in {}", file.display());
            Ok(())
        }
        FollowCommand::Stats { file, json: _ } => {
            println!("Logic-Mesh stats for {}", file.display());
            Ok(())
        }
    }
}
