//! `"parameter-extractor"` node — LLM-powered structured parameter extraction.
//!
//! Uses an LLM to infer and extract typed parameters from natural language
//! text. The output is a JSON object keyed by the declared parameter names,
//! suitable for passing directly to downstream HTTP-request or code nodes.
//!
//! Works with any OpenAI-compatible API — same backend options as the `"llm"`
//! node (OpenAI, Ollama, vLLM, Together AI, etc.).
//!
//! # Config schema
//!
//! ```json
//! {
//!   "model":      "gpt-4o-mini",
//!   "query":      "Book a flight from {{ city }} to London on {{ date }}",
//!   "parameters": [
//!     { "name": "origin",      "type": "string",  "description": "Departure city", "required": true },
//!     { "name": "destination", "type": "string",  "description": "Arrival city",   "required": true },
//!     { "name": "date",        "type": "string",  "description": "ISO date",        "required": false }
//!   ],
//!   "api_base":    "https://api.openai.com/v1",
//!   "api_key":     "sk-...",
//!   "temperature": 0.0
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `model` | string | ✅ | Model identifier |
//! | `query` | string | ✅ | Text to extract from — rendered as Jinja2 template |
//! | `parameters` | array | ✅ | At least one parameter declaration |
//! | `parameters[].name` | string | ✅ | Parameter key in the output object |
//! | `parameters[].type` | string | — | Type hint for the LLM (`"string"`, `"number"`, `"boolean"`, `"object"`, `"array"`) — default `"string"` |
//! | `parameters[].description` | string | — | Extraction guidance for the LLM |
//! | `parameters[].required` | boolean | — | If true, extraction fails when the LLM cannot find the value (default `false`) |
//! | `api_base`, `api_key`, `temperature`, `max_tokens` | | — | Same as `"llm"` node |
//!
//! ## Template context
//!
//! `query` is a Jinja2 template rendered with the same context as the `"llm"` node:
//! global `variables` plus upstream node outputs keyed by node ID.
//!
//! # Output schema
//!
//! ```json
//! { "origin": "Paris", "destination": "London", "date": null }
//! ```
//!
//! Optional parameters that the LLM cannot find are set to `null`.
//! Required parameters that cannot be found cause an `Internal` error.

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

use super::llm::{build_jinja_context, do_chat_completion, render, ChatMessage, LlmConfig};

// ── Parameter declaration ──────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct ParamDecl {
    name: String,
    #[serde(rename = "type", default = "default_param_type")]
    param_type: String,
    #[serde(default)]
    description: String,
    #[serde(default)]
    required: bool,
}

fn default_param_type() -> String {
    "string".into()
}

// ── Node ──────────────────────────────────────────────────────────────────

/// Parameter extractor node — LLM-powered structured extraction.
pub struct ParameterExtractorNode;

