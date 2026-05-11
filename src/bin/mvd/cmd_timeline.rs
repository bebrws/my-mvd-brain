use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::scope::ScopeRead;

#[derive(Args)]
pub struct TimelineArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub reverse: bool,
    #[arg(long)]
    pub limit: Option<u64>,
    #[command(flatten)]
    pub scope: ScopeRead,
}

pub fn run(args: TimelineArgs) -> Result<()> {
    let mut mem = crate::common::open_memory_ro(&args.file)?;
    let resolved_scope = args.scope.resolve();

    let mut builder = memvid_core::TimelineQuery::builder()
        .reverse(args.reverse);
    // When filtering, fetch all frames and trim post-hoc (the scope filter
    // could otherwise hide every frame in the requested limit).
    let user_limit = args.limit;
    if let Some(limit) = user_limit {
        if resolved_scope.is_unfiltered() {
            if let Some(nz) = std::num::NonZeroU64::new(limit) {
                builder = builder.limit(nz);
            }
        }
    }
    let query = builder.build();
    let mut entries = mem.timeline(query).map_err(|e| anyhow::anyhow!("{e}"))?;

    if !resolved_scope.is_unfiltered() {
        entries.retain(|entry| match mem.frame_by_id(entry.frame_id) {
            Ok(frame) => resolved_scope.matches(&frame.extra_metadata),
            Err(_) => false,
        });
        if let Some(limit) = user_limit {
            entries.truncate(limit as usize);
        }
    }

    if args.json {
        let json = serde_json::to_string_pretty(&entries)?;
        println!("{json}");
    } else {
        if !resolved_scope.is_unfiltered() {
            println!(
                "Scope: {} (--all-repos to span everything)",
                resolved_scope.describe()
            );
        }
        for entry in &entries {
            println!("#{:<6} {} {}", entry.frame_id, entry.timestamp, entry.preview);
        }
        if entries.is_empty() {
            println!("No frames found.");
        }
    }
    Ok(())
}
