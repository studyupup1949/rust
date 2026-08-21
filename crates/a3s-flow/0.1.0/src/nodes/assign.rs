//! `"assign"` node — write key-value pairs into the flow's variable scope.
//!
//! Evaluates each entry in `data["assigns"]` and returns the result as a JSON
//! object. The runner automatically merges this output into the live variable
//! map so that all downstream nodes see the new values in `ctx.variables`.
//!
//! String values are rendered as Jinja2 templates (same context as the `"llm"`
//! node: variables + upstream inputs). Non-string JSON values (numbers,
//! booleans, objects, arrays) are used as-is.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "assigns": {
//!     "greeting": "Hello, {{ name }}!",
//!     "count":    0,
//!     "tags":     ["a", "b"]
//!   }
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `assigns` | object | ✅ | Map of variable names to Jinja2 templates (strings) or literal JSON values |
//!
//! ## Template context
//!
//! Same as the `"llm"` node: all global `variables` plus all upstream node
//! outputs keyed by node ID. Upstream inputs shadow variables with the same key.
//!
//! # Output schema
//!
//! The output is the resolved assignment map — identical to what is merged into
//! the flow's variable scope:
//!
//! ```json
//! { "greeting": "Hello, Alice!", "count": 0, "tags": ["a", "b"] }
//! ```
//!
//! # Example
//!
//! ```json
//! {
//!   "id": "init",
//!   "type": "assign",
//!   "data": {
//!     "assigns": {
//!       "user_name": "{{ start.name }}",
//!       "attempt":   1
//!     }
//!   }
//! }
//! ```

use async_trait::async_trait;
use serde_json::Value;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

use super::llm::{build_jinja_context, render};

/// Assign node — merges key-value pairs into the flow's variable scope.
pub struct AssignNode;

#[async_trait]
impl Node for AssignNode {
    fn node_type(&self) -> &str {
        "assign"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let assigns = ctx.data["assigns"].as_object().ok_or_else(|| {
            FlowError::InvalidDefinition(
                "assign: missing or non-object data.assigns".into(),
            )
        })?;

        let jinja_ctx = build_jinja_context(&ctx);
        let mut out = serde_json::Map::new();

        for (key, template_val) in assigns {
            let resolved = if let Some(s) = template_val.as_str() {
                // String values are Jinja2 templates.
                Value::String(render(s, &jinja_ctx)?)
            } else {
                // Non-string values are used as-is (numbers, bools, objects, arrays).
                template_val.clone()
            };
            out.insert(key.clone(), resolved);
        }

        Ok(Value::Object(out))
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn ctx(data: Value) -> ExecContext {
        ExecContext { data, ..Default::default() }
    }

    fn ctx_with_vars(data: Value, variables: HashMap<String, Value>) -> ExecContext {
        ExecContext { data, variables, ..Default::default() }
    }

    fn ctx_with_inputs(
        data: Value,
        inputs: HashMap<String, Value>,
        variables: HashMap<String, Value>,
    ) -> ExecContext {
        ExecContext { data, inputs, variables, ..Default::default() }
    }

    #[tokio::test]
    async fn assigns_string_template() {
        let node = AssignNode;
        let out = node
            .execute(ctx_with_vars(
                json!({ "assigns": { "greeting": "Hello, {{ name }}!" } }),
                HashMap::from([("name".into(), json!("Alice"))]),
            ))
            .await
            .unwrap();
        assert_eq!(out["greeting"], json!("Hello, Alice!"));
    }

    #[tokio::test]
    async fn assigns_literal_non_string_values() {
        let node = AssignNode;
        let out = node
            .execute(ctx(json!({
                "assigns": {
                    "count":  0,
                    "flag":   true,
                    "tags":   ["a", "b"],
                    "nested": { "x": 1 }
                }
            })))
            .await
            .unwrap();
        assert_eq!(out["count"], json!(0));
        assert_eq!(out["flag"], json!(true));
        assert_eq!(out["tags"], json!(["a", "b"]));
        assert_eq!(out["nested"], json!({ "x": 1 }));
    }

    #[tokio::test]
    async fn template_reads_upstream_input() {
        let node = AssignNode;
        let out = node
            .execute(ctx_with_inputs(
                json!({ "assigns": { "msg": "status={{ fetch.status }}" } }),
                HashMap::from([("fetch".into(), json!({ "status": 200 }))]),
                HashMap::new(),
            ))
            .await
            .unwrap();
        assert_eq!(out["msg"], json!("status=200"));
    }

    #[tokio::test]
    async fn inputs_shadow_variables_in_template() {
        let node = AssignNode;
        let out = node
            .execute(ctx_with_inputs(
                json!({ "assigns": { "val": "{{ x }}" } }),
                HashMap::from([("x".into(), json!("from_input"))]),
                HashMap::from([("x".into(), json!("from_var"))]),
            ))
            .await
            .unwrap();
        assert_eq!(out["val"], json!("from_input"));
    }

    #[tokio::test]
    async fn empty_assigns_returns_empty_object() {
        let node = AssignNode;
        let out = node
            .execute(ctx(json!({ "assigns": {} })))
            .await
            .unwrap();
        assert_eq!(out, json!({}));
    }

    #[tokio::test]
    async fn missing_assigns_returns_error() {
        let node = AssignNode;
        let err = node
            .execute(ctx(json!({})))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn non_object_assigns_returns_error() {
        let node = AssignNode;
        let err = node
            .execute(ctx(json!({ "assigns": ["not", "an", "object"] })))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn invalid_template_returns_error() {
        let node = AssignNode;
        let err = node
            .execute(ctx(json!({ "assigns": { "x": "{{ bad | unknown_filter }}" } })))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::Internal(_)));
    }
}
