//! Built-in `"code"` node — executes an inline script in a sandboxed Rhai
//! engine and returns the result as JSON.
//!
//! Mirrors Dify's Code node. Rhai is a safe, embedded scripting language with
//! Rust-like syntax. It has no file system, network, or OS access by default.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "language": "rhai",
//!   "code": "let total = inputs.items.len(); #{ total: total }"
//! }
//! ```
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | `language` | string | Must be `"rhai"` (the only supported language) |
//! | `code` | string | Rhai script body |
//!
//! # Script context
//!
//! Two variables are injected into the script scope:
//! - `inputs` — map keyed by upstream node ID (equivalent to `ctx.inputs`)
//! - `variables` — global flow variables (equivalent to `ctx.variables`)
//!
//! # Output schema
//!
//! If the script returns a Rhai object map, it becomes the node output directly:
//!
//! ```rhai
//! #{ status: inputs.fetch.status, ok: inputs.fetch.ok }
//! // → { "status": 200, "ok": true }
//! ```
//!
//! Any other return type is wrapped under `"output"`:
//!
//! ```rhai
//! inputs.fetch.status == 200
//! // → { "output": true }
//! ```
//!
//! # Safety limits
//!
//! The engine enforces:
//! - Max 100,000 operations (prevents infinite loops)
//! - Max string size: 1 MB
//! - Max array size: 10,000 elements
//!
//! # Rhai syntax reference
//!
//! - Variables: `let x = 42;`
//! - Maps: `#{ key: value }`
//! - Arrays: `[1, 2, 3]`
//! - String ops: `s.len()`, `s.contains("x")`, `s.to_upper()`
//! - Math: `+`, `-`, `*`, `/`, `%`
//! - Conditionals: `if x > 0 { "pos" } else { "neg" }`
//! - Loops: `for item in arr { ... }`
//!
//! Full docs: <https://rhai.rs/book/>

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

/// Sandboxed script execution node (Dify-compatible, Rhai engine).
pub struct CodeNode;

#[async_trait]
impl Node for CodeNode {
    fn node_type(&self) -> &str {
        "code"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let language = ctx.data["language"].as_str().unwrap_or("rhai");
        if language != "rhai" {
            return Err(FlowError::InvalidDefinition(format!(
                "code: unsupported language '{language}' — only 'rhai' is supported"
            )));
        }

        let code = ctx.data["code"]
            .as_str()
            .ok_or_else(|| FlowError::InvalidDefinition("code: missing data.code".into()))?;

        let mut engine = rhai::Engine::new();
        engine.set_max_operations(100_000);
        engine.set_max_string_size(1_000_000);
        engine.set_max_array_size(10_000);

        let mut scope = rhai::Scope::new();

        let inputs_dyn = rhai::serde::to_dynamic(&ctx.inputs)
            .map_err(|e| FlowError::Internal(format!("code: failed to serialize inputs: {e}")))?;
        scope.push_dynamic("inputs", inputs_dyn);

        let vars_dyn = rhai::serde::to_dynamic(&ctx.variables).map_err(|e| {
            FlowError::Internal(format!("code: failed to serialize variables: {e}"))
        })?;
        scope.push_dynamic("variables", vars_dyn);

        let result: rhai::Dynamic = engine
            .eval_with_scope(&mut scope, code)
            .map_err(|e| FlowError::Internal(format!("code: script error: {e}")))?;

        // Object maps become the output directly; anything else wraps in {"output": ...}.
        let output: Value = rhai::serde::from_dynamic(&result)
            .map_err(|e| FlowError::Internal(format!("code: result serialization failed: {e}")))?;

        if output.is_object() {
            Ok(output)
        } else {
            Ok(json!({ "output": output }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ctx(inputs: HashMap<String, Value>, code: &str) -> ExecContext {
        ExecContext {
            data: json!({ "language": "rhai", "code": code }),
            inputs,
            variables: HashMap::new(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn scalar_wrapped_in_output() {
        let out = CodeNode.execute(ctx(HashMap::new(), "42")).await.unwrap();
        assert_eq!(out["output"], 42);
    }

    #[tokio::test]
    async fn bool_result() {
        let out = CodeNode
            .execute(ctx(HashMap::new(), "1 + 1 == 2"))
            .await
            .unwrap();
        assert_eq!(out["output"], true);
    }

    #[tokio::test]
    async fn object_map_returned_directly() {
        let out = CodeNode
            .execute(ctx(HashMap::new(), "#{ x: 1, y: 2 }"))
            .await
            .unwrap();
        assert_eq!(out["x"], 1);
        assert_eq!(out["y"], 2);
        assert!(out.get("output").is_none());
    }

    #[tokio::test]
    async fn inputs_accessible_in_script() {
        let out = CodeNode
            .execute(ctx(
                HashMap::from([("fetch".into(), json!({ "status": 200 }))]),
                "inputs.fetch.status == 200",
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], true);
    }

    #[tokio::test]
    async fn rejects_unsupported_language() {
        let err = CodeNode
            .execute(ExecContext {
                data: json!({ "language": "python", "code": "pass" }),
                inputs: HashMap::new(),
                variables: HashMap::new(),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn rejects_missing_code() {
        let err = CodeNode
            .execute(ExecContext {
                data: json!({ "language": "rhai" }),
                inputs: HashMap::new(),
                variables: HashMap::new(),
                ..Default::default()
            })
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn script_error_returns_internal() {
        let err = CodeNode
            .execute(ctx(HashMap::new(), "undefined_fn()"))
            .await
            .unwrap_err();
        assert!(matches!(err, FlowError::Internal(_)));
    }

    #[tokio::test]
    async fn string_manipulation() {
        let out = CodeNode
            .execute(ctx(
                HashMap::from([("msg".into(), json!({ "text": "hello" }))]),
                r#"let t = inputs.msg.text; t.to_upper()"#,
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], "HELLO");
    }
}
