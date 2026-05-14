use anyhow::Result;
use clap::Args;
use std::io::{self, Write};
use std::path::PathBuf;

#[derive(Args)]
pub struct ChatArgs {
    pub file: PathBuf,
    #[arg(long, default_value = "8")]
    pub top_k: usize,
    #[arg(long, default_value = "240")]
    pub snippet_chars: usize,
}

pub fn run(args: ChatArgs) -> Result<()> {
    let write_opts = crate::common::WriteOpts {
        lock_timeout: 250,
        force: false,
    };
    let mut mem = crate::common::open_memory_rw(&args.file, &write_opts)?;

    // Attempt to load any active replay session so we can record this conversation
    let _ = mem.load_active_session();

    // Load the LLM eagerly so the loading messages don't appear *after* the
    // first ">" prompt, making the user think their question hung.
    if let Err(e) = crate::llm::ensure_loaded() {
        eprintln!("Warning: local LLM unavailable: {e}");
        eprintln!("Chat will still retrieve context but cannot synthesize answers.");
    }

    println!("Memvid Interactive Chat. Type 'exit' or 'quit' to end.");
    if let Some(id) = mem.active_session_id() {
        println!("Recording interaction to ReplaySession: {}", id);
    }

    let mut history = String::new();

    loop {
        print!("\n> ");
        io::stdout().flush()?;

        let mut input = String::new();
        if io::stdin().read_line(&mut input).is_err() {
            break;
        }
        let input = input.trim();
        if input.is_empty() {
            continue;
        }
        if input == "exit" || input == "quit" {
            break;
        }

        let ask_request = memvid_core::AskRequest {
            question: input.to_string(),
            top_k: args.top_k,
            snippet_chars: args.snippet_chars,
            uri: None,
            scope: None,
            cursor: None,
            start: None,
            end: None,
            #[cfg(feature = "temporal_track")]
            temporal: None,
            context_only: false,
            mode: memvid_core::AskMode::Hybrid,
            as_of_frame: None,
            as_of_ts: None,
            adaptive: None,
            acl_context: None,
            acl_enforcement_mode: memvid_core::AclEnforcementMode::Audit,
        };

        let ask_response = match mem.ask::<dyn memvid_core::VecEmbedder>(ask_request, None) {
            Ok(resp) => resp,
            Err(e) => {
                eprintln!("\nRetrieval error: {}", e);
                continue;
            }
        };

        let mut context = String::new();
        for (i, hit) in ask_response.retrieval.hits.iter().enumerate() {
            let score_str = hit.score.map_or("n/a".to_string(), |s| format!("{s:.4}"));
            context.push_str(&format!(
                "[{}] (frame {}, score: {})\n{}\n\n",
                i + 1, hit.frame_id, score_str, hit.text
            ));
        }

        let answer = if context.is_empty() {
            let msg = "No relevant context found in memory.".to_string();
            println!("\n{msg}");
            msg
        } else {
            let system_prompt = "You are a helpful assistant. Answer the user's question \
                based ONLY on the provided context. If the context does not contain enough \
                information, say so. Cite sources using [N] notation where N is the chunk number.";
            let user_prompt = format!(
                "Conversation History:\n{}\n\nContext:\n{}\n---\nQuestion: {}\n\nAnswer:",
                history, context, input
            );

            // Stream tokens to stdout as they arrive so the user sees progress
            // and isn't left wondering whether the model is hung.
            print!("\n");
            io::stdout().flush()?;
            let stdout_print = |tok: &str| {
                print!("{tok}");
                let _ = io::stdout().flush();
            };
            match crate::llm::llm_chat_streaming(system_prompt, &user_prompt, stdout_print) {
                Ok(ans) => {
                    println!();
                    ans
                }
                Err(e) => {
                    let fallback =
                        format!("LLM synthesis failed: {e}\n\nRetrieved context:\n{context}");
                    println!("\n{fallback}");
                    fallback
                }
            }
        };

        // Note: memvid internally records the RAG Ask request into the active ReplaySession
        // (if one exists). The synthesis steps are currently part of the CLI layer, not core,
        // so we don't explicitly record the chat synthesis output into the replay action yet.
        // We just ensure we commit the session if anything changed.
        
        history.push_str(&format!("User: {}\nAssistant: {}\n", input, answer));
        if history.len() > 8000 {
            let trim_idx = history.len() - 8000;
            history = history[trim_idx..].to_string();
        }

        if mem.active_session_id().is_some() {
            let _ = mem.save_active_session();
            let _ = mem.commit();
        }
    }

    Ok(())
}
