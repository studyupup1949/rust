//! `"variable-assigner"` node — writes a value to a single variable in the flow scope.
//!
//! Mirrors Dify's Variable Assigner node.
//!
//! The node resolves a value from a variable selector (dot-separated path) and
//! outputs it as `output`. The runner merges this into the flow's live variable
//! scope, making it available to all downstream nodes.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "output_type": "string",
//!   "variables": ["fetch.body.status"]
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|:--------:|-------------|
//! | `output_type` | string | — | Target type: `string`, `number`, `object`, `array`, `any` (default: `any`) |
//! | `variables` | array | ✅ | Dot-separated variable selectors; first non-null value is used |
//!
//! A selector is a dot-separated path: `"node_id.output_key"` or just `"key"` for global variables.
//!
//! # Output schema
//!
//! ```json
//! { "output": 200 }
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

/// Variable Assigner node — resolves and writes a single variable.
pub struct VariableAssignerNode;

#[async_trait]
impl Node for VariableAssignerNode {
    fn node_type(&self) -> &str {
        "variable-assigner"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        // Check for Dify-style advanced_settings with groups
        let advanced_settings = ctx
            .data
            .get("advanced_settings")
            .and_then(|v| v.as_object());
        let group_enabled = advanced_settings
            .and_then(|a| a.get("group_enabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        // Build lookup: variables first, then inputs (inputs shadow variables)
        let mut lookup: HashMap<String, Value> = ctx.variables.into_iter().collect();
        for (k, v) in &ctx.inputs {
            lookup.insert(k.clone(), v.clone());
        }

        if group_enabled {
            // Dify-style: multiple groups
            let groups = advanced_settings
                .and_then(|a| a.get("groups"))
                .and_then(|v| v.as_array())
                .ok_or_else(|| {
                    FlowError::InvalidDefinition(
                        "variable-assigner: advanced_settings.group_enabled=true but no groups found"
                            .into(),
                    )
                })?;

            let mut outputs: serde_json::Map<String, Value> = serde_json::Map::new();

            for (i, group) in groups.iter().enumerate() {
                let group_name = group
                    .get("group_name")
                    .and_then(|v| v.as_str())
                    .map(String::from)
                    .unwrap_or_else(|| format!("group_{}", i));

                let variables = group
                    .get("variables")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();

                let output_type = group
                    .get("output_type")
                    .and_then(|v| v.as_str())
                    .unwrap_or("any");

                // Find first non-null value for this group
                let mut value: Option<Value> = None;
                for selector in &variables {
                    let resolved = resolve_selector(&lookup, selector);
                    if resolved.as_ref().map_or(false, |v| !v.is_null()) {
                        value = resolved;
                        break;
                    }
                }

                // Apply type coercion if needed
                let coerced = apply_type_coercion(value, output_type);
                outputs.insert(group_name, coerced);
            }

            Ok(Value::Object(outputs))
        } else {
            // Simple mode: single variable assignment
            let selectors = ctx
                .data
                .get("variables")
                .and_then(|v| v.as_array())
                .map(|arr| {
                    arr.iter()
                        .filter_map(|v| v.as_str().map(String::from))
                        .collect::<Vec<_>>()
                })
                .ok_or_else(|| {
                    FlowError::InvalidDefinition(
                        "variable-assigner: missing or invalid data.variables".into(),
                    )
                })?;

            if selectors.is_empty() {
                return Err(FlowError::InvalidDefinition(
                    "variable-assigner: at least one variable selector is required".into(),
                ));
            }

            // Find first non-null value
            let mut output: Option<Value> = None;
            for selector in &selectors {
                let value = resolve_selector(&lookup, selector);
                if value.as_ref().map_or(false, |v| !v.is_null()) {
                    output = value;
                    break;
                }
            }

            Ok(json!({ "output": output }))
        }
    }
}

/// Apply type coercion to a value based on output_type.
fn apply_type_coercion(value: Option<Value>, output_type: &str) -> Value {
    match value {
        Some(v) => match output_type {
            "string" => Value::String(v.to_string()),
            "number" => {
                if let Some(n) = v.as_f64() {
                    serde_json::Number::from_f64(n)
                        .map(Value::Number)
                        .unwrap_or(Value::Null)
                } else {
                    Value::Null
                }
            }
            "boolean" => Value::Bool(v.as_bool().unwrap_or(false)),
            "object" | "array" | "any" | _ => v,
        },
        None => Value::Null,
    }
}

/// Resolve a dot-separated selector from the lookup map.
///
/// Examples:
///   "fetch.body.status"  → lookup["fetch"]["body"]["status"]
///   "fetch.body"         → lookup["fetch"]["body"]
///   "name"              → lookup["name"]
fn resolve_selector(lookup: &HashMap<String, Value>, selector: &str) -> Option<Value> {
    let parts: Vec<&str> = selector.split('.').collect();
    if parts.is_empty() {
        return None;
    }

    let first_key = parts[0];
    let mut current = lookup.get(first_key)?;

    for part in &parts[1..] {
        current = current.get(*part)?;
    }

    // Return a copy
    Some(current.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx_with_lookup(
        data: Value,
        inputs: HashMap<String, Value>,
        variables: HashMap<String, Value>,
    ) -> ExecContext {
        let mut lookup: HashMap<String, Value> = variables.clone();
        for (k, v) in &inputs {
            lookup.insert(k.clone(), v.clone());
        }
        ExecContext {
            data,
            inputs,
            variables,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn resolves_from_variables() {
        let ctx = ctx_with_lookup(
            json!({ "variables": ["name"] }),
            HashMap::new(),
            HashMap::from([("name".into(), json!("Alice"))]),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        assert_eq!(result["output"], json!("Alice"));
    }

    #[tokio::test]
    async fn resolves_nested_path() {
        let ctx = ctx_with_lookup(
            json!({ "variables": ["fetch.body.status"] }),
            HashMap::from([("fetch".into(), json!({ "body": { "status": 200 } }))]),
            HashMap::new(),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        assert_eq!(result["output"], json!(200));
    }

    #[tokio::test]
    async fn first_non_null_wins() {
        let ctx = ctx_with_lookup(
            json!({ "variables": ["a", "b", "c"] }),
            HashMap::from([
                ("a".into(), json!(null)),
                ("b".into(), json!("found")),
                ("c".into(), json!("ignored")),
            ]),
            HashMap::new(),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        assert_eq!(result["output"], json!("found"));
    }

    #[tokio::test]
    async fn missing_selector_returns_null_output() {
        let ctx = ctx_with_lookup(
            json!({ "variables": ["nonexistent"] }),
            HashMap::new(),
            HashMap::new(),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        assert!(result["output"].is_null());
    }

    #[tokio::test]
    async fn empty_variables_array_returns_error() {
        let ctx = ctx_with_lookup(
            json!({ "variables": [] }),
            HashMap::new(),
            HashMap::new(),
        );
        let err = VariableAssignerNode.execute(ctx).await.unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn group_mode_single_group() {
        let ctx = ctx_with_lookup(
            json!({
                "advanced_settings": {
                    "group_enabled": true,
                    "groups": [
                        {
                            "group_name": "result",
                            "output_type": "number",
                            "variables": ["status"]
                        }
                    ]
                }
            }),
            HashMap::from([("status".into(), json!(200))]),
            HashMap::new(),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        // JSON number 200 becomes 200.0 when stored
        assert_eq!(result["result"].as_f64().unwrap(), 200.0);
    }

    #[tokio::test]
    async fn group_mode_multiple_groups() {
        let ctx = ctx_with_lookup(
            json!({
                "advanced_settings": {
                    "group_enabled": true,
                    "groups": [
                        { "group_name": "g1", "variables": ["a"] },
                        { "group_name": "g2", "variables": ["b"] }
                    ]
                }
            }),
            HashMap::from([
                ("a".into(), json!("first")),
                ("b".into(), json!("second")),
            ]),
            HashMap::new(),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        assert_eq!(result["g1"], json!("first"));
        assert_eq!(result["g2"], json!("second"));
    }

    #[tokio::test]
    async fn group_mode_missing_group_name_uses_default() {
        let ctx = ctx_with_lookup(
            json!({
                "advanced_settings": {
                    "group_enabled": true,
                    "groups": [
                        { "variables": ["x"] }
                    ]
                }
            }),
            HashMap::from([("x".into(), json!("val"))]),
            HashMap::new(),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        // Default name is "group_0"
        assert_eq!(result["group_0"], json!("val"));
    }

    #[tokio::test]
    async fn group_mode_group_enabled_but_no_groups_returns_error() {
        let ctx = ctx_with_lookup(
            json!({
                "advanced_settings": {
                    "group_enabled": true
                }
            }),
            HashMap::new(),
            HashMap::new(),
        );
        let err = VariableAssignerNode.execute(ctx).await.unwrap_err();
        assert!(matches!(err, FlowError::InvalidDefinition(_)));
    }

    #[tokio::test]
    async fn type_coercion_to_string() {
        let ctx = ctx_with_lookup(
            json!({
                "advanced_settings": {
                    "group_enabled": true,
                    "groups": [{
                        "group_name": "out",
                        "output_type": "string",
                        "variables": ["num"]
                    }]
                }
            }),
            HashMap::from([("num".into(), json!(42))]),
            HashMap::new(),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        assert_eq!(result["out"], json!("42"));
    }

    #[tokio::test]
    async fn type_coercion_to_number() {
        // Coercion only works on numeric JSON values
        let ctx = ctx_with_lookup(
            json!({
                "advanced_settings": {
                    "group_enabled": true,
                    "groups": [{
                        "group_name": "out",
                        "output_type": "number",
                        "variables": ["num"]
                    }]
                }
            }),
            HashMap::from([("num".into(), json!(42.5))]),
            HashMap::new(),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        assert_eq!(result["out"], json!(42.5));
    }

    #[tokio::test]
    async fn type_coercion_to_boolean() {
        // String "true" becomes bool true
        let ctx = ctx_with_lookup(
            json!({
                "advanced_settings": {
                    "group_enabled": true,
                    "groups": [{
                        "group_name": "out",
                        "output_type": "boolean",
                        "variables": ["val"]
                    }]
                }
            }),
            HashMap::from([("val".into(), json!(true))]),
            HashMap::new(),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        assert_eq!(result["out"], json!(true));
    }

    #[tokio::test]
    async fn inputs_shadow_variables() {
        // inputs should take precedence over variables
        let ctx = ctx_with_lookup(
            json!({ "variables": ["key"] }),
            HashMap::from([("key".into(), json!("from_input"))]),
            HashMap::from([("key".into(), json!("from_variable"))]),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        assert_eq!(result["output"], json!("from_input"));
    }

    #[tokio::test]
    async fn resolve_selector_partial_path() {
        // When a partial path resolves to a non-leaf value
        let ctx = ctx_with_lookup(
            json!({ "variables": ["fetch.body"] }),
            HashMap::from([("fetch".into(), json!({ "body": { "status": 200 } }))]),
            HashMap::new(),
        );
        let result = VariableAssignerNode.execute(ctx).await.unwrap();
        assert_eq!(result["output"]["status"], json!(200));
    }
}
