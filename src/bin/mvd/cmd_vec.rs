use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::scope::ScopeRead;

#[derive(Args)]
pub struct VecArgs {
    pub file: PathBuf,
    /// Query text to embed and search.
    #[arg(long)]
    pub query: String,
    #[arg(long, default_value = "8")]
    pub limit: usize,
    #[arg(long, default_value = "240")]
    pub snippet_chars: usize,
    #[arg(long = "search-scope")]
    pub search_scope: Option<String>,
    #[arg(long)]
    pub json: bool,
    #[command(flatten)]
    pub scope: ScopeRead,
}

pub fn run(args: VecArgs) -> Result<()> {
    let query = args.query.as_str();

    let mut mem = crate::common::open_memory_ro(&args.file)?;

    // Use the local text embedder to generate a query embedding,
    // then perform true vector (cosine similarity) search.
    let config = memvid_core::text_embed::TextEmbedConfig::default();
    let embedder = memvid_core::text_embed::LocalTextEmbedder::new(config)
        .map_err(|e| anyhow::anyhow!("Failed to load text embedder: {e}"))?;

    use memvid_core::types::embedding::EmbeddingProvider;
    let query_embedding = embedder
        .embed_text(query)
        .map_err(|e| anyhow::anyhow!("Failed to embed query: {e}"))?;

    let resolved_scope = args.scope.resolve();
    let fetch_limit = if resolved_scope.is_unfiltered() {
        args.limit
    } else {
        args.limit.saturating_mul(4).max(args.limit + 16)
    };

    let mut response = mem
        .vec_search_with_embedding(
            query,
            &query_embedding,
            fetch_limit,
            args.snippet_chars,
            args.search_scope.as_deref(),
        )
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let pre_filter = response.hits.len();
    resolved_scope.filter_hits(&mut response.hits);
    response.hits.truncate(args.limit);
    response.total_hits = response.hits.len();

    if args.json {
        println!("{}", serde_json::to_string_pretty(&response)?);
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
            println!(
                "\n--- Frame {} (score: {}) ---",
                hit.frame_id, score_str
            );
            println!("{}", hit.text);
        }
    }
    Ok(())
}
