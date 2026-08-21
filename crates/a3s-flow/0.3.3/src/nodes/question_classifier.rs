//! `"question-classifier"` node — LLM-powered routing.
//!
//! Classifies an input question into one of several user-defined classes using
//! an LLM, then outputs `{ "branch": "class_id" }` — the same shape as the
//! `"if-else"` node, so `run_if` conditions work identically.
//!
//! Uses the same OpenAI-compatible API as the [`LlmNode`](super::llm::LlmNode).
//!
//! # Config schema
//!
//! ```json
//! {
//!   "model":    "gpt-4o-mini",
//!   "question": "{{ user_input }}",
//!   "classes": [
//!     { "id": "technical", "name": "Technical question",
//!       "description": "Questions about code, APIs, or system behaviour" },
//!     { "id": "billing",   "name": "Billing question" },
//!     { "id": "general",   "name": "General question" }
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
//! | `question` | string | ✅ | Question to classify — rendered as Jinja2 template |
//! | `classes` | array | ✅ | At least 2 classes; each requires `id` and `name` |
//! | `classes[].id` | string | ✅ | Unique identifier returned as `branch` |
//! | `classes[].name` | string | ✅ | Human-readable class name |
//! | `classes[].description` | string | — | Optional extra guidance for the LLM |
//! | `api_base`, `api_key`, `temperature`, `max_tokens` | | — | Same as `"llm"` node |
//!
//! # Output schema
//!
//! ```json
//! { "branch": "technical" }
//! ```
//!
//! If the LLM response does not match any declared class ID (case-insensitive),
//! the node falls back to the first class. Use `run_if` on downstream nodes to
//! route by branch:
//!
//! ```json
//! {
//!   "run_if": { "from": "classifier", "path": "branch",
//!               "op": "eq", "value": "technical" }
//! }
//! ```

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::{json, Value};

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

use super::llm::{build_jinja_context, do_chat_completion, render, ChatMessage, LlmConfig};

#[derive(Debug, Deserialize)]
struct ClassDecl {
    id: String,
    name: String,
    #[serde(default)]
    description: String,
}

pub struct QuestionClassifierNode;

#[async_trait]
impl Node for QuestionClassifierNode {
    fn node_type(&self) -> &str {
        "question-classifier"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let config = LlmConfig::from_connection_data(&ctx.data)?;

        let question_template = ctx.data["question"].as_str().ok_or_else(|| {
            FlowError::InvalidDefinition("question-classifier: missing data.question".into())
        })?;

        let classes: Vec<ClassDecl> =
            serde_json::from_value(ctx.data["classes"].clone()).map_err(|e| {
                FlowError::InvalidDefinition(format!(
                    "question-classifier: invalid classes declaration: {e}"
                ))
            })?;

        if classes.len() < 2 {
            return Err(FlowError::InvalidDefinition(
                "question-classifier: at least 2 classes required".into(),
            ));
        }

        let jinja_ctx = build_jinja_context(&ctx);
        let question = render(question_template, &jinja_ctx)?;

        // Build a deterministic classification prompt.
        let class_list: String = classes
            .iter()
            .map(|c| {
                if c.description.is_empty() {
                    format!("- {}: {}", c.id, c.name)
                } else {
                    format!("- {}: {} — {}", c.id, c.name, c.description)
                }
            })
            .collect::<Vec<_>>()
            .join("\n");

        let system_prompt = format!(
            "You are a question classifier. Classify the user's question into \
             exactly one of the following categories:\n{class_list}\n\n\
             Respond with ONLY the category ID (e.g., \"{}\"). \
             Do not include any explanation or punctuation.",
            classes[0].id
        );

        let messages = vec![
            ChatMessage {
                role: "system".into(),
                content: system_prompt,
            },
            ChatMessage {
                role: "user".into(),
                content: question,
            },
        ];

        let result = do_chat_completion(
            &config.api_base,
            &config.api_key,
            &config.model,
            messages,
            Some(config.temperature),
            Some(config.max_tokens.unwrap_or(16)), // short response — just the ID
        )
        .await?;

        let branch = pick_class(&result.text.trim().to_lowercase(), &classes);
        Ok(json!({ "branch": branch }))
    }
}

