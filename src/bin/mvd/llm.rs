//! Local LLM inference for mvd CLI using mistral.rs.
//!
//! Runs `google/gemma-4-E4B-it` locally via the mistral.rs SDK. Gemma 4 is
//! exposed only as `Gemma4ForConditionalGeneration` (a multimodal class), so
//! the text-only `TextModelBuilder` rejects it. mistralrs also ships an
//! auto-detecting `ModelBuilder`, but in 0.8.1 that auto-detector misroutes
//! Gemma 4 to the text-only `-CausalLM` loader and fails with
//! "Unsupported model class". We therefore use `MultimodalModelBuilder`
//! directly with `MultimodalLoaderType::Gemma4`, which talks to the multimodal
//! pipeline but accepts plain text chat requests. On macOS the Metal backend
//! is enabled via Cargo target deps, so generation runs on the GPU. No
//! external API calls or API keys.
//!
//! ## Quantization caching (UQFF)
//!
//! `with_isq(Q4K)` quantizes the bf16 weights to 4-bit at every load — a
//! ~30-120 s CPU/GPU pass. mistralrs supports persisting the quantized form
//! as UQFF so subsequent loads are seconds, not minutes. We use that:
//!
//! * `mvd setup` calls `warm_uqff_cache()`, which builds with
//!   `.with_isq(Q4K).write_uqff(...)` — quantize once, serialize to disk.
//! * Every other LLM-using command (`mvd chat`, `mvd ask`, `mvd enrich --llm`)
//!   prefers `UqffMultimodalModelBuilder::new(MODEL_ID, [shard0]).build()`,
//!   which loads pre-quantized weights and skips the ISQ pass.
//! * If the UQFF cache is missing we fall back to the legacy quantize-on-load
//!   path, with an honest log line about what's about to happen.

use anyhow::Result;
#[cfg(feature = "local-llm")]
use std::path::PathBuf;

/// Hugging Face id of the model used by `mvd chat` / `mvd ask` for synthesis.
#[cfg(feature = "local-llm")]
const MODEL_ID: &str = "google/gemma-4-E4B-it";

/// Default cap on generated tokens per response. Keeps CPU fallbacks from
/// looking hung and bounds latency on slower hardware. Sized to fit ~10–20
/// JSON fact objects for enrichment without truncating mid-object.
#[cfg(feature = "local-llm")]
const DEFAULT_MAX_TOKENS: usize = 2048;

/// Stem of the UQFF write path. mistralrs shards into `<stem>-0.uqff`,
/// `<stem>-1.uqff`, ... in the parent directory.
#[cfg(feature = "local-llm")]
const UQFF_STEM: &str = "q4k.uqff";

/// First shard filename — what we look for to detect "cache present", and
/// what we hand to `UqffMultimodalModelBuilder` for read.
#[cfg(feature = "local-llm")]
const UQFF_FIRST_SHARD: &str = "q4k-0.uqff";

/// `~/.cache/memvid/llm/gemma-4-E4B-it/`
#[cfg(feature = "local-llm")]
fn uqff_dir() -> Option<PathBuf> {
    dirs_next::cache_dir().map(|p| p.join("memvid").join("llm").join("gemma-4-E4B-it"))
}

/// Path passed to `write_uqff()`. mistralrs derives shard names from the stem.
#[cfg(feature = "local-llm")]
fn uqff_write_path() -> Option<PathBuf> {
    uqff_dir().map(|d| d.join(UQFF_STEM))
}

/// Path to the first shard. Used both to detect cache hits and to hand to the
/// UQFF reader (which auto-discovers the remaining shards).
#[cfg(feature = "local-llm")]
fn uqff_first_shard_path() -> Option<PathBuf> {
    uqff_dir().map(|d| d.join(UQFF_FIRST_SHARD))
}

/// `true` if the UQFF cache is ready to load directly (skip ISQ entirely).
#[cfg(feature = "local-llm")]
pub fn uqff_cached() -> bool {
    uqff_first_shard_path()
        .map(|p| p.exists())
        .unwrap_or(false)
}

/// `true` if the raw HF model is already in `~/.cache/huggingface/hub/`.
/// We check by snapshot-dir presence rather than reading any specific file —
/// HF cache layout has multiple revisions and we just want a yes/no.
#[cfg(feature = "local-llm")]
fn hf_weights_cached() -> bool {
    let Some(home) = dirs_next::home_dir() else { return false };
    let snapshots = home
        .join(".cache")
        .join("huggingface")
        .join("hub")
        .join("models--google--gemma-4-E4B-it")
        .join("snapshots");
    let Ok(mut iter) = std::fs::read_dir(&snapshots) else { return false };
    iter.next().is_some()
}

