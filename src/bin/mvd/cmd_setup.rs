//! `mvd setup` — download all models and create the memory file for fully offline usage.
//!
//! This command downloads all required model files so that mvd can run
//! entirely offline after setup completes. No network access is needed
//! for any other mvd command after running `mvd setup`.

use anyhow::{Context, Result};
use clap::Args;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Args)]
pub struct SetupArgs {
    /// Skip downloading the LLM model (Gemma 4)
    #[arg(long)]
    pub skip_llm: bool,
    /// Skip downloading Whisper model
    #[arg(long)]
    pub skip_whisper: bool,
    /// Skip creating the memory file
    #[arg(long)]
    pub skip_create: bool,
}

pub fn run(args: SetupArgs) -> Result<()> {
    eprintln!("━━━ mvd setup ━━━");
    eprintln!("Downloading all models for fully offline usage.\n");

    let mut step = 1;
    let total_steps = 6
        - if args.skip_llm { 1 } else { 0 }
        - if args.skip_whisper { 1 } else { 0 }
        - if args.skip_create { 1 } else { 0 };

    // ── Step 1: Text Embedding (GTE-large) ──────────────────────────────
    eprintln!("[{step}/{total_steps}] Downloading text embedding model (gte-large, ~335 MB)...");
    download_text_embed_models()?;
    step += 1;

    // ── Step 2: CLIP (SigLIP-base) ──────────────────────────────────────
    eprintln!("[{step}/{total_steps}] Downloading CLIP model (siglip-base, ~210 MB)...");
    download_clip_models()?;
    step += 1;

    // ── Step 3: NER (DistilBERT-NER) ────────────────────────────────────
    eprintln!("[{step}/{total_steps}] Downloading NER model (distilbert-ner, ~261 MB)...");
    download_ner_models()?;
    step += 1;

    // ── Step 4: Whisper ─────────────────────────────────────────────────
    if !args.skip_whisper {
        eprintln!("[{step}/{total_steps}] Downloading Whisper model (whisper-small-en, ~244 MB)...");
        download_whisper_model()?;
        step += 1;
    }

    // ── Step 5: LLM (Gemma 4) ───────────────────────────────────────────
    if !args.skip_llm {
        eprintln!("[{step}/{total_steps}] Downloading LLM (google/gemma-4-E4B-it, ~2-3 GB)...");
        eprintln!("  This is the largest download and may take several minutes.");
        download_llm_model()?;
        step += 1;
    }

    // ── Step 6: Create memory file ──────────────────────────────────────
    if !args.skip_create {
        eprintln!("[{step}/{total_steps}] Creating memory file ~/mvd.mv2...");
        create_memory_file()?;
    }

    eprintln!("\n✅ Setup complete! mvd is ready to use fully offline.");
    eprintln!("   Memory file: ~/mvd.mv2");
    eprintln!("   All models cached locally — no network needed.");
    Ok(())
}

/// Download text embedding model (GTE-large ONNX + tokenizer)
fn download_text_embed_models() -> Result<()> {
    let models_dir = dirs_next::cache_dir()
        .map(|p| p.join("memvid").join("text-models"))
        .unwrap_or_else(|| PathBuf::from(".memvid-cache/text-models"));

    let model_info = memvid_core::get_text_model_info("gte-large");

    let model_path = models_dir.join("gte-large.onnx");
    if !model_path.exists() {
        download_file(model_info.model_url, &model_path, "gte-large.onnx")?;
    } else {
        eprintln!("  ✓ gte-large.onnx already downloaded");
    }

    let tok_path = models_dir.join("gte-large_tokenizer.json");
    if !tok_path.exists() {
        download_file(model_info.tokenizer_url, &tok_path, "gte-large_tokenizer.json")?;
    } else {
        eprintln!("  ✓ gte-large_tokenizer.json already downloaded");
    }

    Ok(())
}

/// Download CLIP model (SigLIP-base vision + text + tokenizer)
fn download_clip_models() -> Result<()> {
    let models_dir = std::env::var("MEMVID_MODELS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs_next::home_dir()
                .map(|h| h.join(".memvid").join("models"))
                .unwrap_or_else(|| PathBuf::from(".memvid/models"))
        });

    let model_info = memvid_core::get_model_info("siglip-base");

    let vision_path = models_dir.join("siglip-base_vision.onnx");
    if !vision_path.exists() {
        download_file(model_info.vision_url, &vision_path, "siglip-base_vision.onnx")?;
    } else {
        eprintln!("  ✓ siglip-base_vision.onnx already downloaded");
    }

    let text_path = models_dir.join("siglip-base_text.onnx");
    if !text_path.exists() {
        download_file(model_info.text_url, &text_path, "siglip-base_text.onnx")?;
    } else {
        eprintln!("  ✓ siglip-base_text.onnx already downloaded");
    }

    let tok_path = models_dir.join("siglip-base_tokenizer.json");
    if !tok_path.exists() {
        download_file(model_info.tokenizer_url, &tok_path, "siglip-base_tokenizer.json")?;
    } else {
        eprintln!("  ✓ siglip-base_tokenizer.json already downloaded");
    }

    Ok(())
}

