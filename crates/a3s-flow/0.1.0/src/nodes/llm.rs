//! `"llm"` node — OpenAI-compatible chat completion.
//!
//! Renders system and user prompts as Jinja2 templates, calls a
//! `/v1/chat/completions` endpoint, and returns the assistant's reply along
//! with token-usage statistics.
//!
//! Works with any OpenAI-compatible API: OpenAI, Anthropic (via proxy),
//! Ollama, LM Studio, vLLM, Together AI, etc.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "model":         "gpt-4o-mini",
//!   "user_prompt":   "Answer concisely: {{ query }}",
//!   "system_prompt": "You are a helpful assistant.",
//!   "api_base":      "https://api.openai.com/v1",
//!   "api_key":       "sk-...",
//!   "temperature":   0.7,
//!   "max_tokens":    1024
//! }
//! ```
//!
//! | Field | Type | Required | Default | Description |
//! |-------|------|:--------:|---------|-------------|
//! | `model` | string | ✅ | — | Model identifier |
//! | `user_prompt` | string | ✅ | — | User turn — rendered as Jinja2 template |
//! | `system_prompt` | string | — | _(none)_ | System turn — rendered as Jinja2 template |
//! | `api_base` | string | — | `https://api.openai.com/v1` | Base URL (no trailing slash) |
//! | `api_key` | string | — | `""` | Bearer token; may be empty for local models |
//! | `temperature` | number | — | `0.7` | Sampling temperature `[0, 2]` |
//! | `max_tokens` | integer | — | _(none)_ | Max completion tokens |
//!
//! ## Template context
//!
//! Both prompts are Jinja2 templates. The rendering context contains:
//! - All global flow `variables` (by key)
//! - All upstream node outputs (by node ID)
//!
//! Upstream inputs shadow variables with the same key.
//!
//! # Output schema
//!
//! ```json
//! {
//!   "text":          "The answer is 42.",
//!   "model":         "gpt-4o-mini",
//!   "finish_reason": "stop",
//!   "usage": {
//!     "prompt_tokens":     15,
//!     "completion_tokens":  8,
//!     "total_tokens":      23
//!   }
//! }
//! ```

use std::collections::HashMap;

use async_trait::async_trait;
use serde::Serialize;
use serde_json::{json, Value};

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

const DEFAULT_API_BASE: &str = "https://api.openai.com/v1";
const DEFAULT_TEMPERATURE: f64 = 0.7;

// ── Public node ───────────────────────────────────────────────────────────────

/// LLM chat-completion node (OpenAI-compatible).
pub struct LlmNode;

#[async_trait]
impl Node for LlmNode {
    fn node_type(&self) -> &str {
        "llm"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let config = LlmConfig::from_data(&ctx.data)?;
        let jinja_ctx = build_jinja_context(&ctx);

        let user_prompt = render(&config.user_prompt, &jinja_ctx)?;
        let system_prompt = config
            .system_prompt
            .as_deref()
            .map(|t| render(t, &jinja_ctx))
            .transpose()?;

        let mut messages: Vec<ChatMessage> = Vec::new();
        if let Some(sys) = system_prompt {
            messages.push(ChatMessage { role: "system".into(), content: sys });
        }
        messages.push(ChatMessage { role: "user".into(), content: user_prompt });

        let result = do_chat_completion(
            &config.api_base,
            &config.api_key,
            &config.model,
            messages,
            Some(config.temperature),
            config.max_tokens,
        )
        .await?;

        Ok(json!({
            "text": result.text,
            "model": result.model,
            "finish_reason": result.finish_reason,
            "usage": {
                "prompt_tokens":     result.prompt_tokens,
                "completion_tokens": result.completion_tokens,
                "total_tokens":      result.total_tokens,
            }
        }))
    }
}

// ── Shared internals (used by question-classifier too) ────────────────────────

/// Parsed node configuration.
#[derive(Debug)]
pub(crate) struct LlmConfig {
    pub model: String,
    pub user_prompt: String,
    pub system_prompt: Option<String>,
    pub api_base: String,
    pub api_key: String,
    pub temperature: f64,
    pub max_tokens: Option<u64>,
}

