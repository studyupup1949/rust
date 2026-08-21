//! Built-in `"http-request"` node — executes an HTTP request and returns the
//! response status, body, and headers.
//!
//! Mirrors Dify's HTTP Request node.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "url":     "https://api.example.com/items",
//!   "method":  "POST",
//!   "headers": { "Authorization": "Bearer {{token}}" },
//!   "body":    { "key": "value" }
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `url` | string | ✓ | Request URL |
//! | `method` | string | | HTTP method — `GET` (default), `POST`, `PUT`, `DELETE`, `PATCH` |
//! | `headers` | object | | String-valued request headers |
//! | `body` | any JSON | | Request body (sent as `application/json`) |
//!
//! # Output schema
//!
//! ```json
//! { "status": 200, "ok": true, "body": { ... } }
//! ```
//!
//! `body` is parsed as JSON when the Content-Type is JSON; otherwise stored
//! as a plain string.

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

/// HTTP request node (Dify-compatible).
///
/// Supports `GET`, `POST`, `PUT`, `DELETE`, and `PATCH`.
pub struct HttpRequestNode;

#[async_trait]
impl Node for HttpRequestNode {
    fn node_type(&self) -> &str {
        "http-request"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let data = &ctx.data;

        let url = data["url"]
            .as_str()
            .ok_or_else(|| FlowError::InvalidDefinition("http-request: missing data.url".into()))?;

        let method = data["method"].as_str().unwrap_or("GET");

        let client = reqwest::Client::new();
        let mut req = match method.to_ascii_uppercase().as_str() {
            "GET" => client.get(url),
            "POST" => client.post(url),
            "PUT" => client.put(url),
            "DELETE" => client.delete(url),
            "PATCH" => client.patch(url),
            other => {
                return Err(FlowError::InvalidDefinition(format!(
                    "http-request: unsupported method '{other}'"
                )))
            }
        };

        if let Some(headers) = data["headers"].as_object() {
            for (name, val) in headers {
                if let Some(v) = val.as_str() {
                    req = req.header(name.as_str(), v);
                }
            }
        }

        if let Some(body) = data.get("body") {
            if !body.is_null() {
                req = req.json(body);
            }
        }

        let response = req
            .send()
            .await
            .map_err(|e| FlowError::Internal(format!("http-request: request failed: {e}")))?;

        let status = response.status().as_u16();
        let ok = response.status().is_success();

        let text = response.text().await.map_err(|e| {
            FlowError::Internal(format!("http-request: failed to read response: {e}"))
        })?;

        let body: Value = serde_json::from_str(&text).unwrap_or(Value::String(text));

        Ok(json!({ "status": status, "ok": ok, "body": body }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn rejects_missing_url() {
        let err = HttpRequestNode
            .execute(ExecContext {
                data: json!({}),
                inputs: HashMap::new(),
                variables: HashMap::new(),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn rejects_unsupported_method() {
        let err = HttpRequestNode
            .execute(ExecContext {
                data: json!({ "url": "http://example.com", "method": "HEAD" }),
                inputs: HashMap::new(),
                variables: HashMap::new(),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }
}
