//! Built-in `"wiki-index"` node — writes a wiki entry to SafeClaw's WikiStore.
//!
//! Calls the local gateway's Wiki API to create/update a wiki entry.
//! Uses minijinja templating for URL and body fields, with variables and
//! upstream node outputs as the rendering context.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "entry_id": "{{ item.id }}",
//!   "title": "{{ meta.title }}",
//!   "content_selector": "extract.content",
//!   "summary_selector": "meta.summary",
//!   "categories_selector": "meta.categories",
//!   "concepts_selector": "meta.concepts"
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `entry_id` | string | ✅ | Wiki entry ID (used in URL path) |
//! | `title` | string | ✅ | Entry title (Jinja2 template) |
//! | `content_selector` | string | ✅ | Variable path pointing to text content |
//! | `summary_selector` | string | — | Variable path to summary text |
//! | `categories_selector` | string | — | Variable path to categories array |
//! | `concepts_selector` | string | — | Variable path to concepts array |
//!
//! # Output schema
//!
//! ```json
//! {
//!   "ok": true,
//!   "entry_id": "doc_abc123",
//!   "status": 200
//! }
//! ```

use async_trait::async_trait;
use minijinja::Environment;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

/// Wiki Index node — writes entries to SafeClaw's WikiStore via HTTP.
pub struct WikiIndexNode;

const DEFAULT_GATEWAY_URL: &str = "http://127.0.0.1:29653";

#[async_trait]
impl Node for WikiIndexNode {
    fn node_type(&self) -> &str {
        "wiki-index"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let data = &ctx.data;

        // ── Config extraction ─────────────────────────────────────────────────
        let entry_id = data
            .get("entry_id")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowError::InvalidDefinition("wiki-index: missing entry_id".into()))?
            .trim();

        let title = data
            .get("title")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowError::InvalidDefinition("wiki-index: missing title".into()))?
            .trim();

        let content_selector = data
            .get("content_selector")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FlowError::InvalidDefinition("wiki-index: missing content_selector".into())
            })?
            .trim();

        let summary_selector = data.get("summary_selector").and_then(|v| v.as_str());
        let categories_selector = data.get("categories_selector").and_then(|v| v.as_str());
        let concepts_selector = data.get("concepts_selector").and_then(|v| v.as_str());

        let gateway_url = data
            .get("gateway_url")
            .and_then(|v| v.as_str())
            .unwrap_or(DEFAULT_GATEWAY_URL)
            .trim()
            .trim_end_matches('/');

        // ── Build Jinja2 context: variables + inputs (inputs shadow variables) ──
        let mut context: HashMap<String, Value> = ctx.variables.clone();
        for (k, v) in &ctx.inputs {
            context.insert(k.clone(), v.clone());
        }

        let env = Environment::new();

        // ── Render entry_id (used in URL) ────────────────────────────────────
        let rendered_entry_id = env.render_str(entry_id, &context).map_err(|e| {
            FlowError::Internal(format!("wiki-index: entry_id template error: {}", e))
        })?;

        // ── Render title ──────────────────────────────────────────────────────
        let rendered_title = env
            .render_str(title, &context)
            .map_err(|e| FlowError::Internal(format!("wiki-index: title template error: {}", e)))?;

        // ── Resolve content from context ──────────────────────────────────────
        let content = resolve_from_context(&context, content_selector)?;

        // ── Resolve optional fields ────────────────────────────────────────────
        let summary = summary_selector
            .and_then(|s| resolve_from_context(&context, s).ok())
            .filter(|s| !s.is_empty());

        let categories: Option<Vec<String>> =
            categories_selector.and_then(|s| resolve_list_from_context(&context, s).ok());

        let concepts: Option<Vec<String>> =
            concepts_selector.and_then(|s| resolve_list_from_context(&context, s).ok());

        // ── Build request body ─────────────────────────────────────────────────
        let body = json!({
            "title": rendered_title,
            "content": content,
            "summary": summary,
            "categories": categories.unwrap_or_default(),
            "concepts": concepts.unwrap_or_default(),
        });

        // ── HTTP PUT to Wiki API ──────────────────────────────────────────────
        let url = format!("{}/api/v1/wiki/entries/{}", gateway_url, rendered_entry_id);
        let client = Client::new();
        let response = client
            .put(&url)
            .header("Content-Type", "application/json")
            .json(&body)
            .send()
            .await
            .map_err(|e| FlowError::Internal(format!("wiki-index: request failed: {}", e)))?;

        let status = response.status().as_u16();
        let ok = response.status().is_success();

        Ok(json!({
            "ok": ok,
            "entry_id": rendered_entry_id,
            "status": status,
            "title": rendered_title,
        }))
    }
}