impl LlmConfig {
    pub(crate) fn from_data(data: &Value) -> Result<Self> {
        let model = data["model"]
            .as_str()
            .ok_or_else(|| FlowError::InvalidDefinition("llm: missing data.model".into()))?
            .to_string();

        let user_prompt = data["user_prompt"]
            .as_str()
            .ok_or_else(|| {
                FlowError::InvalidDefinition("llm: missing data.user_prompt".into())
            })?
            .to_string();

        let system_prompt = data["system_prompt"].as_str().map(str::to_string);
        let api_base = data["api_base"]
            .as_str()
            .unwrap_or(DEFAULT_API_BASE)
            .trim_end_matches('/')
            .to_string();
        let api_key = data["api_key"].as_str().unwrap_or("").to_string();
        let temperature = data["temperature"].as_f64().unwrap_or(DEFAULT_TEMPERATURE);
        let max_tokens = data["max_tokens"].as_u64();

        Ok(Self {
            model,
            user_prompt,
            system_prompt,
            api_base,
            api_key,
            temperature,
            max_tokens,
        })
    }

    /// Parse only the connection-level fields (model, api_base, api_key, temperature,
    /// max_tokens). Does NOT require `user_prompt` — used by nodes that build their
    /// own prompts (e.g. `question-classifier`).
    pub(crate) fn from_connection_data(data: &Value) -> Result<Self> {
        let model = data["model"]
            .as_str()
            .ok_or_else(|| FlowError::InvalidDefinition("llm: missing data.model".into()))?
            .to_string();

        let api_base = data["api_base"]
            .as_str()
            .unwrap_or(DEFAULT_API_BASE)
            .trim_end_matches('/')
            .to_string();
        let api_key = data["api_key"].as_str().unwrap_or("").to_string();
        let temperature = data["temperature"].as_f64().unwrap_or(DEFAULT_TEMPERATURE);
        let max_tokens = data["max_tokens"].as_u64();

        Ok(Self {
            model,
            user_prompt: String::new(),
            system_prompt: None,
            api_base,
            api_key,
            temperature,
            max_tokens,
        })
    }
}

/// One message in a chat conversation.
#[derive(Debug, Serialize)]
pub(crate) struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Extracted fields from a successful chat-completion response.
#[derive(Debug)]
pub(crate) struct CompletionResult {
    pub text: String,
    pub model: String,
    pub finish_reason: String,
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

/// Build a Jinja2 rendering context from the execution context.
///
/// Variables have lower priority; upstream inputs shadow same-named variables.
pub(crate) fn build_jinja_context(ctx: &ExecContext) -> HashMap<String, Value> {
    let mut map: HashMap<String, Value> = ctx.variables.clone();
    for (k, v) in &ctx.inputs {
        map.insert(k.clone(), v.clone());
    }
    map
}

/// Render a Jinja2 template string against the given context map.
pub(crate) fn render(template: &str, context: &HashMap<String, Value>) -> Result<String> {
    let env = minijinja::Environment::new();
    env.render_str(template, context)
        .map_err(|e| FlowError::Internal(format!("llm: template render error: {e}")))
}

/// Call the `/v1/chat/completions` endpoint and return the parsed result.
pub(crate) async fn do_chat_completion(
    api_base: &str,
    api_key: &str,
    model: &str,
    messages: Vec<ChatMessage>,
    temperature: Option<f64>,
    max_tokens: Option<u64>,
) -> Result<CompletionResult> {
    let mut body = json!({
        "model": model,
        "messages": messages,
        "temperature": temperature.unwrap_or(DEFAULT_TEMPERATURE),
    });
    if let Some(max_tok) = max_tokens {
        body["max_tokens"] = json!(max_tok);
    }

    let url = format!("{api_base}/chat/completions");
    let client = reqwest::Client::new();
    let mut req = client.post(&url).json(&body);
    if !api_key.is_empty() {
        req = req.bearer_auth(api_key);
    }

    let response = req
        .send()
        .await
        .map_err(|e| FlowError::Internal(format!("llm: HTTP request failed: {e}")))?;

    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| FlowError::Internal(format!("llm: failed to read response body: {e}")))?;

    if !status.is_success() {
        return Err(FlowError::Internal(format!(
            "llm: API returned {status}: {text}"
        )));
    }

    let resp: Value = serde_json::from_str(&text)
        .map_err(|e| FlowError::Internal(format!("llm: failed to parse response JSON: {e}")))?;

    parse_completion_response(&resp)
}

