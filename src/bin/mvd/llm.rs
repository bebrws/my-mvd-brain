//! Local LLM inference for mvd CLI using mistral.rs.
//!
//! Runs google/gemma-4-E4B-it locally via the mistralrs SDK.
//! Gemma 4 is a multimodal model, so we use MultimodalModelBuilder.
//! No external API calls or API keys required.

use anyhow::Result;

/// Send a chat completion to the local Gemma model.
///
/// The model is loaded lazily on first call and cached for subsequent calls.
/// Uses Q4K in-situ quantization for reasonable memory usage.
///
/// # Arguments
/// * `system_prompt` - System instructions for the model
/// * `user_prompt` - User message to respond to
///
/// # Returns
/// The model's text response
#[cfg(feature = "local-llm")]
pub fn llm_chat(system_prompt: &str, user_prompt: &str) -> Result<String> {
    use std::sync::{Mutex, OnceLock};
    use mistralrs::{
        IsqType, MultimodalModelBuilder, MultimodalMessages, TextMessageRole, Model,
    };

    // Lazy-initialize a shared tokio runtime
    static RT: OnceLock<tokio::runtime::Runtime> = OnceLock::new();
    let rt = RT.get_or_init(|| {
        tokio::runtime::Runtime::new().expect("Failed to create tokio runtime")
    });

    // Lazy-initialize the model (downloads on first use if not cached).
    // Gemma 4 is a multimodal (conditional generation) model, so it requires
    // MultimodalModelBuilder even when used for text-only inference.
    static MODEL: OnceLock<Mutex<Model>> = OnceLock::new();

    let model_mutex = MODEL.get_or_init(|| {
        eprintln!("Loading local LLM (google/gemma-4-E4B-it with Q4K quantization)...");
        eprintln!("This may take a moment on first run while the model loads.");
        let m = rt.block_on(async {
            MultimodalModelBuilder::new("google/gemma-4-E4B-it")
                .with_isq(IsqType::Q4K)
                .with_logging()
                .build()
                .await
        }).expect("Failed to load local LLM model");
        eprintln!("✅ Local LLM loaded.");
        Mutex::new(m)
    });

    let model = model_mutex.lock()
        .map_err(|e| anyhow::anyhow!("Failed to lock LLM model: {e}"))?;

    // Build the chat request using MultimodalMessages with text-only messages.
    // MultimodalMessages supports add_message() for plain text alongside
    // multimodal content methods.
    let messages = MultimodalMessages::new()
        .add_message(TextMessageRole::System, system_prompt)
        .add_message(TextMessageRole::User, user_prompt);

    // Send the request and get the response
    let response = rt.block_on(async {
        model.send_chat_request(messages).await
    })?;

    let content = response.choices.first()
        .and_then(|c| c.message.content.as_ref())
        .map(|s| s.to_string())
        .unwrap_or_default();

    Ok(content)
}

/// Fallback when local-llm feature is not enabled
#[cfg(not(feature = "local-llm"))]
pub fn llm_chat(_system_prompt: &str, _user_prompt: &str) -> Result<String> {
    anyhow::bail!(
        "Local LLM not available. Rebuild with --features local-llm to enable.\n\
         This requires the mistralrs crate and will embed google/gemma-4-E4B-it."
    )
}
