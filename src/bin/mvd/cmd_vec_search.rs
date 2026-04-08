use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct VecSearchArgs {
    pub file: PathBuf,
    #[arg(long)]
    pub query: Option<String>,
    #[arg(long, default_value = "8")]
    pub limit: usize,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: VecSearchArgs) -> Result<()> {
    let query = args.query.as_deref()
        .ok_or_else(|| anyhow::anyhow!("--query is required"))?;
    // Use the regular search with semantic mode
    let mut mem = crate::common::open_memory_ro(&args.file)?;
    let request = memvid_core::SearchRequest {
        query: query.to_string(),
        top_k: args.limit,
        snippet_chars: 240,
        uri: None,
        scope: None,
        cursor: None,
        #[cfg(feature = "temporal_track")]
        temporal: None,
        as_of_frame: None,
        as_of_ts: None,
        no_sketch: false,
        acl_context: None,
        acl_enforcement_mode: memvid_core::AclEnforcementMode::Audit,
    };
    let response = mem.search(request).map_err(|e| anyhow::anyhow!("{e}"))?;
    if args.json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        for hit in &response.hits {
            let score_str = hit.score.map_or("n/a".to_string(), |s| format!("{s:.4}"));
            println!("Frame {} (score: {})", hit.frame_id, score_str);
        }
    }
    Ok(())
}