/// Parse a raw `/v1/chat/completions` JSON response into [`CompletionResult`].
///
/// Extracted as a separate function so it can be unit-tested without network.
pub(crate) fn parse_completion_response(resp: &Value) -> Result<CompletionResult> {
    let text = resp
        .pointer("/choices/0/message/content")
        .and_then(|v| v.as_str())
        .ok_or_else(|| {
            FlowError::Internal("llm: unexpected response shape (missing choices[0].message.content)".into())
        })?
        .to_string();

    let finish_reason = resp
        .pointer("/choices/0/finish_reason")
        .and_then(|v| v.as_str())
        .unwrap_or("stop")
        .to_string();

    let model = resp["model"].as_str().unwrap_or("unknown").to_string();
    let prompt_tokens = resp.pointer("/usage/prompt_tokens").and_then(|v| v.as_u64()).unwrap_or(0);
    let completion_tokens = resp
        .pointer("/usage/completion_tokens")
        .and_then(|v| v.as_u64())
        .unwrap_or(0);
    let total_tokens = resp.pointer("/usage/total_tokens").and_then(|v| v.as_u64()).unwrap_or(0);

    Ok(CompletionResult {
        text,
        model,
        finish_reason,
        prompt_tokens,
        completion_tokens,
        total_tokens,
    })
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    // ── Config validation ──────────────────────────────────────────────────

    #[test]
    fn rejects_missing_model() {
        let err = LlmConfig::from_data(&json!({ "user_prompt": "hi" })).unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[test]
    fn rejects_missing_user_prompt() {
        let err = LlmConfig::from_data(&json!({ "model": "gpt-4o" })).unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[test]
    fn applies_defaults() {
        let cfg = LlmConfig::from_data(&json!({
            "model": "gpt-4o-mini",
            "user_prompt": "hello"
        }))
        .unwrap();
        assert_eq!(cfg.api_base, DEFAULT_API_BASE);
        assert_eq!(cfg.api_key, "");
        assert!((cfg.temperature - DEFAULT_TEMPERATURE).abs() < 1e-9);
        assert!(cfg.max_tokens.is_none());
        assert!(cfg.system_prompt.is_none());
    }

    #[test]
    fn trailing_slash_stripped_from_api_base() {
        let cfg = LlmConfig::from_data(&json!({
            "model": "x",
            "user_prompt": "y",
            "api_base": "http://localhost:11434/v1/"
        }))
        .unwrap();
        assert_eq!(cfg.api_base, "http://localhost:11434/v1");
    }

    // ── Template rendering ─────────────────────────────────────────────────

    #[test]
    fn renders_user_prompt_with_variables() {
        let ctx_map = HashMap::from([("query".to_string(), json!("What is 2+2?"))]);
        let rendered = render("Answer: {{ query }}", &ctx_map).unwrap();
        assert_eq!(rendered, "Answer: What is 2+2?");
    }

    #[test]
    fn renders_user_prompt_with_upstream_input() {
        let ctx_map = HashMap::from([("fetch".to_string(), json!({ "body": "data" }))]);
        let rendered = render("Got: {{ fetch.body }}", &ctx_map).unwrap();
        assert_eq!(rendered, "Got: data");
    }

    #[test]
    fn inputs_shadow_variables_in_context() {
        let mut ctx = ExecContext {
            variables: HashMap::from([("x".to_string(), json!("from_var"))]),
            inputs: HashMap::from([("x".to_string(), json!("from_input"))]),
            ..Default::default()
        };
        ctx.data = json!({});
        let map = build_jinja_context(&ctx);
        assert_eq!(map["x"], json!("from_input"));
    }

    // ── Response parsing ───────────────────────────────────────────────────

    #[test]
    fn parses_standard_completion_response() {
        let resp = json!({
            "model": "gpt-4o-mini",
            "choices": [{
                "message": { "role": "assistant", "content": "Hello!" },
                "finish_reason": "stop"
            }],
            "usage": {
                "prompt_tokens": 10,
                "completion_tokens": 5,
                "total_tokens": 15
            }
        });
        let result = parse_completion_response(&resp).unwrap();
        assert_eq!(result.text, "Hello!");
        assert_eq!(result.model, "gpt-4o-mini");
        assert_eq!(result.finish_reason, "stop");
        assert_eq!(result.prompt_tokens, 10);
        assert_eq!(result.completion_tokens, 5);
        assert_eq!(result.total_tokens, 15);
    }

    #[test]
    fn missing_choices_returns_error() {
        let err = parse_completion_response(&json!({ "model": "x", "choices": [] })).unwrap_err();
        assert!(matches!(err, FlowError::Internal(_)));
    }

    #[test]
    fn missing_content_returns_error() {
        let err = parse_completion_response(&json!({
            "choices": [{ "message": { "role": "assistant" } }]
        }))
        .unwrap_err();
        assert!(matches!(err, FlowError::Internal(_)));
    }

    #[test]
    fn partial_usage_fields_default_to_zero() {
        let resp = json!({
            "model": "x",
            "choices": [{ "message": { "content": "ok" }, "finish_reason": "stop" }]
        });
        let result = parse_completion_response(&resp).unwrap();
        assert_eq!(result.total_tokens, 0);
    }
}
