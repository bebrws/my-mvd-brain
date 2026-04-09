use anyhow::Result;
use clap::Args;
use std::path::PathBuf;

#[derive(Args)]
pub struct AskArgs {
    pub file: PathBuf,
    #[arg(long, short = 'q')]
    pub question: String,
    /// LLM provider for answer synthesis: openai, claude, groq, gemini
    /// If not set, only retrieval context is shown (no LLM synthesis).
    #[arg(long = "use-model")]
    pub use_model: Option<String>,
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
    /// Only show retrieved context, skip synthesis
    #[arg(long)]
    pub context_only: bool,
    /// Include source citations in output
    #[arg(long)]
    pub sources: bool,
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

    // If --use-model is set and not --context-only, do LLM synthesis
    let llm_answer = if !args.context_only {
        if let Some(ref model_name) = args.use_model {
            let provider = crate::llm::LlmProvider::from_str(model_name)?;

            // Build context from retrieved chunks
            let mut context = String::new();
            for (i, hit) in response.hits.iter().enumerate() {
                let score_str = hit.score.map_or("n/a".to_string(), |s| format!("{s:.4}"));
                context.push_str(&format!(
                    "[{}] (frame {}, score: {})\n{}\n\n",
                    i + 1,
                    hit.frame_id,
                    score_str,
                    hit.text
                ));
            }

            if context.is_empty() {
                None
            } else {
                let system_prompt = "You are a helpful assistant. Answer the user's question \
                    based ONLY on the provided context. If the context does not contain enough \
                    information, say so. Cite sources using [N] notation where N is the chunk number.";
                let user_prompt = format!(
                    "Context:\n{context}\n---\nQuestion: {}\n\nAnswer:",
                    args.question
                );

                eprintln!("Synthesizing answer via {} ...", provider.label());
                let answer = crate::llm::llm_chat(provider, system_prompt, &user_prompt)?;
                Some(answer)
            }
        } else {
            None
        }
    } else {
        None
    };

    if args.json {
        let mut obj = serde_json::to_value(&response)?;
        if let Some(ref answer) = llm_answer {
            obj["answer"] = serde_json::Value::String(answer.clone());
            obj["model"] = serde_json::Value::String(
                args.use_model.clone().unwrap_or_default(),
            );
        }
        println!("{}", serde_json::to_string_pretty(&obj)?);
    } else {
        println!("Question: {}\n", args.question);
        println!("Retrieved {} relevant chunks.", response.total_hits);

        if args.sources || args.context_only || llm_answer.is_none() {
            for hit in &response.hits {
                let score_str = hit.score.map_or("n/a".to_string(), |s| format!("{s:.4}"));
                println!("\n--- [{}] Frame {} (score: {}) ---", hit.rank, hit.frame_id, score_str);
                println!("{}", hit.text);
            }
        }

        if let Some(ref answer) = llm_answer {
            let model_label = args
                .use_model
                .as_deref()
                .unwrap_or("unknown");
            println!("\n━━━ Answer (via {model_label}) ━━━\n");
            println!("{answer}");
        } else if !args.context_only && args.use_model.is_none() {
            println!("\n[Use --use-model openai|claude|groq|gemini for LLM synthesis]");
        }
    }
    Ok(())
}
