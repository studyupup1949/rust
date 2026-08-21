//! `"context-get"` node — read values from the shared execution context.
//!
//! Reads one or more keys from [`ExecContext::context`] and returns them as a
//! JSON object. Missing keys produce a `null` value. The returned object can be
//! referenced by downstream nodes in the usual way (`{{ context_get_1.key }}`).
//!
//! # Config schema
//!
//! ```json
//! { "keys": ["user_id", "attempt", "metadata"] }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `keys` | string[] | ✅ | List of context keys to read; missing keys resolve to `null` |
//!
//! # Output schema
//!
//! ```json
//! { "user_id": "u_123", "attempt": 1, "metadata": null }
//! ```

use async_trait::async_trait;
use serde_json::Value;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

/// Context-get node — reads key-value pairs from the shared execution context.
pub struct ContextGetNode;

#[async_trait]
impl Node for ContextGetNode {
    fn node_type(&self) -> &str {
        "context-get"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let keys = ctx.data["keys"].as_array().ok_or_else(|| {
            FlowError::InvalidDefinition("context-get: missing or non-array data.keys".into())
        })?;

        let context = ctx.context.read().unwrap();
        let mut out = serde_json::Map::new();

        for key_val in keys {
            let key = key_val.as_str().ok_or_else(|| {
                FlowError::InvalidDefinition(
                    "context-get: each entry in data.keys must be a string".into(),
                )
            })?;
            let value = context.get(key).cloned().unwrap_or(Value::Null);
            out.insert(key.to_string(), value);
        }

        Ok(Value::Object(out))
    }
}

// ── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock};

    fn ctx_with_context(data: Value, shared: Arc<RwLock<HashMap<String, Value>>>) -> ExecContext {
        ExecContext {
            data,
            context: shared,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn reads_existing_keys() {
        let shared = Arc::new(RwLock::new(HashMap::from([
            ("user_id".into(), json!("u_42")),
            ("count".into(), json!(7)),
        ])));
        let node = ContextGetNode;
        let out = node
            .execute(ctx_with_context(
                json!({ "keys": ["user_id", "count"] }),
                Arc::clone(&shared),
            ))
            .await
            .unwrap();

        assert_eq!(out["user_id"], json!("u_42"));
        assert_eq!(out["count"], json!(7));
    }

    #[tokio::test]
    async fn missing_key_returns_null() {
        let shared = Arc::new(RwLock::new(HashMap::new()));
        let node = ContextGetNode;
        let out = node
            .execute(ctx_with_context(
                json!({ "keys": ["nonexistent"] }),
                Arc::clone(&shared),
            ))
            .await
            .unwrap();

        assert_eq!(out["nonexistent"], json!(null));
    }

    #[tokio::test]
    async fn empty_keys_returns_empty_object() {
        let shared = Arc::new(RwLock::new(HashMap::new()));
        let node = ContextGetNode;
        let out = node
            .execute(ctx_with_context(json!({ "keys": [] }), Arc::clone(&shared)))
            .await
            .unwrap();

        assert_eq!(out, json!({}));
    }

    #[tokio::test]
    async fn missing_keys_field_returns_error() {
        let node = ContextGetNode;
        let err = node
            .execute(ExecContext {
                data: json!({}),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn non_string_key_entry_returns_error() {
        let shared = Arc::new(RwLock::new(HashMap::new()));
        let node = ContextGetNode;
        let err = node
            .execute(ctx_with_context(
                json!({ "keys": [42] }),
                Arc::clone(&shared),
            ))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }
}
