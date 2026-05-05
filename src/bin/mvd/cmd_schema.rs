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
    /// Infer schemas from existing memory cards
    Infer {
        file: PathBuf,
        /// Register inferred schemas in the registry
        #[arg(long)]
        register: bool,
        /// Overwrite existing schemas when registering
        #[arg(long)]
        overwrite: bool,
        #[arg(long)]
        json: bool,
    },
    /// List registered schemas and inferred summaries
    List {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(args: SchemaArgs) -> Result<()> {
    match args.command {
        SchemaCommand::Infer {
            file,
            register,
            overwrite,
            json,
        } => {
            if register {
                let default_opts = crate::common::WriteOpts {
                    lock_timeout: 250,
                    force: false,
                };
                let mut mem = crate::common::open_memory_rw(&file, &default_opts)?;
                let count = mem.register_inferred_schemas(overwrite);
                mem.commit().map_err(|e| anyhow::anyhow!("{e}"))?;
                println!("Registered {count} inferred schemas.");
            } else {
                let mem = crate::common::open_memory_ro(&file)?;
                let schemas = mem.infer_schemas();
                if json {
                    println!("{}", serde_json::to_string_pretty(&schemas)?);
                } else {
                    if schemas.is_empty() {
                        println!("No schemas inferred (no memory cards found).");
                    } else {
                        println!("Inferred {} schemas:\n", schemas.len());
                        for s in &schemas {
                            println!(
                                "  {}: {} ({:?})",
                                s.id, s.range.description(), s.cardinality
                            );
                        }
                    }
                }
            }
            Ok(())
        }
        SchemaCommand::List { file, json } => {
            let mem = crate::common::open_memory_ro(&file)?;
            let summary = mem.schema_summary();

            if json {
                println!("{}", serde_json::to_string_pretty(&summary)?);
            } else {
                if summary.is_empty() {
                    println!("No schemas found (no memory cards in file).");
                } else {
                    println!(
                        "{:<20} {:<10} {:<12} {:<8} {:<8} {:<8} {}",
                        "Predicate", "Type", "Cardinality", "Entities", "Values", "Unique", "Builtin"
                    );
                    println!("{}", "-".repeat(80));
                    for entry in &summary {
                        println!(
                            "{:<20} {:<10} {:<12} {:<8} {:<8} {:<8} {}",
                            entry.predicate,
                            entry.inferred_type,
                            format!("{:?}", entry.cardinality),
                            entry.entity_count,
                            entry.value_count,
                            entry.unique_values,
                            if entry.is_builtin { "✓" } else { "" },
                        );
                    }
                }
            }
            Ok(())
        }
    }
}
