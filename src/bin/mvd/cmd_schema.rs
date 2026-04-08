use anyhow::Result;
use clap::{Args, Subcommand};
use std::path::PathBuf;

#[derive(Args)]
pub struct SchemaArgs {
    #[command(subcommand)]
    pub command: SchemaCommand,
}

#[derive(Subcommand)]
pub enum SchemaCommand {
    Infer { file: PathBuf },
    List { file: PathBuf },
}

pub fn run(args: SchemaArgs) -> Result<()> {
    match args.command {
        SchemaCommand::Infer { file } => { println!("Schema inference for {}", file.display()); Ok(()) }
        SchemaCommand::List { file } => { println!("Registered schemas for {}", file.display()); Ok(()) }
    }
}