/// Pre-quantize Gemma 4 to Q4K and write it to the UQFF cache so subsequent
/// loads are fast. Idempotent: returns Ok immediately if the cache already
/// exists. Called by `mvd setup`.
///
/// We go directly through `MultimodalModelBuilder` rather than the
/// auto-detecting `ModelBuilder` because mistralrs 0.8.1's auto-detector
/// misroutes `Gemma4ForConditionalGeneration` to the text-only `-CausalLM`
/// pipeline, which then errors with "Unsupported model class". Routing
/// through `MultimodalLoaderType::Gemma4` explicitly avoids that.
#[cfg(feature = "local-llm")]
pub fn warm_uqff_cache() -> Result<()> {
    use mistralrs::{IsqType, MultimodalLoaderType, MultimodalModelBuilder};

    let path = uqff_write_path()
        .ok_or_else(|| anyhow::anyhow!("could not determine UQFF cache path"))?;

    if uqff_cached() {
        eprintln!(
            "  ✓ UQFF cache already present at {}",
            path.parent().map(|p| p.display().to_string()).unwrap_or_default()
        );
        return Ok(());
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    eprintln!(
        "  Quantizing Gemma 4 (Q4K) and writing UQFF cache to {} ...",
        path.parent().map(|p| p.display().to_string()).unwrap_or_default()
    );
    eprintln!("  This is a one-time cost (~1–2 min); future loads will be seconds.");

    let rt = runtime();
    rt.block_on(async {
        MultimodalModelBuilder::new(MODEL_ID)
            .with_loader_type(MultimodalLoaderType::Gemma4)
            .with_isq(IsqType::Q4K)
            .write_uqff(path.clone())
            .build()
            .await
    })
    .map_err(|e| anyhow::anyhow!("UQFF warm failed: {e}"))?;

    eprintln!("  ✅ UQFF cache written.");
    Ok(())
}

/// No-op fallback when local-llm is not compiled in.
#[cfg(not(feature = "local-llm"))]
pub fn warm_uqff_cache() -> Result<()> {
    Ok(())
}

/// Eagerly load the model so the user sees load progress *before* the chat
/// prompt loop starts (instead of staring at a frozen prompt on first message).
#[cfg(feature = "local-llm")]
pub fn ensure_loaded() -> Result<()> {
    get_or_init_model().map(|_| ())
}

#[cfg(not(feature = "local-llm"))]
pub fn ensure_loaded() -> Result<()> {
    Ok(())
}

/// Non-streaming chat completion. Internally streams and collects the full
/// response so callers that want the whole string still get one.
#[cfg(feature = "local-llm")]
pub fn llm_chat(system_prompt: &str, user_prompt: &str) -> Result<String> {
    llm_chat_streaming(system_prompt, user_prompt, |_| {})
}

/// Streaming chat completion. `on_token` is invoked for each generated text
/// chunk as it arrives, so the caller can print it live. The full assembled
/// response is also returned for history / JSON output.
#[cfg(feature = "local-llm")]
pub fn llm_chat_streaming<F: FnMut(&str)>(
    system_prompt: &str,
    user_prompt: &str,
    mut on_token: F,
) -> Result<String> {
    use mistralrs::{
        ChatCompletionChunkResponse, ChunkChoice, Delta, RequestBuilder, Response,
        TextMessageRole,
    };

    let model = get_or_init_model()?;
    let rt = runtime();

    let request = RequestBuilder::new()
        .add_message(TextMessageRole::System, system_prompt)
        .add_message(TextMessageRole::User, user_prompt)
        .set_sampler_max_len(DEFAULT_MAX_TOKENS);

    let collected = rt.block_on(async move {
        let mut stream = model
            .stream_chat_request(request)
            .await
            .map_err(|e| anyhow::anyhow!("stream_chat_request failed: {e}"))?;

        let mut full = String::new();
        while let Some(chunk) = stream.next().await {
            match chunk {
                Response::Chunk(ChatCompletionChunkResponse { choices, .. }) => {
                    if let Some(ChunkChoice {
                        delta:
                            Delta {
                                content: Some(content),
                                ..
                            },
                        ..
                    }) = choices.first()
                    {
                        on_token(content);
                        full.push_str(content);
                    }
                }
                Response::Done(_) | Response::CompletionDone(_) => break,
                Response::ModelError(err, _) => {
                    return Err(anyhow::anyhow!("LLM model error: {err}"));
                }
                Response::ValidationError(err) | Response::InternalError(err) => {
                    return Err(anyhow::anyhow!("LLM error: {err}"));
                }
                _ => {}
            }
        }
        Ok::<_, anyhow::Error>(full)
    })?;

    Ok(collected)
}

#[cfg(feature = "local-llm")]
fn runtime() -> &'static tokio::runtime::Runtime {
    use std::sync::OnceLock;
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    RT.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("failed to create tokio runtime")
    })
}

