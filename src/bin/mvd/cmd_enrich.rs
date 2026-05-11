use anyhow::Result;
use clap::Args;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use crate::common::WriteOpts;

/// Convert parsed `{subject, predicate, value, confidence}` objects into
/// `MemoryCard`s linked to the source frame. Drops entries that are missing
/// any of the three required string fields rather than guessing.
fn build_cards(
    facts: &[serde_json::Value],
    source_frame: memvid_core::FrameId,
    engine_name: &str,
    engine_version: &str,
    model_label: &str,
) -> Vec<memvid_core::MemoryCard> {
    let mut out = Vec::with_capacity(facts.len());
    for f in facts {
        let Some(subject) = f.get("subject").and_then(|v| v.as_str()) else {
            continue;
        };
        let Some(predicate) = f.get("predicate").and_then(|v| v.as_str()) else {
            continue;
        };
        let value_str = match f.get("value") {
            Some(serde_json::Value::String(s)) => s.clone(),
            Some(other) => other.to_string(),
            None => continue,
        };
        if subject.trim().is_empty() || predicate.trim().is_empty() || value_str.trim().is_empty() {
            continue;
        }
        let confidence = f.get("confidence").and_then(|v| v.as_f64()).map(|c| c as f32);

        let mut builder = memvid_core::MemoryCardBuilder::new()
            .fact()
            .entity(subject.trim())
            .slot(predicate.trim())
            .value(value_str.trim())
            .source(source_frame, None)
            .engine(engine_name, format!("{engine_version}+{model_label}"));
        if let Some(c) = confidence {
            builder = builder.confidence(c);
        }
        match builder.build(0) {
            Ok(card) => out.push(card),
            Err(e) => {
                eprintln!("    skip malformed fact ({subject}/{predicate}): {e}");
            }
        }
    }
    out
}

/// Pull a JSON array out of an LLM response that may be wrapped in markdown
/// fences and/or cut off mid-object due to a `max_tokens` truncation.
///
/// Strategy:
/// 1. Strip an outer ```json ... ``` (or ```...```) fence if present.
/// 2. Locate the first `[`.
/// 3. Try the full slice through the last `]` first; if that doesn't parse,
///    rewind to the last complete `},` (or `}]`) and append `]` so a partial
///    final object doesn't poison the array.
fn extract_json_array(raw: &str) -> String {
    let mut s = raw.trim();

    // Drop a leading ``` or ```json fence line.
    if let Some(rest) = s.strip_prefix("```") {
        let rest = rest.strip_prefix("json").unwrap_or(rest);
        let rest = rest.trim_start_matches('\n').trim_start_matches('\r');
        // And the trailing closing fence, if any.
        s = rest
            .strip_suffix("```")
            .map(str::trim_end)
            .unwrap_or(rest)
            .trim_end_matches("```")
            .trim();
    }

    let Some(start) = s.find('[') else {
        return s.to_string();
    };
    let body = &s[start..];

    if let Some(end) = body.rfind(']') {
        let candidate = &body[..=end];
        if serde_json::from_str::<serde_json::Value>(candidate).is_ok() {
            return candidate.to_string();
        }
    }

    // Truncated mid-object: walk back to the last `}` that ends a complete
    // object, drop everything after it, and re-close the array.
    if let Some(last_obj_end) = body.rfind('}') {
        return format!("{}]", &body[..=last_obj_end]);
    }

    body.to_string()
}

#[derive(Args)]
pub struct EnrichArgs {
    pub file: PathBuf,
    /// Use the local LLM (Gemma) for entity/fact extraction instead of rules engine
    #[arg(long)]
    pub llm: bool,
    /// Force re-enrichment of already-enriched frames
    #[arg(long = "reenrich")]
    pub reenrich: bool,
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

    let pending = if args.reenrich {
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
    let mut facts_persisted = 0usize;
    let mut facts_skipped = 0usize;
    let mut errors = 0usize;

    let engine_name = "mvd-enrich-llm";
    let engine_version = env!("CARGO_PKG_VERSION");
    let model_label = "gemma-4-E4B-it";

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
                // Try to parse the JSON array of facts. Gemma reliably wraps
                // its output in ```json ... ``` markdown fences and may also
                // truncate mid-object when hitting max_tokens, so we strip
                // fences and trim back to the last complete object before the
                // closing bracket.
                let json_str = extract_json_array(&response);
                match serde_json::from_str::<Vec<serde_json::Value>>(&json_str) {
                    Ok(facts) => {
                        facts_found += facts.len();
                        let cards = build_cards(
                            &facts,
                            entry.frame_id,
                            engine_name,
                            engine_version,
                            model_label,
                        );
                        let well_formed = cards.len();
                        facts_skipped += facts.len() - well_formed;

                        if well_formed > 0 {
                            match mem.put_memory_cards(cards) {
                                Ok(ids) => {
                                    facts_persisted += ids.len();
                                    eprintln!(
                                        "  Frame {}: {} facts extracted, {} stored",
                                        entry.frame_id,
                                        facts.len(),
                                        ids.len()
                                    );
                                }
                                Err(e) => {
                                    eprintln!(
                                        "  Frame {}: extracted {} facts but persist failed: {e}",
                                        entry.frame_id,
                                        facts.len()
                                    );
                                    errors += 1;
                                }
                            }
                        } else if !facts.is_empty() {
                            eprintln!(
                                "  Frame {}: {} facts extracted, 0 well-formed (all missing subject/predicate/value)",
                                entry.frame_id,
                                facts.len()
                            );
                        }
                    }
                    Err(parse_err) => {
                        // Show the raw response so the user can see *why* parsing
                        // failed (markdown fences, prose preamble, wrong shape, etc.)
                        eprintln!(
                            "  Frame {}: could not parse LLM response: {parse_err}\n  --- raw response ---\n{}\n  --- end raw ---",
                            entry.frame_id,
                            response.trim()
                        );
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
            "{{\"engine\":\"local-llm\",\"model\":\"gemma-4-E4B-it\",\"frames_processed\":{},\"facts_extracted\":{},\"facts_persisted\":{},\"facts_skipped\":{},\"errors\":{}}}",
            processed,
            facts_found,
            facts_persisted,
            facts_skipped,
            errors
        );
    } else {
        println!("\nEnrichment complete (engine: local LLM, model: gemma-4-E4B-it).");
        println!("  Frames processed: {processed}");
        println!("  Facts extracted:  {facts_found}");
        println!("  Facts persisted:  {facts_persisted}");
        println!("  Facts skipped:    {facts_skipped} (missing subject/predicate/value)");
        println!("  Errors:           {errors}");
    }
    Ok(())
}
