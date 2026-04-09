//! LLM provider abstraction for mvd CLI.
//!
//! Supports OpenAI, Anthropic (Claude), Groq, and Google Gemini.
//! Each provider reads its API key from the standard environment variable.

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// Which LLM provider to use for synthesis / enrichment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LlmProvider {
    /// OpenAI (GPT-4o by default)
    OpenAI,
    /// Anthropic Claude (claude-3-5-sonnet)
    Claude,
    /// Groq (llama-3.3-70b-versatile)
    Groq,
    /// Google Gemini (gemini-2.0-flash)
    Gemini,
}

impl LlmProvider {
    /// Parse a provider name from a CLI flag value.
    pub fn from_str(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "openai" | "gpt4" | "gpt-4" | "gpt" => Ok(Self::OpenAI),
            "claude" | "anthropic" => Ok(Self::Claude),
            "groq" | "llama" => Ok(Self::Groq),
            "gemini" | "google" => Ok(Self::Gemini),
            other => bail!(
                "Unknown LLM provider: '{other}'. \
                 Supported: openai, claude, groq, gemini"
            ),
        }
    }

    /// The environment variable that holds the API key for this provider.
    pub fn api_key_env(&self) -> &'static str {
        match self {
            Self::OpenAI => "OPENAI_API_KEY",
            Self::Claude => "ANTHROPIC_API_KEY",
            Self::Groq => "GROQ_API_KEY",
            Self::Gemini => "GOOGLE_API_KEY",
        }
    }

    /// Default model identifier.
    fn default_model(&self) -> &'static str {
        match self {
            Self::OpenAI => "gpt-4o",
            Self::Claude => "claude-3-5-sonnet-20241022",
            Self::Groq => "llama-3.3-70b-versatile",
            Self::Gemini => "gemini-2.0-flash",
        }
    }

    /// API base URL.
    fn base_url(&self) -> &'static str {
        match self {
            Self::OpenAI => "https://api.openai.com/v1/chat/completions",
            Self::Claude => "https://api.anthropic.com/v1/messages",
            Self::Groq => "https://api.groq.com/openai/v1/chat/completions",
            Self::Gemini => "https://generativelanguage.googleapis.com/v1beta/models",
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::OpenAI => "OpenAI",
            Self::Claude => "Anthropic Claude",
            Self::Groq => "Groq",
            Self::Gemini => "Google Gemini",
        }
    }
}

/// Send a prompt to an LLM provider and return the generated text.
pub fn llm_chat(provider: LlmProvider, system: &str, user: &str) -> Result<String> {
    let api_key = std::env::var(provider.api_key_env()).with_context(|| {
        format!(
            "{} environment variable not set. Required for {} provider.",
            provider.api_key_env(),
            provider.label()
        )
    })?;

    if api_key.is_empty() {
        bail!("{} is empty", provider.api_key_env());
    }

    match provider {
        LlmProvider::OpenAI | LlmProvider::Groq => openai_compatible_chat(provider, &api_key, system, user),
        LlmProvider::Claude => anthropic_chat(&api_key, system, user),
        LlmProvider::Gemini => gemini_chat(&api_key, system, user),
    }
}

// ---------------------------------------------------------------------------
// OpenAI-compatible (OpenAI + Groq both use the same format)
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct OpenAIChatRequest<'a> {
    model: &'a str,
    messages: Vec<OpenAIMessage<'a>>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Serialize)]
struct OpenAIMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct OpenAIChatResponse {
    choices: Vec<OpenAIChoice>,
}

#[derive(Deserialize)]
struct OpenAIChoice {
    message: OpenAIResponseMessage,
}

#[derive(Deserialize)]
struct OpenAIResponseMessage {
    content: String,
}

fn openai_compatible_chat(
    provider: LlmProvider,
    api_key: &str,
    system: &str,
    user: &str,
) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .context("Failed to build HTTP client")?;

    let body = OpenAIChatRequest {
        model: provider.default_model(),
        messages: vec![
            OpenAIMessage { role: "system", content: system },
            OpenAIMessage { role: "user", content: user },
        ],
        temperature: 0.3,
        max_tokens: 2048,
    };

    let resp = client
        .post(provider.base_url())
        .header("Authorization", format!("Bearer {api_key}"))
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .with_context(|| format!("Request to {} failed", provider.label()))?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().unwrap_or_default();
        bail!("{} API error ({}): {}", provider.label(), status, text);
    }

    let parsed: OpenAIChatResponse = resp.json().context("Failed to parse response")?;
    parsed
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .ok_or_else(|| anyhow::anyhow!("No response from {}", provider.label()))
}

// ---------------------------------------------------------------------------
// Anthropic Claude
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct AnthropicRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    system: &'a str,
    messages: Vec<AnthropicMessage<'a>>,
}

#[derive(Serialize)]
struct AnthropicMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<AnthropicContent>,
}

#[derive(Deserialize)]
struct AnthropicContent {
    text: String,
}

fn anthropic_chat(api_key: &str, system: &str, user: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .context("Failed to build HTTP client")?;

    let body = AnthropicRequest {
        model: LlmProvider::Claude.default_model(),
        max_tokens: 2048,
        system,
        messages: vec![AnthropicMessage { role: "user", content: user }],
    };

    let resp = client
        .post(LlmProvider::Claude.base_url())
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .context("Request to Anthropic failed")?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().unwrap_or_default();
        bail!("Anthropic API error ({}): {}", status, text);
    }

    let parsed: AnthropicResponse = resp.json().context("Failed to parse Anthropic response")?;
    parsed
        .content
        .into_iter()
        .next()
        .map(|c| c.text)
        .ok_or_else(|| anyhow::anyhow!("No response from Anthropic"))
}

// ---------------------------------------------------------------------------
// Google Gemini
// ---------------------------------------------------------------------------

#[derive(Serialize)]
struct GeminiRequest<'a> {
    contents: Vec<GeminiContent<'a>>,
    #[serde(rename = "systemInstruction", skip_serializing_if = "Option::is_none")]
    system_instruction: Option<GeminiContent<'a>>,
}

#[derive(Serialize)]
struct GeminiContent<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    role: Option<&'a str>,
    parts: Vec<GeminiPart<'a>>,
}

#[derive(Serialize)]
struct GeminiPart<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiResponseContent,
}

#[derive(Deserialize)]
struct GeminiResponseContent {
    parts: Vec<GeminiResponsePart>,
}

#[derive(Deserialize)]
struct GeminiResponsePart {
    text: String,
}

fn gemini_chat(api_key: &str, system: &str, user: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()
        .context("Failed to build HTTP client")?;

    let model = LlmProvider::Gemini.default_model();
    let url = format!(
        "{}/{}:generateContent?key={}",
        LlmProvider::Gemini.base_url(),
        model,
        api_key
    );

    let body = GeminiRequest {
        contents: vec![GeminiContent {
            role: Some("user"),
            parts: vec![GeminiPart { text: user }],
        }],
        system_instruction: Some(GeminiContent {
            role: None,
            parts: vec![GeminiPart { text: system }],
        }),
    };

    let resp = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .context("Request to Gemini failed")?;

    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().unwrap_or_default();
        bail!("Gemini API error ({}): {}", status, text);
    }

    let parsed: GeminiResponse = resp.json().context("Failed to parse Gemini response")?;
    parsed
        .candidates
        .into_iter()
        .next()
        .and_then(|c| c.content.parts.into_iter().next())
        .map(|p| p.text)
        .ok_or_else(|| anyhow::anyhow!("No response from Gemini"))
}
