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
    /// Traverse the entity graph following a relationship type
    Traverse {
        file: PathBuf,
        #[arg(long)]
        entity: String,
        /// Relationship type to follow (e.g. "employer", "manager")
        #[arg(long, default_value = "related")]
        link: String,
        #[arg(long, default_value = "2")]
        depth: usize,
        #[arg(long)]
        json: bool,
    },
    /// List all entities in the Logic-Mesh
    Entities {
        file: PathBuf,
        /// Filter by entity kind (person, organization, location, etc.)
        #[arg(long)]
        kind: Option<String>,
        #[arg(long)]
        json: bool,
    },
    /// Show graph statistics
    Stats {
        file: PathBuf,
        #[arg(long)]
        json: bool,
    },
}

pub fn run(args: FollowArgs) -> Result<()> {
    match args.command {
        FollowCommand::Traverse {
            file,
            entity,
            link,
            depth,
            json,
        } => {
            let mem = crate::common::open_memory_ro(&file)?;

            if !mem.has_logic_mesh() {
                eprintln!("No Logic-Mesh found in {}. Run `mvd enrich` first.", file.display());
                return Ok(());
            }

            let results = mem.follow(&entity, &link, depth);

            if json {
                println!("{}", serde_json::to_string_pretty(&results)?);
            } else {
                if results.is_empty() {
                    println!("No results found following '{entity}' via '{link}' (depth {depth}).");
                } else {
                    println!(
                        "Follow: {} → '{}' (depth {depth})\n",
                        entity, link
                    );
                    for r in &results {
                        println!(
                            "  {} ({:?}) — hop {}",
                            r.node, r.kind, r.path_length
                        );
                    }
                }
            }
            Ok(())
        }
        FollowCommand::Entities { file, kind, json } => {
            let mem = crate::common::open_memory_ro(&file)?;

            if !mem.has_logic_mesh() {
                eprintln!("No Logic-Mesh found in {}.", file.display());
                return Ok(());
            }

            let mesh = mem.logic_mesh();
            let nodes: Vec<_> = if let Some(ref kind_str) = kind {
                mesh.nodes
                    .iter()
                    .filter(|n| n.kind.as_str().eq_ignore_ascii_case(kind_str))
                    .collect()
            } else {
                mesh.nodes.iter().collect()
            };

            if json {
                let items: Vec<_> = nodes
                    .iter()
                    .map(|n| {
                        serde_json::json!({
                            "name": n.display_name,
                            "kind": n.kind.as_str(),
                            "mentions": n.mentions.len(),
                            "frames": n.frame_ids.len(),
                        })
                    })
                    .collect();
                println!("{}", serde_json::to_string_pretty(&items)?);
            } else {
                println!(
                    "{:<30} {:<15} {:<10} {}",
                    "Entity", "Kind", "Mentions", "Frames"
                );
                println!("{}", "-".repeat(70));
                for n in &nodes {
                    println!(
                        "{:<30} {:<15} {:<10} {}",
                        n.display_name,
                        n.kind.as_str(),
                        n.mentions.len(),
                        n.frame_ids.len(),
                    );
                }
                println!("\nTotal: {} entities", nodes.len());
            }
            Ok(())
        }
        FollowCommand::Stats { file, json } => {
            let mem = crate::common::open_memory_ro(&file)?;
            let stats = mem.logic_mesh_stats();

            if json {
                println!("{}", serde_json::to_string_pretty(&stats)?);
            } else {
                println!("Logic-Mesh stats for {}", file.display());
                println!("  Nodes: {}", stats.node_count);
                println!("  Edges: {}", stats.edge_count);
                if !stats.entity_kinds.is_empty() {
                    println!("  By kind:");
                    for (kind, count) in &stats.entity_kinds {
                        println!("    {kind}: {count}");
                    }
                }
                if !stats.link_types.is_empty() {
                    println!("  Edge types:");
                    for (link_type, count) in &stats.link_types {
                        println!("    {link_type}: {count}");
                    }
                }
            }
            Ok(())
        }
    }
}
