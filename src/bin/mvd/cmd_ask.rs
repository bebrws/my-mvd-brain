use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct AskArgs {
    pub file: PathBuf,
    #[arg(long, short = 'q')]
    pub question: String,
    #[arg(long)]
    pub uri: Option<String>,
    #[arg(long)]
    pub scope: Option<String>,
    #[arg(long, default_value = "8")]
    pub top_k: usize,
    #[arg(long = "snippet-chars", default_value = "480")]
    pub snippet_chars: usize,
    #[arg(long)]
    pub cursor: Option<String>,
    #[arg(long)]
    pub json: bool,
    #[arg(long)]
    pub context_only: bool,
    #[arg(long)]
    pub no_llm: bool,
}

pub fn run(args: AskArgs) -> Result<()> {
    let mut mem = crate::common::open_memory_ro(&args.file)?;
    let search_req = memvid_core::SearchRequest {
        query: args.question.clone(),
        top_k: args.top_k,
        snippet_chars: args.snippet_chars,
        uri: args.uri.clone(),
        scope: args.scope.clone(),
        cursor: args.cursor.clone(),
        #[cfg(feature = "temporal_track")]
        temporal: None,
        as_of_frame: None,
        as_of_ts: None,
        no_sketch: false,
        acl_context: None,
        acl_enforcement_mode: memvid_core::AclEnforcementMode::Audit,
    };
    let response = mem.search(search_req).map_err(|e| anyhow::anyhow!("{e}"))?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
        println!("Question: {}\n", args.question);
        println!("Retrieved {} relevant chunks.", response.total_hits);
        for hit in &response.hits {
            let score_str = hit.score.map_or("n/a".to_string(), |s| format!("{s:.4}"));
            println!("\n--- Frame {} (score: {}) ---", hit.frame_id, score_str);
            println!("{}", hit.text);
        }
        if !args.context_only && !args.no_llm {
            println!("\n[LLM synthesis requires an external LLM API]");
        }
    }
    Ok(())
}