#[async_trait]
impl Node for ParameterExtractorNode {
    fn node_type(&self) -> &str {
        "parameter-extractor"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let config = LlmConfig::from_connection_data(&ctx.data)?;

        let query_template = ctx.data["query"].as_str().ok_or_else(|| {
            FlowError::InvalidDefinition(
                "parameter-extractor: missing data.query".into(),
            )
        })?;

        let params: Vec<ParamDecl> =
            serde_json::from_value(ctx.data["parameters"].clone()).map_err(|e| {
                FlowError::InvalidDefinition(format!(
                    "parameter-extractor: invalid parameters declaration: {e}"
                ))
            })?;

        if params.is_empty() {
            return Err(FlowError::InvalidDefinition(
                "parameter-extractor: at least one parameter required".into(),
            ));
        }

        let jinja_ctx = build_jinja_context(&ctx);
        let query = render(query_template, &jinja_ctx)?;

        // ── Build extraction prompt ────────────────────────────────────────
        let param_list: String = params
            .iter()
            .map(|p| {
                let req = if p.required { " (required)" } else { " (optional)" };
                if p.description.is_empty() {
                    format!("- {}: {}{}", p.name, p.param_type, req)
                } else {
                    format!("- {}: {} — {}{}", p.name, p.param_type, p.description, req)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let param_names: String = params
            .iter()
            .map(|p| format!("\"{}\"", p.name))
            .collect::<Vec<_>>()
            .join(", ");

        let system_prompt = format!(
            "You are a parameter extraction assistant. Extract the following parameters \
             from the user's text and return them as a JSON object.\n\
             Parameters to extract:\n{param_list}\n\n\
             Return ONLY a valid JSON object with keys: {param_names}.\n\
             For optional parameters that cannot be found, use null.\n\
             Do not include any explanation or markdown code fences."
        );

        let messages = vec![
            ChatMessage { role: "system".into(), content: system_prompt },
            ChatMessage { role: "user".into(), content: query },
        ];

        let result = do_chat_completion(
            &config.api_base,
            &config.api_key,
            &config.model,
            messages,
            Some(config.temperature),
            config.max_tokens,
        )
        .await?;

        // ── Parse LLM response ─────────────────────────────────────────────
        let json_text = strip_markdown_fences(result.text.trim());
        let extracted: Value = serde_json::from_str(json_text).map_err(|e| {
            FlowError::Internal(format!(
                "parameter-extractor: LLM returned invalid JSON: {e}\nResponse: {}",
                result.text
            ))
        })?;

        let obj = extracted.as_object().ok_or_else(|| {
            FlowError::Internal(
                "parameter-extractor: LLM response is not a JSON object".into(),
            )
        })?;

        // Validate required parameters.
        for param in &params {
            if param.required && matches!(obj.get(&param.name), None | Some(Value::Null)) {
                return Err(FlowError::Internal(format!(
                    "parameter-extractor: required parameter '{}' not found in LLM response",
                    param.name
                )));
            }
        }

        Ok(extracted)
    }
}

// ── Helpers ───────────────────────────────────────────────────────────────

/// Strip markdown code fences from a string (e.g. ` ```json ... ``` `).
fn strip_markdown_fences(text: &str) -> &str {
    // Handle ```json ... ``` or ``` ... ```
    if let Some(start) = text.find("```") {
        let after_fence = &text[start + 3..];
        // Skip optional language tag on the same line.
        let content_start = after_fence.find('\n').map(|n| n + 1).unwrap_or(0);
        let content = &after_fence[content_start..];
        if let Some(end) = content.rfind("```") {
            return content[..end].trim();
        }
    }
    text
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    // ── strip_markdown_fences ──────────────────────────────────────────────

    #[test]
    fn strips_json_code_fence() {
        let input = "```json\n{\"a\": 1}\n```";
        assert_eq!(strip_markdown_fences(input), "{\"a\": 1}");
    }

    #[test]
    fn strips_plain_code_fence() {
        let input = "```\n{\"a\": 1}\n```";
        assert_eq!(strip_markdown_fences(input), "{\"a\": 1}");
    }

    #[test]
    fn passthrough_when_no_fence() {
        let input = "{\"a\": 1}";
        assert_eq!(strip_markdown_fences(input), input);
    }

    // ── Config validation ──────────────────────────────────────────────────

    #[tokio::test]
    async fn rejects_missing_model() {
        let node = ParameterExtractorNode;
        let err = node
            .execute(ExecContext {
                data: json!({
                    "query": "hello",
                    "parameters": [{ "name": "x", "type": "string" }]
                }),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn rejects_missing_query() {
        let node = ParameterExtractorNode;
        let err = node
            .execute(ExecContext {
                data: json!({
                    "model": "gpt-4o-mini",
                    "parameters": [{ "name": "x" }]
                }),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn rejects_empty_parameters() {
        let node = ParameterExtractorNode;
        let err = node
            .execute(ExecContext {
                data: json!({
                    "model": "gpt-4o-mini",
                    "query":  "hello",
                    "parameters": []
                }),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn rejects_invalid_template() {
        // Template render error fires before any network call.
        let node = ParameterExtractorNode;
        let err = node
            .execute(ExecContext {
                data: json!({
                    "model": "gpt-4o-mini",
                    "query":  "{{ x | bad_filter }}",
                    "parameters": [{ "name": "x" }]
                }),
                variables: HashMap::new(),
                inputs: HashMap::new(),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::Internal(_)));
    }
}
