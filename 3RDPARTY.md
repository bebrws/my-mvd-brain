# Third-Party Services & Embedding Systems Audit

This document catalogs every external API call, embedding system, and network-dependent component in the memvid codebase, along with guidance on replacing each with a fully local alternative.

---

## Table of Contents

1. [Network / API Usage](#1-network--api-usage)
2. [Embedding Systems](#2-embedding-systems)
3. [Trait Interfaces](#3-trait-interfaces)
4. [Storage Backends](#4-storage-backends)
5. [Local Replacement Guide](#5-local-replacement-guide)

---

## 1. Network / API Usage

### 1.1 OpenAI Embeddings API

| Field | Detail |
|---|---|
| **File** | `src/api_embed.rs` |
| **Feature gate** | `api_embed` |
| **Endpoint** | `https://api.openai.com/v1/embeddings` |
| **Auth** | `OPENAI_API_KEY` env var, Bearer token |
| **Models** | `text-embedding-3-small` (1536d), `text-embedding-3-large` (3072d), `text-embedding-ada-002` (1536d) |
| **Used by** | Any code that creates an `OpenAIEmbedder` — currently only integration tests and the `openai_embedding` example |
| **HTTP client** | `reqwest::blocking::Client` with 30s timeout, exponential backoff on 429s |
| **Data sent** | Text strings to be embedded |
| **Data received** | Float32 embedding vectors |

**How it works:** The `OpenAIEmbedder` struct implements `EmbeddingProvider`. On each call to `embed_text()`, it POSTs a JSON payload `{model, input, encoding_format: "float"}` to the OpenAI API. Batch support splits input into chunks of `max_batch_size` (2048). Rate-limit 429 responses trigger exponential backoff up to `max_retries` (3).

### 1.2 OpenAI / Groq Chat Completions

| Field | Detail |
|---|---|
| **File** | `src/bin/mvd/llm.rs` (lines 132–174) |
| **Feature gate** | CLI binary (requires `reqwest` via `api_embed`) |
| **Endpoints** | OpenAI: `https://api.openai.com/v1/chat/completions`, Groq: `https://api.groq.com/openai/v1/chat/completions` |
| **Auth** | `OPENAI_API_KEY` or `GROQ_API_KEY`, Bearer token |
| **Models** | OpenAI: `gpt-4o`, Groq: `llama-3.3-70b-versatile` |
| **Used by** | `mvd ask --use-model openai/groq`, `mvd enrich --engine openai/groq` |
| **Data sent** | System prompt + user prompt (frame text for enrichment, question + context for ask) |
| **Data received** | Generated text (facts JSON for enrich, answer text for ask) |

**How it works:** The `openai_compatible_chat()` function sends a standard OpenAI-format chat completion request with `temperature: 0.3`, `max_tokens: 2048`. Both OpenAI and Groq use the identical request format. The 120s timeout accommodates long generations.

### 1.3 Anthropic Claude Chat

| Field | Detail |
|---|---|
| **File** | `src/bin/mvd/llm.rs` (lines 204–239) |
| **Feature gate** | CLI binary |
| **Endpoint** | `https://api.anthropic.com/v1/messages` |
| **Auth** | `ANTHROPIC_API_KEY`, `x-api-key` header |
| **Model** | `claude-3-5-sonnet-20241022` |
| **Used by** | `mvd ask --use-model claude`, `mvd enrich --engine claude` |
| **Data sent** | System prompt + user messages |
| **Data received** | Generated text |

**How it works:** Uses Anthropic's Messages API with `anthropic-version: 2023-06-01`. The request format differs from OpenAI — system prompt is a top-level field, not a message. Same 120s timeout.

### 1.4 Google Gemini Chat

| Field | Detail |
|---|---|
| **File** | `src/bin/mvd/llm.rs` (lines 284–330) |
| **Feature gate** | CLI binary |
| **Endpoint** | `https://generativelanguage.googleapis.com/v1beta/models/{model}:generateContent?key={key}` |
| **Auth** | `GOOGLE_API_KEY`, passed as URL query parameter |
| **Model** | `gemini-2.0-flash` |
| **Used by** | `mvd ask --use-model gemini`, `mvd enrich --engine gemini` |
| **Data sent** | System instruction + user content |
| **Data received** | Generated text |

**How it works:** Uses Gemini's `generateContent` endpoint. The API key is in the URL (not a header). System instruction uses the `systemInstruction` field. Same 120s timeout.

### 1.5 HuggingFace Hub (Whisper Model Download)

| Field | Detail |
|---|---|
| **File** | `src/whisper.rs` (lines 515–637) |
| **Feature gate** | `whisper` |
| **Service** | HuggingFace Hub API via `hf_hub` crate |
| **Repos accessed** | `openai/whisper-small.en`, `openai/whisper-tiny.en`, `openai/whisper-tiny`, `lmz/candle-whisper` |
| **Files downloaded** | `config.json`, `tokenizer.json`, `model.safetensors` or `model-tiny-*.gguf` |
| **Auth** | None required (public models), optional `HF_TOKEN` |
| **Used by** | `WhisperTranscriber::new()` — auto-downloads on first use |

**How it works:** The `hf_hub::api::sync::Api` client downloads model files to `~/.cache/huggingface/hub/`. For quantized models, config/tokenizer come from the base OpenAI repo while weights come from `lmz/candle-whisper`. Files are cached after first download.

### 1.6 Text Embedding Model URLs (Currently Manual Download)

| Field | Detail |
|---|---|
| **File** | `src/text_embed.rs` (lines 171–208) |
| **Feature gate** | `vec` |
| **URLs defined** | 4 models from HuggingFace: `BAAI/bge-small-en-v1.5`, `BAAI/bge-base-en-v1.5`, `nomic-ai/nomic-embed-text-v1.5`, `thenlper/gte-large` |
| **Files needed** | `model.onnx` + `tokenizer.json` per model |
| **Current behavior** | Returns error with manual `curl` instructions if files missing |
| **Stored at** | `~/.cache/memvid/text-models/` |

### 1.7 CLIP Model URLs (Currently Manual Download)

| Field | Detail |
|---|---|
| **File** | `src/clip.rs` (lines 171–214) |
| **Feature gate** | `clip` |
| **URLs defined** | `mobileclip-s2` (vision + text ONNX + tokenizer), `siglip-base` (vision + text ONNX + tokenizer) |
| **Current behavior** | Returns error with manual `curl` instructions if files missing |
| **Stored at** | `~/.memvid/models/` (or `MEMVID_MODELS_DIR` env) |

### 1.8 NER Model URLs (Currently Manual Download)

| Field | Detail |
|---|---|
| **File** | `src/analysis/ner.rs` (lines 31–36) |
| **Feature gate** | `logic_mesh` |
| **URLs defined** | `dslim/distilbert-NER` — `model.onnx` (~261 MB) + `tokenizer.json` |
| **Current behavior** | Must be manually downloaded before use |
| **Stored at** | `{models_dir}/distilbert-ner/` |

---

## 2. Embedding Systems

### 2.1 Local Text Embedding (ONNX Runtime)

| Field | Detail |
|---|---|
| **File** | `src/text_embed.rs` |
| **Feature** | `vec` |
| **Runtime** | ONNX Runtime via `ort` crate |
| **Tokenizer** | HuggingFace `tokenizers` crate |
| **Architecture** | BERT-style encoder models |
| **Pooling** | CLS token (first token of `last_hidden_state`) |
| **Normalization** | L2 normalized to unit length |
| **Max sequence** | 512 tokens |
| **Caching** | LRU cache (default 1000 entries) with hash-based keys |
| **Lifecycle** | Lazy-loaded, auto-unloaded after 5min idle |

**Available models:**

| Model | Dimensions | Size | Quality |
|---|---|---|---|
| `bge-small-en-v1.5` | 384 | ~33 MB | Good, fast |
| `bge-base-en-v1.5` | 768 | ~110 MB | Better |
| `nomic-embed-text-v1.5` | 768 | ~137 MB | Versatile |
| `gte-large` | 1024 | ~335 MB | Highest quality |

**Data flow:** Text → tokenizer (pad to 512, truncate) → ONNX session (input_ids, attention_mask, token_type_ids) → CLS token embedding → L2 normalize → cache → return.

### 2.2 OpenAI Cloud Embedding

| Field | Detail |
|---|---|
| **File** | `src/api_embed.rs` |
| **Feature** | `api_embed` |
| **Transport** | HTTPS REST API via `reqwest` |
| **Auth** | API key from environment variable |
| **Batch support** | Up to 2048 texts per request |
| **Retry logic** | Exponential backoff on 429 (rate limit) |

**This is the only cloud-based embedding system.** All others are local ONNX.

### 2.3 CLIP Multimodal Embedding (ONNX Runtime)

| Field | Detail |
|---|---|
| **File** | `src/clip.rs` |
| **Feature** | `clip` (depends on `vec`) |
| **Runtime** | ONNX Runtime via `ort` |
| **Capabilities** | Image → embedding, Text → embedding (shared space) |
| **Image preprocessing** | Resize shortest edge → center crop to model resolution → normalize [0,1] → NCHW format |
| **Text tokenizer** | BPE tokenizer, max 77 tokens |

**Available models:**

| Model | Dims | Resolution | Notes |
|---|---|---|---|
| `mobileclip-s2` | 512 | 256×256 | Default, fast |
| `siglip-base` | 768 | 224×224 | Better quality |

**Data flow (image):** Image → resize/crop → normalize → NCHW tensor → vision ONNX → L2 normalize.
**Data flow (text):** Text → BPE tokenize (pad/truncate to 77) → text ONNX → L2 normalize.

### 2.4 NER Entity Extraction (ONNX Runtime)

| Field | Detail |
|---|---|
| **File** | `src/analysis/ner.rs` |
| **Feature** | `logic_mesh` |
| **Model** | DistilBERT-NER (~261 MB, 92% F1 on CoNLL-03) |
| **Output** | Per-token entity labels (PER, ORG, LOC, MISC) with confidence scores |
| **Runtime** | ONNX Runtime via `ort` |

**Not an embedding system per se** — it produces entity labels, not vectors. But it uses the same ONNX + tokenizer infrastructure.

### 2.5 Whisper Audio Transcription (Candle)

| Field | Detail |
|---|---|
| **File** | `src/whisper.rs` |
| **Feature** | `whisper` |
| **Runtime** | Candle ML framework (not ONNX) |
| **GPU support** | Metal (macOS), CUDA (NVIDIA), CPU fallback |
| **Audio input** | Any format via Symphonia → resampled to 16kHz mono |
| **Output** | Transcribed text with timestamps |

**Available models:**

| Model | Format | Size | Notes |
|---|---|---|---|
| `whisper-small-en` | SafeTensors (FP32) | ~461 MB | Default, good quality |
| `whisper-tiny-en` | SafeTensors (FP32) | ~75 MB | Faster, lower quality |
| `whisper-tiny-en-q8k` | GGUF (Q8) | ~19 MB | Quantized, very fast |
| `whisper-tiny-q8k` | GGUF (Q8) | ~19 MB | Multilingual quantized |

**Not an embedding system** — it produces text transcriptions. Uses Candle framework with HuggingFace Hub auto-download.

---

## 3. Trait Interfaces

### 3.1 `VecEmbedder` trait

**Location:** `src/types/ask.rs` line 159

```rust
pub trait VecEmbedder {
    fn embed_query(&self, text: &str) -> Result<Vec<f32>>;
    fn embed_chunks(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn embedding_dimension(&self) -> usize;
}
```

**Used by:** `Memvid::search()`, `EmbeddingBatcher`, `Memvid::audit()`, `TableStorage`. This is the core trait for semantic search — any embedder passed to the memory system must implement this.

### 3.2 `EmbeddingProvider` trait

**Location:** `src/types/embedding.rs` line 140

```rust
pub trait EmbeddingProvider: Send + Sync {
    fn kind(&self) -> &str;        // "local", "openai", etc.
    fn model(&self) -> &str;       // model identifier
    fn dimension(&self) -> usize;  // embedding dimension
    fn embed_text(&self, text: &str) -> Result<Vec<f32>>;
    fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;
    fn is_ready(&self) -> bool;
    fn init(&mut self) -> Result<()>;
}
```

**Implemented by:** `LocalTextEmbedder` (kind="local"), `OpenAIEmbedder` (kind="openai"). This is the newer, richer interface that supports provider identification.

### 3.3 `ClipEmbeddingProvider` trait

**Location:** `src/clip.rs` line 1508

```rust
pub trait ClipEmbeddingProvider: Send + Sync {
    fn encode_image_bytes(&self, bytes: &[u8]) -> Result<Vec<f32>>;
    fn encode_text(&self, text: &str) -> Result<Vec<f32>>;
    fn dims(&self) -> u32;
    fn model_name(&self) -> &str;
}
```

**Implemented by:** `ClipModel`. Specialized for multimodal image+text embedding in a shared vector space.

---

## 4. Storage Backends

### 4.1 `VecIndex` — Text Embedding Storage

**Location:** `src/vec.rs`

Three storage modes:

| Mode | Condition | Search | Memory |
|---|---|---|---|
| **Uncompressed** | < 1000 vectors | Brute-force L2 | Full f32 vectors in memory |
| **HNSW** | ≥ 1000 vectors | Approximate nearest neighbor (ef=50) | Graph structure + vectors |
| **Product Quantization** | Large indices | Approximate (quantized distances) | Compressed codes |

### 4.2 `ClipIndex` — CLIP Embedding Storage

**Location:** `src/clip.rs` line 297

Simple brute-force L2 search over `ClipDocument` entries. No HNSW or PQ — CLIP indices are typically small (one per page/image).

### 4.3 `QuantizedVecIndex` — Compressed Storage

**Location:** `src/vec_pq.rs`

Product Quantization (PQ) for large vector indices. Splits vectors into subspaces, quantizes each to nearest centroid. Trades accuracy for ~10-30x memory reduction.

---

## 5. Local Replacement Guide

### 5.1 Replacing OpenAI Embeddings → Already Done

The codebase already has `LocalTextEmbedder` using ONNX models. To use local embeddings instead of OpenAI:

- **Don't enable** the `api_embed` feature
- **Enable** the `vec` feature (enabled by default when using CLI)
- Use `TextEmbedConfig::gte_large()` for highest quality (1024d) or `TextEmbedConfig::bge_small()` for fastest (384d)
- Download models: `curl -L 'https://huggingface.co/thenlper/gte-large/resolve/main/onnx/model.onnx' -o ~/.cache/memvid/text-models/gte-large.onnx`

**Quality comparison:**
- OpenAI `text-embedding-3-small` (1536d) ≈ GTE-large (1024d) for most retrieval tasks
- Local models are **free**, **private**, and work **offline**
- Latency: Local ONNX is typically 5-20ms per text vs 100-500ms for API calls

### 5.2 Replacing LLM Chat APIs → Local LLM Options

The `llm.rs` module supports OpenAI, Claude, Groq, and Gemini for two features:

1. **`mvd ask --use-model`** — RAG-style question answering over retrieved context
2. **`mvd enrich --engine`** — Entity/fact extraction from frame text

**To replace with local LLMs:**

| Approach | How | Effort |
|---|---|---|
| **Ollama** | Run `ollama serve`, point `base_url` to `http://localhost:11434/v1` with OpenAI-compatible API | Low — modify `LlmProvider` to add `Local` variant |
| **llama.cpp server** | Run `llama-server`, uses OpenAI-compatible API at `http://localhost:8080/v1` | Low — same approach as Ollama |
| **vLLM** | Run `vllm serve`, OpenAI-compatible at `http://localhost:8000/v1` | Low — same approach |
| **Candle (in-process)** | Add a Candle-based LLM (like the Whisper module does) | High — significant code addition |

**Recommended approach:** Add a `Local` variant to `LlmProvider` that sends requests to `http://localhost:11434/v1` (Ollama default). Since Ollama uses OpenAI-compatible chat format, the existing `openai_compatible_chat()` function works with zero changes — only the URL and auth need modification.

```rust
// Proposed addition to LlmProvider:
Self::Local => "http://localhost:11434/v1/chat/completions",
// No API key needed for local
```

### 5.3 Replacing HuggingFace Hub Downloads → Pre-bundled Models

The Whisper module uses `hf_hub` for auto-download. Options:

| Approach | Pros | Cons |
|---|---|---|
| **Keep as-is** | Models cached after first download, works offline after | Requires internet on first run |
| **Pre-download script** | `make download-models` already exists in Makefile | User must run manually |
| **Bundle in binary** | Include model weights in the binary | Huge binary size (hundreds of MB) |
| **System package** | Distribute models as OS packages | Platform-specific packaging |

**Current state:** Whisper is already fully local after first download. The `hf_hub` crate caches files in `~/.cache/huggingface/hub/`. Set `MEMVID_OFFLINE=1` to prevent any network access.

### 5.4 CLIP — Already Fully Local

CLIP models are ONNX-based and run locally. No cloud API involved. Just need to download the model files once:

```bash
mkdir -p ~/.memvid/models
# MobileCLIP-S2 (default)
curl -L 'https://huggingface.co/memvid/mobileclip-s2-onnx/resolve/main/mobileclip-s2_vision.onnx' \
  -o ~/.memvid/models/mobileclip-s2_vision.onnx
curl -L 'https://huggingface.co/memvid/mobileclip-s2-onnx/resolve/main/mobileclip-s2_text.onnx' \
  -o ~/.memvid/models/mobileclip-s2_text.onnx
```

### 5.5 NER — Already Fully Local

DistilBERT-NER runs locally via ONNX Runtime. Download once:

```bash
mkdir -p ~/.memvid/models/distilbert-ner
curl -L 'https://huggingface.co/dslim/distilbert-NER/resolve/main/onnx/model.onnx' \
  -o ~/.memvid/models/distilbert-ner/model.onnx
curl -L 'https://huggingface.co/dslim/distilbert-NER/resolve/main/tokenizer.json' \
  -o ~/.memvid/models/distilbert-ner/tokenizer.json
```

### 5.6 Summary: What Requires Internet

| Component | Requires Internet? | When? | Can Run Offline? |
|---|---|---|---|
| Text embeddings (ONNX) | Only first run | Model download (~33-335 MB) | ✅ Yes, after download |
| CLIP embeddings | Only first run | Model download (~50-150 MB) | ✅ Yes, after download |
| NER extraction | Only first run | Model download (~261 MB) | ✅ Yes, after download |
| Whisper transcription | Only first run | Model download (~19-461 MB) | ✅ Yes, after download |
| OpenAI embeddings | Every call | API request | ❌ No — use local ONNX instead |
| LLM chat (ask/enrich) | Every call | API request | ❌ No — use Ollama/llama.cpp instead |

**Bottom line:** The only components that truly require ongoing internet access are the OpenAI embedding API and the LLM chat providers. Both have drop-in local replacements. Everything else downloads once and runs offline forever.