/// Find the matching class ID from the LLM response text.
///
/// Strategy (in order):
/// 1. Exact match against a class ID (case-insensitive)
/// 2. A class ID appears anywhere in the response
/// 3. Fall back to the first class
fn pick_class(response: &str, classes: &[ClassDecl]) -> String {
    // 1. Exact match.
    for c in classes {
        if response == c.id.to_lowercase() {
            return c.id.clone();
        }
    }
    // 2. Substring match.
    for c in classes {
        if response.contains(c.id.to_lowercase().as_str()) {
            return c.id.clone();
        }
    }
    // 3. Fallback.
    classes[0].id.clone()
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn make_classes() -> Vec<ClassDecl> {
        vec![
            ClassDecl {
                id: "tech".into(),
                name: "Technical".into(),
                description: String::new(),
            },
            ClassDecl {
                id: "billing".into(),
                name: "Billing".into(),
                description: String::new(),
            },
            ClassDecl {
                id: "general".into(),
                name: "General".into(),
                description: String::new(),
            },
        ]
    }

    // ── pick_class ─────────────────────────────────────────────────────────

    #[test]
    fn exact_match_returns_class_id() {
        let classes = make_classes();
        assert_eq!(pick_class("billing", &classes), "billing");
    }

    #[test]
    fn case_insensitive_exact_match() {
        let classes = make_classes();
        assert_eq!(pick_class("TECH", &classes), "tech");
    }

    #[test]
    fn substring_match_when_llm_adds_extra_words() {
        let classes = make_classes();
        // LLM said "technical question" but the ID is "tech"... wait, ID is "tech"
        // so "technical" contains "tech" — first-prefix match
        assert_eq!(pick_class("general inquiry", &classes), "general");
    }

    #[test]
    fn fallback_to_first_class_on_no_match() {
        let classes = make_classes();
        assert_eq!(pick_class("something completely unknown", &classes), "tech");
    }

    // ── Config validation ──────────────────────────────────────────────────

    #[tokio::test]
    async fn rejects_missing_question() {
        let node = QuestionClassifierNode;
        let err = node
            .execute(ExecContext {
                data: json!({
                    "model": "gpt-4o-mini",
                    "classes": [
                        { "id": "a", "name": "A" },
                        { "id": "b", "name": "B" }
                    ]
                }),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn rejects_fewer_than_two_classes() {
        let node = QuestionClassifierNode;
        let err = node
            .execute(ExecContext {
                data: json!({
                    "model": "gpt-4o-mini",
                    "question": "hi",
                    "classes": [{ "id": "only", "name": "Only one" }]
                }),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn rejects_missing_model() {
        let node = QuestionClassifierNode;
        let err = node
            .execute(ExecContext {
                data: json!({
                    "question": "hi",
                    "classes": [
                        { "id": "a", "name": "A" },
                        { "id": "b", "name": "B" }
                    ]
                }),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn question_template_renders_with_variables() {
        // Verify rendering happens: if the template is invalid the error fires
        // before any network call, so we get Internal (render) not network error.
        let node = QuestionClassifierNode;
        let err = node
            .execute(ExecContext {
                data: json!({
                    "model": "gpt-4o-mini",
                    "question": "{{ undefined_var | some_unknown_filter }}",
                    "classes": [
                        { "id": "a", "name": "A" },
                        { "id": "b", "name": "B" }
                    ]
                }),
                variables: HashMap::new(),
                inputs: HashMap::new(),
                ..Default::default()
            })
            .await
            .unwrap_err();
        // Render errors map to FlowError::Internal.
        assert!(matches!(err, FlowError::Internal(_)));
    }
}
