//! `"trigger-webhook"` node — fires the workflow when an HTTP request is received.
//!
//! This node acts as the entry point of a DAG. When a workflow containing this
//! node receives an HTTP request that matches its configured path and method,
//! this node emits the request details (method, headers, body) as output.
//!
//! The webhook receiver and routing logic lives in the SafeClaw app layer, not
//! in a3s-flow. This node only validates the webhook configuration.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "method": "POST",
//!   "path": "/webhook/myflow",
//!   "content_type": "application/json",
//!   "headers": {},
//!   "params": {},
//!   "body": [],
//!   "async_mode": true,
//!   "timeout": 30,
//!   "status_code": 200,
//!   "response_body": ""
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `method` | string | — | HTTP method: GET, POST, PUT, DELETE, PATCH (default: "POST") |
//! | `path` | string | ✅ | URL path for this webhook (must start with /) |
//! | `content_type` | string | — | Expected Content-Type header (default: "application/json") |
//! | `headers` | object | — | Required header key-value pairs to validate |
//! | `params` | object | — | Expected query parameter keys |
//! | `body` | array | — | Expected body field keys to extract |
//! | `async_mode` | bool | — | If true, respond immediately before workflow completes (default: true) |
//! | `timeout` | number | — | Seconds to wait for workflow completion in sync mode (default: 30) |
//! | `status_code` | number | — | HTTP status code to return (default: 200) |
//! | `response_body` | string | — | Response body template (Jinja2, sent with sync response) |
//!
//! # Output schema
//!
//! ```json
//! {
//!   "method": "POST",
//!   "path": "/webhook/myflow",
//!   "headers": { "Content-Type": "application/json" },
//!   "body": { "key": "value" },
//!   "raw_body": "{\"key\": \"value\"}"
//! }
//! ```

use async_trait::async_trait;
use serde_json::Value;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

use super::protocol::WebhookTriggerPayload;

pub struct TriggerWebhookNode;

#[async_trait]
impl Node for TriggerWebhookNode {
    fn node_type(&self) -> &str {
        "trigger-webhook"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let method = ctx
            .data
            .get("method")
            .and_then(|v| v.as_str())
            .unwrap_or("POST")
            .to_uppercase();

        let valid_methods = ["GET", "POST", "PUT", "DELETE", "PATCH", "HEAD", "OPTIONS"];
        if !valid_methods.contains(&method.as_str()) {
            return Err(FlowError::InvalidDefinition(format!(
                "trigger-webhook: invalid method '{}' (allowed: {:?})",
                method, valid_methods
            )));
        }

        let path = ctx
            .data
            .get("path")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FlowError::InvalidDefinition(
                    "trigger-webhook: missing required field 'path'".to_string(),
                )
            })?;

        if !path.starts_with('/') {
            return Err(FlowError::InvalidDefinition(format!(
                "trigger-webhook: path must start with '/' (got '{}')",
                path
            )));
        }

        // Extract headers and body from variables injected by the app layer.
        // The app layer populates these when triggering the workflow.
        let headers = ctx
            .variables
            .get("_webhook_headers")
            .and_then(|v| v.as_object())
            .map(|obj| {
                obj.iter()
                    .filter_map(|(k, v)| v.as_str().map(|s| (k.clone(), s.to_string())))
                    .collect()
            })
            .unwrap_or_default();

        let body = ctx
            .variables
            .get("_webhook_body")
            .cloned()
            .unwrap_or(Value::Null);

        let raw_body = if body.is_string() {
            body.as_str().map(String::from)
        } else {
            serde_json::to_string(&body).ok()
        };

        let payload = WebhookTriggerPayload {
            method,
            path: path.to_string(),
            headers,
            body,
            raw_body,
        };

        Ok(serde_json::to_value(payload).expect("payload is serializable"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn ctx(data: Value, variables: HashMap<String, Value>) -> ExecContext {
        ExecContext {
            data,
            variables,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn valid_webhook_emits_payload() {
        let node = TriggerWebhookNode;
        let mut vars = HashMap::new();
        vars.insert(
            "_webhook_headers".into(),
            json!({ "Content-Type": "application/json" }),
        );
        vars.insert("_webhook_body".into(), json!({ "key": "value" }));

        let out = node
            .execute(ctx(
                json!({ "method": "POST", "path": "/webhook/test" }),
                vars,
            ))
            .await
            .unwrap();
        assert_eq!(out["method"], "POST");
        assert_eq!(out["path"], "/webhook/test");
        assert_eq!(out["headers"]["Content-Type"], "application/json");
        assert_eq!(out["body"]["key"], "value");
    }

    #[tokio::test]
    async fn default_method_is_post() {
        let node = TriggerWebhookNode;
        let out = node
            .execute(ctx(json!({ "path": "/webhook/test" }), HashMap::new()))
            .await
            .unwrap();
        assert_eq!(out["method"], "POST");
    }

    #[tokio::test]
    async fn missing_path_returns_error() {
        let node = TriggerWebhookNode;
        let result = node
            .execute(ctx(json!({ "method": "POST" }), HashMap::new()))
            .await;
        assert!(matches!(result, Err(FlowError::InvalidDefinition(_))));
    }

    #[tokio::test]
    async fn path_without_leading_slash_returns_error() {
        let node = TriggerWebhookNode;
        let result = node
            .execute(ctx(json!({ "path": "webhook/test" }), HashMap::new()))
            .await;
        assert!(matches!(result, Err(FlowError::InvalidDefinition(_))));
    }

    #[tokio::test]
    async fn invalid_method_returns_error() {
        let node = TriggerWebhookNode;
        let result = node
            .execute(ctx(
                json!({ "method": "INVALID", "path": "/webhook/test" }),
                HashMap::new(),
            ))
            .await;
        assert!(matches!(result, Err(FlowError::InvalidDefinition(_))));
    }

    #[tokio::test]
    async fn raw_body_serialization() {
        let node = TriggerWebhookNode;
        let mut vars = HashMap::new();
        vars.insert("_webhook_body".into(), json!({ "nested": { "key": 42 } }));
        let out = node
            .execute(ctx(json!({ "path": "/test" }), vars))
            .await
            .unwrap();
        assert!(out["raw_body"].is_string());
        let raw: String = serde_json::from_value(out["raw_body"].clone()).unwrap();
        assert!(raw.contains("nested"));
    }
}
