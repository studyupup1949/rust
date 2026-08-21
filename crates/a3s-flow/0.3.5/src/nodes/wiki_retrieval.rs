//! Built-in `"wiki-retrieval"` node — semantic search over SafeClaw's WikiStore.
//!
//! Calls the local gateway's Search API to query the wiki knowledge base.
//! Uses minijinja templating for the query parameter, with variables and
//! upstream node outputs as the rendering context.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "query": "{{ start.query }}",
//!   "top_k": 5,
//!   "score_threshold": 0.0
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `query` | string | ✅ | Search query (Jinja2 template) |
//! | `top_k` | number | — | Max results to return (default 5) |
//! | `score_threshold` | number | — | Minimum similarity score (default 0.0) |
//!
//! # Output schema
//!
//! ```json
//! {
//!   "query": "original query text",
//!   "results": [
//!     { "id": "...", "title": "...", "content": "...", "score": 0.95, "categories": [] }
//!   ],
//!   "total": 3
//! }
//! ```

use async_trait::async_trait;
use minijinja::Environment;
use reqwest::Client;
use serde_json::{json, Value};
use std::collections::HashMap;
use urlencoding::encode;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

/// Wiki Retrieval node — queries SafeClaw's WikiStore via HTTP Search API.
pub struct WikiRetrievalNode;

const DEFAULT_GATEWAY_URL: &str = "http://127.0.0.1:29653";

#[async_trait]
impl Node for WikiRetrievalNode {
    fn node_type(&self) -> &str {
        "wiki-retrieval"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let data = &ctx.data;

        // ── Config extraction ─────────────────────────────────────────────────
        let query_template = data
            .get("query")
            .and_then(|v| v.as_str())
            .ok_or_else(|| FlowError::InvalidDefinition("wiki-retrieval: missing query".into()))?
            .trim();

        let top_k = data.get("top_k").and_then(|v| v.as_u64()).unwrap_or(5) as usize;

        let _score_threshold = data
            .get("score_threshold")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.0);

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

        // ── Render query ───────────────────────────────────────────────────────
        let rendered_query = env.render_str(query_template, &context).map_err(|e| {
            FlowError::Internal(format!("wiki-retrieval: query template error: {}", e))
        })?;

        if rendered_query.is_empty() {
            return Ok(json!({
                "query": "",
                "results": [],
                "total": 0,
            }));
        }

        // ── HTTP GET to Search API ──────────────────────────────────────────────
        let url = format!(
            "{}/api/v1/search/query?q={}&limit={}",
            gateway_url,
            encode(&rendered_query),
            top_k
        );

        let client = Client::new();
        let response = client
            .get(&url)
            .header("Accept", "application/json")
            .send()
            .await
            .map_err(|e| FlowError::Internal(format!("wiki-retrieval: request failed: {}", e)))?;

        let status = response.status().as_u16();
        let text = response.text().await.map_err(|e| {
            FlowError::Internal(format!("wiki-retrieval: failed to read response: {}", e))
        })?;

        let body: Value = serde_json::from_str(&text)
            .unwrap_or_else(|_| json!({ "error": format!("failed to parse response: {}", text) }));

        if body.get("error").is_some() {
            return Err(FlowError::Internal(format!(
                "wiki-retrieval: search API error: {}",
                body.get("error")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
            )));
        }

        // Parse search API response: { query, hits: [{ id, score, title, snippet?, categories }], count }
        let hits = body
            .get("hits")
            .and_then(|v| v.as_array())
            .cloned()
            .unwrap_or_default();
        let results: Vec<Value> = hits
            .iter()
            .map(|hit| {
                json!({
                    "id": hit.get("id").and_then(|v| v.as_str()).unwrap_or_default(),
                    "title": hit.get("title").and_then(|v| v.as_str()).unwrap_or_default(),
                    "content": hit.get("snippet").and_then(|v| v.as_str()).unwrap_or_default(),
                    "score": hit.get("score").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    "categories": hit.get("categories").and_then(|v| v.as_array()).map(|arr| {
                        arr.iter().filter_map(|v| v.as_str().map(String::from)).collect::<Vec<_>>()
                    }).unwrap_or_default(),
                })
            })
            .collect();

        let total = results.len();

        Ok(json!({
            "query": rendered_query,
            "results": results,
            "total": total,
            "status": status,
        }))
    }
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
    async fn rejects_empty_query() {
        let node = WikiRetrievalNode;
        let out = node
            .execute(make_ctx(
                HashMap::new(),
                HashMap::from([("start".into(), json!({ "query": "" }))]),
                json!({
                    "query": "{{ start.query }}",
                    "top_k": 5
                }),
            ))
            .await
            .unwrap();
        assert_eq!(out["query"], "");
        assert_eq!(out["total"], 0);
    }
}
