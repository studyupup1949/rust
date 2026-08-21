//! `"start"` node — Dify-compatible flow entry point with input declaration.
//!
//! Declares the flow's expected input variables, applies defaults for any that
//! are absent, and emits the resolved set as the node's output. Place a single
//! `"start"` node at the root of your DAG; downstream nodes access inputs via
//! `ctx.inputs["start"]["variable_name"]`.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "inputs": [
//!     { "name": "query",      "type": "string"  },
//!     { "name": "max_tokens", "type": "number",  "default": 256 },
//!     { "name": "verbose",    "type": "bool",    "default": false }
//!   ]
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|----------|-------------|
//! | `inputs` | array | — | Input variable declarations (empty = pass all variables through) |
//! | `inputs[].name` | string | ✅ | Variable name |
//! | `inputs[].type` | `"string"` \| `"number"` \| `"bool"` \| `"object"` \| `"array"` | — | Expected type (used for validation) |
//! | `inputs[].default` | any | — | Value used when the variable is absent from the flow's `variables` map |
//!
//! # Output schema
//!
//! The output is a JSON object containing the resolved value for each declared
//! input. If no `inputs` array is declared, the output is the entire
//! `variables` map passed to the engine.
//!
//! # Errors
//!
//! - [`FlowError::InvalidDefinition`] — `inputs` is present but not a valid array.
//! - [`FlowError::Internal`] — a required input (no default) is missing from the
//!   flow's `variables` map.

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

#[derive(Debug, Deserialize)]
struct InputDecl {
    name: String,
    #[serde(rename = "type", default)]
    type_hint: String,
    default: Option<Value>,
}

pub struct StartNode;

#[async_trait]
impl Node for StartNode {
    fn node_type(&self) -> &str {
        "start"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let decls_val = ctx.data.get("inputs");

        // No declaration → pass all variables through.
        let Some(decls_val) = decls_val else {
            return Ok(Value::Object(
                ctx.variables.into_iter().collect(),
            ));
        };

        let decls: Vec<InputDecl> = serde_json::from_value(decls_val.clone()).map_err(|e| {
            FlowError::InvalidDefinition(format!("start node: invalid inputs declaration: {e}"))
        })?;

        if decls.is_empty() {
            return Ok(Value::Object(ctx.variables.into_iter().collect()));
        }

        let mut output = serde_json::Map::new();
        for decl in &decls {
            let val = match ctx.variables.get(&decl.name).cloned() {
                Some(v) => {
                    validate_type(&decl.name, &v, &decl.type_hint)?;
                    v
                }
                None => {
                    decl.default.clone().ok_or_else(|| {
                        FlowError::Internal(format!(
                            "start node: required input '{}' not provided",
                            decl.name
                        ))
                    })?
                }
            };
            output.insert(decl.name.clone(), val);
        }

        Ok(Value::Object(output))
    }
}

/// Light type validation — checks the JSON value kind matches the declared type.
///
/// Silently passes if `type_hint` is empty or unrecognised.
fn validate_type(name: &str, value: &Value, type_hint: &str) -> Result<()> {
    let ok = match type_hint {
        "string" => value.is_string(),
        "number" => value.is_number(),
        "bool" => value.is_boolean(),
        "object" => value.is_object(),
        "array" => value.is_array(),
        _ => true, // unknown or empty hint — no validation
    };

    if !ok {
        return Err(FlowError::InvalidDefinition(format!(
            "start node: input '{name}' expected type '{type_hint}', got {}",
            value_kind(value)
        )));
    }
    Ok(())
}

fn value_kind(v: &Value) -> &'static str {
    match v {
        Value::Null => "null",
        Value::Bool(_) => "bool",
        Value::Number(_) => "number",
        Value::String(_) => "string",
        Value::Array(_) => "array",
        Value::Object(_) => "object",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn ctx(variables: HashMap<String, Value>, data: Value) -> ExecContext {
        ExecContext {
            data,
            variables,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn no_declaration_returns_all_variables() {
        let node = StartNode;
        let vars = HashMap::from([("x".into(), json!(1)), ("y".into(), json!("hello"))]);
        let out = node.execute(ctx(vars, json!({}))).await.unwrap();
        assert_eq!(out["x"], json!(1));
        assert_eq!(out["y"], json!("hello"));
    }

    #[tokio::test]
    async fn declared_inputs_resolved_from_variables() {
        let node = StartNode;
        let vars = HashMap::from([("q".into(), json!("hello"))]);
        let out = node
            .execute(ctx(
                vars,
                json!({ "inputs": [{ "name": "q", "type": "string" }] }),
            ))
            .await
            .unwrap();
        assert_eq!(out["q"], json!("hello"));
    }

    #[tokio::test]
    async fn default_applied_when_variable_absent() {
        let node = StartNode;
        let out = node
            .execute(ctx(
                HashMap::new(),
                json!({
                    "inputs": [
                        { "name": "n", "type": "number", "default": 42 }
                    ]
                }),
            ))
            .await
            .unwrap();
        assert_eq!(out["n"], json!(42));
    }

    #[tokio::test]
    async fn missing_required_input_returns_error() {
        let node = StartNode;
        let result = node
            .execute(ctx(
                HashMap::new(),
                json!({ "inputs": [{ "name": "required_var", "type": "string" }] }),
            ))
            .await;
        assert!(matches!(result, Err(FlowError::Internal(_))));
    }

    #[tokio::test]
    async fn type_mismatch_returns_invalid_definition() {
        let node = StartNode;
        let vars = HashMap::from([("x".into(), json!(42))]);
        let result = node
            .execute(ctx(
                vars,
                json!({ "inputs": [{ "name": "x", "type": "string" }] }),
            ))
            .await;
        assert!(matches!(result, Err(FlowError::InvalidDefinition(_))));
    }

    #[tokio::test]
    async fn unknown_type_hint_is_accepted() {
        let node = StartNode;
        let vars = HashMap::from([("x".into(), json!(true))]);
        let out = node
            .execute(ctx(
                vars,
                json!({ "inputs": [{ "name": "x", "type": "custom_type" }] }),
            ))
            .await
            .unwrap();
        assert_eq!(out["x"], json!(true));
    }

    #[tokio::test]
    async fn empty_inputs_array_returns_all_variables() {
        let node = StartNode;
        let vars = HashMap::from([("a".into(), json!(1))]);
        let out = node
            .execute(ctx(vars, json!({ "inputs": [] })))
            .await
            .unwrap();
        assert_eq!(out["a"], json!(1));
    }
}
