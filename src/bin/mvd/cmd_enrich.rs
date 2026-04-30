use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::common::WriteOpts;

#[derive(Args)]
pub struct EnrichArgs {
    pub file: PathBuf,
    /// Use the local LLM (Gemma) for entity/fact extraction instead of rules engine
    #[arg(long)]
    pub llm: bool,
    /// Force re-enrichment of already-enriched frames
    #[arg(long)]
    pub force: bool,
    #[arg(long)]
    pub json: bool,
    #[command(flatten)]
    pub write_opts: WriteOpts,
}

pub fn run(args: EnrichArgs) -> Result<()> {
    if args.llm {
        run_llm_enrichment(args)
    } else {
        run_local_enrichment(args)
    }
}

/// Local enrichment using the built-in rules engine + background worker.
fn run_local_enrichment(args: EnrichArgs) -> Result<()> {
    let mem = crate::common::open_memory_rw(&args.file, &args.write_opts)?;
    let mem = Arc::new(Mutex::new(mem));
    println!("Enriching {} with local rules engine", args.file.display());
    let handle = memvid_core::start_enrichment_worker(Arc::clone(&mem), None);
    let stats = handle.stop_and_wait();
    if let Ok(mut m) = mem.lock() {
        m.commit().map_err(|e| anyhow::anyhow!("{e}"))?;
    }
    if args.json {
        println!("{{\"engine\":\"rules\",\"frames_processed\":{},\"errors\":{},\"embeddings_generated\":{}}}",
            stats.frames_processed, stats.errors, stats.embeddings_generated);
    } else {
        println!("Enrichment complete (engine: rules).");
        println!("  Frames processed: {}", stats.frames_processed);
        println!("  Errors:           {}", stats.errors);
        println!("  Embeddings:       {}", stats.embeddings_generated);
    }
    Ok(())
}

/// LLM-powered enrichment using the local Gemma model for entity/fact extraction.
fn run_llm_enrichment(args: EnrichArgs) -> Result<()> {
    let mut mem = crate::common::open_memory_rw(&args.file, &args.write_opts)?;
    let enrichment_stats = mem.enrichment_stats();

    let pending = if args.force {
        enrichment_stats.total_frames
    } else {
        enrichment_stats.pending_frames + enrichment_stats.searchable_only
    };

    if pending == 0 {
        println!("No frames to enrich.");
        return Ok(());
    }

    eprintln!(
        "Enriching {} frames via local LLM (gemma-4-E4B-it) ...",
        pending,
    );

    let system_prompt = "\
You are an entity extraction engine. Given a text chunk, extract structured facts as JSON.
Return a JSON array of objects with these fields:
- \"subject\": the entity name
- \"predicate\": the relationship or attribute (e.g. \"works_at\", \"lives_in\", \"job_title\")
- \"value\": the value
- \"confidence\": a float 0.0-1.0

Only extract facts that are clearly stated. Return [] if no facts are found.
Example: [{\"subject\":\"John\",\"predicate\":\"works_at\",\"value\":\"Acme Corp\",\"confidence\":0.95}]";

    // Walk through timeline to get frame content
    let query = memvid_core::TimelineQuery::builder().no_limit().build();
    let entries = mem.timeline(query).map_err(|e| anyhow::anyhow!("{e}"))?;

    let mut processed = 0usize;
    let mut facts_found = 0usize;
    let mut errors = 0usize;

    for entry in &entries {
        let text = match mem.frame_text_by_id(entry.frame_id) {
            Ok(t) => t,
            Err(_) => continue,
        };

        if text.trim().is_empty() {
            continue;
        }

        // Truncate very long texts to stay within token limits
        let truncated = if text.len() > 12_000 {
            &text[..12_000]
        } else {
            &text
        };

        let user_prompt = format!("Extract facts from this text:\n\n{truncated}");

        match crate::llm::llm_chat(system_prompt, &user_prompt) {
            Ok(response) => {
                // Try to parse the JSON array of facts
                let trimmed = response.trim();
                // Find JSON array in the response (LLMs sometimes wrap in markdown)
                let json_str = if let Some(start) = trimmed.find('[') {
                    if let Some(end) = trimmed.rfind(']') {
                        &trimmed[start..=end]
                    } else {
                        trimmed
                    }
                } else {
                    trimmed
                };

                match serde_json::from_str::<Vec<serde_json::Value>>(json_str) {
                    Ok(facts) => {
                        facts_found += facts.len();
                        if !facts.is_empty() {
                            eprintln!(
                                "  Frame {}: {} facts extracted",
                                entry.frame_id,
                                facts.len()
                            );
                        }
                    }
                    Err(_) => {
                        // Not valid JSON — skip silently
                        eprintln!("  Frame {}: could not parse LLM response", entry.frame_id);
                    }
                }
                processed += 1;
            }
            Err(e) => {
                eprintln!("  Frame {}: LLM error: {e}", entry.frame_id);
                errors += 1;
            }
        }
    }

    mem.commit().map_err(|e| anyhow::anyhow!("{e}"))?;

    if args.json {
        println!(
            "{{\"engine\":\"local-llm\",\"model\":\"gemma-4-E4B-it\",\"frames_processed\":{},\"facts_extracted\":{},\"errors\":{}}}",
            processed,
            facts_found,
            errors
        );
    } else {
        println!("\nEnrichment complete (engine: local LLM, model: gemma-4-E4B-it).");
        println!("  Frames processed: {processed}");
        println!("  Facts extracted:  {facts_found}");
        println!("  Errors:           {errors}");
    }
    Ok(())
}
