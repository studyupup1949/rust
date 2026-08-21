//! `"context-set"` node — write key-value pairs into the shared execution context.
//!
//! Works like the `"assign"` node but writes to [`ExecContext::context`] (the
//! shared `Arc<RwLock<HashMap>>`) instead of the flow-level variable scope.
//! Any downstream node — or an external caller via [`FlowEngine::get_context`] —
//! can read the values immediately after this node completes.
//!
//! String values are rendered as Jinja2 templates using the same context as the
//! `"llm"` node (variables + upstream inputs). Non-string JSON values are stored
//! as-is.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "assigns": {
//!     "user_id":  "{{ start.user_id }}",
//!     "attempt":  1,
//!     "metadata": { "source": "webhook" }
//!   }
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `assigns` | object | ✅ | Map of context keys to Jinja2 templates (strings) or literal JSON values |
//!
//! # Output schema
//!
//! Returns the resolved key-value map that was written to the context:
//!
//! ```json
//! { "user_id": "u_123", "attempt": 1, "metadata": { "source": "webhook" } }
//! ```

use async_trait::async_trait;
use serde_json::Value;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

use super::llm::{build_jinja_context, render};

/// Context-set node — writes key-value pairs into the shared execution context.
pub struct ContextSetNode;

#[async_trait]
impl Node for ContextSetNode {
    fn node_type(&self) -> &str {
        "context-set"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let assigns = ctx.data["assigns"].as_object().ok_or_else(|| {
            FlowError::InvalidDefinition("context-set: missing or non-object data.assigns".into())
        })?;

        let jinja_ctx = build_jinja_context(&ctx);
        let mut out = serde_json::Map::new();

        for (key, template_val) in assigns {
            let resolved = if let Some(s) = template_val.as_str() {
                Value::String(render(s, &jinja_ctx)?)
            } else {
                template_val.clone()
            };
            out.insert(key.clone(), resolved.clone());

            // Write directly into the shared context.
            ctx.context.write().unwrap().insert(key.clone(), resolved);
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
    async fn writes_literal_values_to_context() {
        let shared = Arc::new(RwLock::new(HashMap::new()));
        let node = ContextSetNode;
        let out = node
            .execute(ctx_with_context(
                json!({ "assigns": { "count": 42, "flag": true } }),
                Arc::clone(&shared),
            ))
            .await
            .unwrap();

        assert_eq!(out["count"], json!(42));
        assert_eq!(out["flag"], json!(true));

        let ctx = shared.read().unwrap();
        assert_eq!(ctx["count"], json!(42));
        assert_eq!(ctx["flag"], json!(true));
    }

    #[tokio::test]
    async fn renders_string_template_into_context() {
        let shared = Arc::new(RwLock::new(HashMap::new()));
        let node = ContextSetNode;
        node.execute(ExecContext {
            data: json!({ "assigns": { "greeting": "Hello, {{ name }}!" } }),
            variables: HashMap::from([("name".into(), json!("Alice"))]),
            context: Arc::clone(&shared),
            ..Default::default()
        })
        .await
        .unwrap();

        assert_eq!(shared.read().unwrap()["greeting"], json!("Hello, Alice!"));
    }

    #[tokio::test]
    async fn missing_assigns_returns_error() {
        let node = ContextSetNode;
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
    async fn overwrites_existing_context_entry() {
        let shared = Arc::new(RwLock::new(HashMap::from([("key".into(), json!("old"))])));
        let node = ContextSetNode;
        node.execute(ctx_with_context(
            json!({ "assigns": { "key": "new" } }),
            Arc::clone(&shared),
        ))
        .await
        .unwrap();

        assert_eq!(shared.read().unwrap()["key"], json!("new"));
    }
}