/// Download NER model (DistilBERT-NER ONNX + tokenizer)
fn download_ner_models() -> Result<()> {
    let models_dir = std::env::var("MEMVID_MODELS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            dirs_next::home_dir()
                .map(|h| h.join(".memvid").join("models"))
                .unwrap_or_else(|| PathBuf::from(".memvid/models"))
        });
    let ner_dir = models_dir.join("distilbert-ner");

    let model_path = ner_dir.join("model.onnx");
    if !model_path.exists() {
        download_file(
            "https://huggingface.co/dslim/distilbert-NER/resolve/main/onnx/model.onnx",
            &model_path,
            "distilbert-ner model.onnx",
        )?;
    } else {
        eprintln!("  ✓ distilbert-ner/model.onnx already downloaded");
    }

    let tok_path = ner_dir.join("tokenizer.json");
    if !tok_path.exists() {
        download_file(
            "https://huggingface.co/dslim/distilbert-NER/resolve/main/tokenizer.json",
            &tok_path,
            "distilbert-ner tokenizer.json",
        )?;
    } else {
        eprintln!("  ✓ distilbert-ner/tokenizer.json already downloaded");
    }

    Ok(())
}

/// Download Whisper model by triggering the hf_hub download mechanism
fn download_whisper_model() -> Result<()> {
    // Whisper uses hf_hub which auto-downloads to ~/.cache/huggingface/hub/
    // We trigger the download by attempting to construct a WhisperTranscriber.
    // If the whisper feature isn't enabled, skip gracefully.
    #[cfg(feature = "whisper")]
    {
        let config = memvid_core::WhisperConfig::default();
        match memvid_core::WhisperTranscriber::new(&config) {
            Ok(_) => eprintln!("  ✅ Whisper model ready"),
            Err(e) => {
                eprintln!("  ⚠️  Whisper download failed: {e}");
                eprintln!("     You can retry later or use mvd without audio transcription.");
            }
        }
    }
    #[cfg(not(feature = "whisper"))]
    {
        eprintln!("  ⚠️  Whisper feature not enabled in this build. Skipping.");
    }
    Ok(())
}

/// Download the LLM model (Gemma 4 E4B) via mistralrs and pre-quantize it
/// to the UQFF cache so subsequent loads skip the ISQ pass.
fn download_llm_model() -> Result<()> {
    #[cfg(feature = "local-llm")]
    {
        // `warm_uqff_cache()` runs `ModelBuilder::new(...).with_isq(Q4K).write_uqff(...).build()`,
        // which:
        //   1. Downloads the bf16 weights into ~/.cache/huggingface/hub if missing.
        //   2. Performs the Q4K in-situ quantization pass.
        //   3. Serializes the quantized tensors into ~/.cache/memvid/llm/...
        // After this point every other LLM-using command loads from the UQFF
        // shards and skips the slow quantize step.
        match crate::llm::warm_uqff_cache() {
            Ok(_) => eprintln!("  ✅ LLM model ready (UQFF cache primed)"),
            Err(e) => {
                eprintln!("  ⚠️  LLM warm failed: {e}");
                eprintln!("     The ask and enrich --llm commands will still work but will");
                eprintln!("     quantize on every load. Rerun `mvd setup` once disk space /");
                eprintln!("     network is sorted to fix that.");
            }
        }
    }
    #[cfg(not(feature = "local-llm"))]
    {
        eprintln!("  ⚠️  local-llm feature not enabled in this build. Skipping.");
        eprintln!("     Rebuild with: cargo build --features local-llm");
    }
    Ok(())
}

/// Create the default memory file at ~/mvd.mv2
fn create_memory_file() -> Result<()> {
    let home = dirs_next::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;
    let mv2_path = home.join("mvd.mv2");

    if mv2_path.exists() {
        eprintln!("  ✓ {} already exists", mv2_path.display());
    } else {
        let mem = memvid_core::Memvid::create(&mv2_path)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .with_context(|| format!("Failed to create memory: {}", mv2_path.display()))?;
        let stats = mem.stats().map_err(|e| anyhow::anyhow!("{e}"))?;
        eprintln!("  ✅ Created {}", mv2_path.display());
        eprintln!("     Frames: {}, Size: {} bytes", stats.frame_count, stats.size_bytes);
    }
    Ok(())
}

/// Download a file from a URL with progress reporting
fn download_file(url: &str, dest: &Path, label: &str) -> Result<()> {
    // Create parent directories
    if let Some(parent) = dest.parent() {
        std::fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create directory: {}", parent.display()))?;
    }

    eprintln!("  Downloading {label}...");

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(600))
        .build()
        .context("Failed to create HTTP client")?;

    let response = client.get(url).send()
        .with_context(|| format!("Failed to download {url}"))?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed with status {}: {}", response.status(), url);
    }

    let total_size = response.content_length();
    if let Some(size) = total_size {
        eprintln!("  Size: {:.1} MB", size as f64 / 1_048_576.0);
    }

    // Download to temp file then rename
    let tmp_path = dest.with_extension("downloading");
    let mut file = std::fs::File::create(&tmp_path)
        .with_context(|| format!("Failed to create temp file: {}", tmp_path.display()))?;

    let bytes = response.bytes()
        .with_context(|| format!("Failed to read response for {label}"))?;

    file.write_all(&bytes)
        .with_context(|| format!("Failed to write {label}"))?;

    std::fs::rename(&tmp_path, dest)
        .with_context(|| format!("Failed to rename temp file to {}", dest.display()))?;

    eprintln!("  ✅ {label}");
    Ok(())
}