/// Lazily initialize the model and return a `&'static Model`.
///
/// We store the load result so a failure on first call is observable and
/// recoverable instead of panicking. The model itself is not behind a Mutex —
/// `Model::send_chat_request` / `stream_chat_request` take `&self` and the
/// underlying engine is internally `Arc`-shared.
///
/// Three load paths, fastest first:
///   1. **UQFF cache present** — load pre-quantized shards, no ISQ pass (~5 s)
///   2. **HF cache present, no UQFF** — quantize-on-load (~30–120 s) and
///      hint that running `mvd setup` would make this fast next time
///   3. **Nothing cached** — first-run download (~15 GB) + quantize
#[cfg(feature = "local-llm")]
fn get_or_init_model() -> Result<&'static mistralrs::Model> {
    use mistralrs::{
        IsqType, MultimodalLoaderType, MultimodalModelBuilder, UqffMultimodalModelBuilder,
    };
    use std::sync::OnceLock;

    static MODEL: OnceLock<std::result::Result<mistralrs::Model, String>> = OnceLock::new();

    let entry = MODEL.get_or_init(|| {
        let uqff = uqff_first_shard_path().filter(|p| p.exists());
        let hf_present = hf_weights_cached();

        let rt = runtime();
        let result = if let Some(shard0) = uqff {
            eprintln!(
                "Loading local LLM ({MODEL_ID}, Q4K) from UQFF cache at {} ...",
                shard0.parent().map(|p| p.display().to_string()).unwrap_or_default()
            );
            rt.block_on(async {
                UqffMultimodalModelBuilder::new(MODEL_ID, vec![shard0])
                    .build()
                    .await
            })
        } else if hf_present {
            eprintln!(
                "Loading local LLM ({MODEL_ID}, Q4K) from Hugging Face cache. \
                 Quantizing in-place (~30–120 s)..."
            );
            eprintln!(
                "  Tip: run `mvd setup` once to pre-quantize and skip this on future loads."
            );
            rt.block_on(async {
                MultimodalModelBuilder::new(MODEL_ID)
                    .with_loader_type(MultimodalLoaderType::Gemma4)
                    .with_isq(IsqType::Q4K)
                    .build()
                    .await
            })
        } else {
            eprintln!(
                "Loading local LLM ({MODEL_ID}, Q4K). First run: downloading ~15 GB then quantizing — \
                 this can take many minutes."
            );
            eprintln!(
                "  Tip: `mvd setup` will do this once and cache the quantized form."
            );
            rt.block_on(async {
                MultimodalModelBuilder::new(MODEL_ID)
                    .with_loader_type(MultimodalLoaderType::Gemma4)
                    .with_isq(IsqType::Q4K)
                    .build()
                    .await
            })
        };

        match result {
            Ok(m) => {
                eprintln!("Local LLM ready.");
                Ok(m)
            }
            Err(e) => Err(format!("{e}")),
        }
    });

    match entry {
        Ok(m) => Ok(m),
        Err(e) => Err(anyhow::anyhow!("failed to load local LLM: {e}")),
    }
}

/// Fallback when `local-llm` feature is not enabled.
#[cfg(not(feature = "local-llm"))]
pub fn llm_chat(_system_prompt: &str, _user_prompt: &str) -> Result<String> {
    anyhow::bail!(
        "Local LLM not available. Rebuild with --features local-llm to enable.\n\
         This requires the mistralrs crate and will embed google/gemma-4-E4B-it."
    )
}

#[cfg(not(feature = "local-llm"))]
pub fn llm_chat_streaming<F: FnMut(&str)>(
    _system_prompt: &str,
    _user_prompt: &str,
    _on_token: F,
) -> Result<String> {
    llm_chat(_system_prompt, _user_prompt)
}
