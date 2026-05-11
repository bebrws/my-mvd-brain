use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

use crate::scope::ScopeRead;

#[derive(Args)]
pub struct AskArgs {
    pub file: PathBuf,
    #[arg(long, short = 'q')]
    pub question: Option<String>,
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
    /// Only show retrieved context, skip LLM synthesis
    #[arg(long)]
    pub context_only: bool,
    /// Include source citations in output
    #[arg(long)]
    pub sources: bool,
    #[command(flatten)]
    pub scope: ScopeRead,
}

pub fn run(args: AskArgs) -> Result<()> {
    let question = match args.question {
        Some(q) => q,
        None => {
            return crate::cmd_chat::run(crate::cmd_chat::ChatArgs {
                file: args.file,
                top_k: args.top_k,
                snippet_chars: args.snippet_chars,
            });
        }
    };

    let mut mem = crate::common::open_memory_ro(&args.file)?;
    let resolved_scope = args.scope.resolve();

    // Over-fetch when filtering so the post-filter still has top_k results.
    let fetch_top_k = if resolved_scope.is_unfiltered() {
        args.top_k
    } else {
        args.top_k.saturating_mul(4).max(args.top_k + 16)
    };

    // Use mem.ask() instead of mem.search() to get:
    // - Disjunctive (OR) query fallback when AND returns 0 hits
    // - Expanded query variants (singular/plural)
    // - Timeline fallback as a last resort
    // - Hybrid lex + vector search when embeddings are available
    //
    // We pass embedder: None since creating a full ONNX embedder just for
    // re-ranking adds startup cost. The fallback chain in ask() is sufficient
    // for robust retrieval even without semantic re-ranking.
    let ask_request = memvid_core::AskRequest {
        question: question.clone(),
        top_k: fetch_top_k,
        snippet_chars: args.snippet_chars,
        uri: args.uri.clone(),
        scope: args.search_scope.clone(),
        cursor: args.cursor.clone(),
        start: None,
        end: None,
        #[cfg(feature = "temporal_track")]
        temporal: None,
        context_only: args.context_only,
        mode: memvid_core::AskMode::Hybrid,
        as_of_frame: None,
        as_of_ts: None,
        adaptive: None,
        acl_context: None,
        acl_enforcement_mode: memvid_core::AclEnforcementMode::Audit,
    };

    let mut ask_response = mem
        .ask::<dyn memvid_core::VecEmbedder>(ask_request, None)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let pre_filter = ask_response.retrieval.hits.len();
    resolved_scope.filter_hits(&mut ask_response.retrieval.hits);
    ask_response.retrieval.hits.truncate(args.top_k);
    ask_response.retrieval.total_hits = ask_response.retrieval.hits.len();

    if !args.json && !resolved_scope.is_unfiltered() {
        eprintln!(
            "Scope: {} ({} → {} after filter; --all-repos to span everything)",
            resolved_scope.describe(),
            pre_filter,
            ask_response.retrieval.hits.len()
        );
    }

    let synthesize = !args.context_only && !ask_response.retrieval.hits.is_empty();

    // Build the LLM context once; we reuse it whether we synthesize now (JSON
    // mode) or later (streamed in the non-JSON output flow).
    let (system_prompt, user_prompt) = if synthesize {
        let mut context = String::new();
        for (i, hit) in ask_response.retrieval.hits.iter().enumerate() {
            let score_str = hit.score.map_or("n/a".to_string(), |s| format!("{s:.4}"));
            context.push_str(&format!(
                "[{}] (frame {}, score: {})\n{}\n\n",
                i + 1,
                hit.frame_id,
                score_str,
                hit.text
            ));
        }
        let system = "You are a helpful assistant. Answer the user's question \
            based ONLY on the provided context. If the context does not contain enough \
            information, say so. Cite sources using [N] notation where N is the chunk number.";
        let user = format!(
            "Context:\n{context}\n---\nQuestion: {}\n\nAnswer:",
            question
        );
        (Some(system), Some(user))
    } else {
        (None, None)
    };

    if args.json {
        // JSON mode: synthesize fully (non-streaming), then emit one object.
        let llm_answer = if let (Some(sys), Some(usr)) = (system_prompt, user_prompt.as_ref()) {
            match crate::llm::llm_chat(sys, usr) {
                Ok(answer) => Some(answer),
                Err(e) => {
                    eprintln!("LLM synthesis failed: {e}");
                    None
                }
            }
        } else {
            None
        };

        let mut obj = serde_json::to_value(&ask_response)?;
        if let Some(ref answer) = llm_answer {
            obj["llm_answer"] = serde_json::Value::String(answer.clone());
            obj["model"] = serde_json::Value::String("gemma-4-E4B-it".to_string());
        }
        println!("{}", serde_json::to_string_pretty(&obj)?);
        return Ok(());
    }

    // Plain (human) output. Print the header + optional sources first, then
    // stream the answer in place so the user sees tokens land immediately.
    println!("Question: {}\n", question);
    println!(
        "Retrieved {} relevant chunks (via {:?}).",
        ask_response.retrieval.total_hits, ask_response.retriever,
    );

    let show_sources = args.sources || args.context_only || !synthesize;
    if show_sources {
        for hit in &ask_response.retrieval.hits {
            let score_str = hit.score.map_or("n/a".to_string(), |s| format!("{s:.4}"));
            println!(
                "\n--- [{}] Frame {} (score: {}) ---",
                hit.rank, hit.frame_id, score_str
            );
            println!("{}", hit.text);
        }
    }

    if let Some(ref builtin_answer) = ask_response.answer {
        println!("\n━━━ Context Summary ━━━\n");
        println!("{builtin_answer}");
    }

    if let (Some(sys), Some(usr)) = (system_prompt, user_prompt.as_ref()) {
        use std::io::Write;
        println!("\n━━━ Answer (via local gemma-4-E4B-it) ━━━\n");
        let on_token = |tok: &str| {
            print!("{tok}");
            let _ = std::io::stdout().flush();
        };
        match crate::llm::llm_chat_streaming(sys, usr, on_token) {
            Ok(_) => println!(),
            Err(e) => {
                eprintln!("\nLLM synthesis failed: {e}");
                eprintln!("(Retrieved context shown above.)");
            }
        }
    }

    Ok(())
}
