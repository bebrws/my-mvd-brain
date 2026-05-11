use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::scope::ScopeRead;

#[derive(Args)]
pub struct FindArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub query: String,
    #[arg(long)]
    pub uri: Option<String>,
    #[arg(long = "search-scope")]
    pub search_scope: Option<String>,
    #[arg(long, default_value = "8")]
    pub top_k: usize,
    #[arg(long = "snippet-chars", default_value = "480")]
    pub snippet_chars: usize,
    #[arg(long)]
    pub cursor: Option<String>,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub no_sketch: bool,
    #[command(flatten)]
    pub scope: ScopeRead,
}

pub fn run(args: FindArgs) -> Result<()> {
    let mut mem = crate::common::open_memory_ro(&args.file)?;
    let resolved_scope = args.scope.resolve();

    // Over-fetch when filtering so we still have ~top_k after the post-filter.
    let fetch_top_k = if resolved_scope.is_unfiltered() {
        args.top_k
    } else {
        args.top_k.saturating_mul(4).max(args.top_k + 16)
    };

    let request = memvid_core::SearchRequest {
        query: args.query.clone(),
        top_k: fetch_top_k,
        snippet_chars: args.snippet_chars,
        uri: args.uri.clone(),
        scope: args.search_scope.clone(),
        cursor: args.cursor.clone(),
        #[cfg(feature = "temporal_track")]
        temporal: None,
        as_of_frame: None,
        as_of_ts: None,
        no_sketch: args.no_sketch,
        acl_context: None,
        acl_enforcement_mode: memvid_core::AclEnforcementMode::Audit,
    };
    let mut response = mem.search(request).map_err(|e| anyhow::anyhow!("{e}"))?;

    let pre_filter = response.hits.len();
    resolved_scope.filter_hits(&mut response.hits);
    response.hits.truncate(args.top_k);
    response.total_hits = response.hits.len();

    if args.json {
        let json = serde_json::to_string_pretty(&response)?;
        println!("{json}");
    } else {
        if !resolved_scope.is_unfiltered() {
            println!(
                "Scope: {} ({} → {} after filter; --all-repos to span everything)",
                resolved_scope.describe(),
                pre_filter,
                response.hits.len()
            );
        }
        println!(
            "Found {} results (engine: {:?})",
            response.total_hits, response.engine
        );
        for hit in &response.hits {
            let score_str = hit.score.map_or("n/a".to_string(), |s| format!("{s:.4}"));
            println!("\n--- Frame {} (score: {}) ---", hit.frame_id, score_str);
            println!("{}", hit.text);
        }
    }
    Ok(())
}