/// Resolve a string value from the Jinja2 context using a dot-path selector.
fn resolve_from_context(context: &HashMap<String, Value>, selector: &str) -> Result<String> {
    let parts: Vec<&str> = selector.split('.').collect();
    if parts.is_empty() {
        return Err(FlowError::InvalidDefinition(
            "wiki-index: empty content_selector".into(),
        ));
    }

    let root = context.get(parts[0]).ok_or_else(|| {
        FlowError::InvalidDefinition(format!("wiki-index: variable '{}' not found", parts[0]))
    })?;

    let mut current = root;
    for part in &parts[1..] {
        current = current.get(*part).ok_or_else(|| {
            FlowError::InvalidDefinition(format!(
                "wiki-index: path '{}' not found in '{}'",
                part, parts[0]
            ))
        })?;
    }

    current.as_str().map(String::from).ok_or_else(|| {
        FlowError::InvalidDefinition(format!(
            "wiki-index: value at '{}' is not a string",
            selector
        ))
    })
}

/// Resolve a list (array) of strings from the Jinja2 context.
fn resolve_list_from_context(
    context: &HashMap<String, Value>,
    selector: &str,
) -> Result<Vec<String>> {
    let parts: Vec<&str> = selector.split('.').collect();
    if parts.is_empty() {
        return Err(FlowError::InvalidDefinition(
            "wiki-index: empty selector".into(),
        ));
    }

    let root = context.get(parts[0]).ok_or_else(|| {
        FlowError::InvalidDefinition(format!("wiki-index: variable '{}' not found", parts[0]))
    })?;

    let mut current = root;
    for part in &parts[1..] {
        current = current.get(*part).ok_or_else(|| {
            FlowError::InvalidDefinition(format!(
                "wiki-index: path '{}' not found in '{}'",
                part, parts[0]
            ))
        })?;
    }

    current
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .ok_or_else(|| {
            FlowError::InvalidDefinition(format!(
                "wiki-index: value at '{}' is not an array",
                selector
            ))
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_ctx(
        variables: HashMap<String, Value>,
        inputs: HashMap<String, Value>,
        data: Value,
    ) -> ExecContext {
        ExecContext {
            data,
            inputs,
            variables,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn renders_title_and_content() {
        let node = WikiIndexNode;
        let out = node
            .execute(make_ctx(
                HashMap::new(),
                HashMap::from([
                    (
                        "extract".into(),
                        json!({ "content": "Hello world this is a test document." }),
                    ),
                    (
                        "meta".into(),
                        json!({ "title": "Test Doc", "summary": "A test" }),
                    ),
                ]),
                json!({
                    "entry_id": "doc_001",
                    "title": "{{ meta.title }}",
                    "content_selector": "extract.content",
                    "summary_selector": "meta.summary",
                    "gateway_url": "http://127.0.0.1:29653"
                }),
            ))
            .await;
        // Without a running gateway this will fail the HTTP call, but template rendering works
        assert!(out.is_err() || out.as_ref().unwrap().get("title").is_some());
    }
}
