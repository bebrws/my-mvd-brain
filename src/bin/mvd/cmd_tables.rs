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
    List {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
    /// View a specific table by ID
    View {
        file: PathBuf,
        #[arg(long)]
        table_id: String,
        /// Export format: csv, json, or text (default)
        #[arg(long, default_value = "text")]
        format: String,
    },
}

pub fn run(args: TablesArgs) -> Result<()> {
    match args.command {
        TablesCommand::List { file, json } => {
            let mut mem = crate::common::open_memory_ro(&file)?;
            let tables = memvid_core::table::list_tables(&mut mem)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            if json {
                println!("{}", serde_json::to_string_pretty(&tables)?);
            } else {
                if tables.is_empty() {
                    println!("No tables found in {}", file.display());
                } else {
                    println!(
                        "{:<20} {:<25} {:<6} {:<6} {:<10} {}",
                        "Table ID", "Source", "Rows", "Cols", "Quality", "Pages"
                    );
                    println!("{}", "-".repeat(80));
                    for t in &tables {
                        println!(
                            "{:<20} {:<25} {:<6} {:<6} {:<10} {}-{}",
                            t.table_id,
                            t.source_file,
                            t.n_rows,
                            t.n_cols,
                            format!("{:?}", t.quality),
                            t.page_start,
                            t.page_end,
                        );
                    }
                    println!("\nTotal: {} tables", tables.len());
                }
            }
            Ok(())
        }
        TablesCommand::View {
            file,
            table_id,
            format,
        } => {
            let mut mem = crate::common::open_memory_ro(&file)?;
            let table = memvid_core::table::get_table(&mut mem, &table_id)
                .map_err(|e| anyhow::anyhow!("{e}"))?;

            match table {
                Some(t) => match format.as_str() {
                    "csv" => {
                        print!("{}", memvid_core::table::export_to_csv(&t));
                    }
                    "json" => {
                        let json = memvid_core::table::export_to_json(&t, true)
                            .map_err(|e| anyhow::anyhow!("{e}"))?;
                        println!("{json}");
                    }
                    _ => {
                        println!("Table: {}", t.table_id);
                        println!("Source: {}", t.source_file);
                        println!(
                            "Pages: {}-{}, Rows: {}, Cols: {}",
                            t.page_start, t.page_end, t.n_rows, t.n_cols
                        );
                        println!("Quality: {:?}", t.quality);
                        if !t.headers.is_empty() {
                            println!("\nHeaders: {}", t.headers.join(" | "));
                        }
                        println!();
                        // Print as simple text table
                        for row in t.data_rows() {
                            let cells: Vec<&str> =
                                row.cells.iter().map(|c| c.text.as_str()).collect();
                            println!("  {}", cells.join(" | "));
                        }
                    }
                },
                None => {
                    eprintln!("Table '{table_id}' not found in {}", file.display());
                }
            }
            Ok(())
        }
    }
}
