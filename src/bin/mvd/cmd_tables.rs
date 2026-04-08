use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct TablesArgs {
    #[command(subcommand)]
    pub command: TablesCommand,
}

#[derive(Subcommand)]
pub enum TablesCommand {
    /// List extracted tables
    List { file: PathBuf, #[arg(long)] json: bool },
    /// View a specific table
    View { file: PathBuf, #[arg(long)] table_id: u64 },
}

pub fn run(args: TablesArgs) -> Result<()> {
    match args.command {
        TablesCommand::List { file, json: _ } => {
            let _mem = crate::common::open_memory_ro(&file)?;
            println!("Tables in {}", file.display());
            Ok(())
        }
        TablesCommand::View { file, table_id } => {
            println!("Table {table_id} from {}", file.display());
            Ok(())
        }
    }
}
