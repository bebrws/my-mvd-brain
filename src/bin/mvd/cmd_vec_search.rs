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
    #[arg(long, default_value = "240")]
    pub snippet_chars: usize,
    #[arg(long)]
    pub scope: Option<String>,
    #[arg(long)]
    pub json: bool,
}

pub fn run(args: VecSearchArgs) -> Result<()> {
    let query = args
        .query
        .as_deref()
        .ok_or_else(|| anyhow::anyhow!("--query is required"))?;

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

    let response = mem
        .vec_search_with_embedding(
            query,
            &query_embedding,
            args.limit,
            args.snippet_chars,
            args.scope.as_deref(),
        )
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    if args.json {
        println!("{}", serde_json::to_string_pretty(&response)?);
    } else {
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
