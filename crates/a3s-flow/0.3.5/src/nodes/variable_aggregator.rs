//! Built-in `"variable-aggregator"` node — collects outputs from multiple
//! upstream branches and returns the first non-null value.
//!
//! Mirrors Dify's Variable Aggregator node. Typical use: merge the outputs
//! of an `"if-else"` fan-out back into a single value for downstream nodes.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "inputs": ["branch_ok", "branch_error"]
//! }
//! ```
//!
//! | Field | Type | Description |
//! |-------|------|-------------|
//! | `inputs` | string[] | Optional. Try these node IDs in order. If omitted, all inputs are tried in alphabetical key order. |
//!
//! # Output schema
//!
//! ```json
//! { "output": { "body": "..." } }
//! ```
//!
//! Returns `{ "output": null }` when all upstream values are `null` (all
//! branches were skipped).
//!
//! # Example
//!
//! ```json
//! {
//!   "nodes": [
//!     { "id": "route",  "type": "if-else", "data": { ... } },
//!     { "id": "path_a", "type": "http-request", "data": { "run_if": { ... } } },
//!     { "id": "path_b", "type": "http-request", "data": { "run_if": { ... } } },
//!     { "id": "merge",  "type": "variable-aggregator", "data": { "inputs": ["path_a", "path_b"] } }
//!   ],
//!   "edges": [
//!     { "source": "route",  "target": "path_a" },
//!     { "source": "route",  "target": "path_b" },
//!     { "source": "path_a", "target": "merge" },
//!     { "source": "path_b", "target": "merge" }
//!   ]
//! }
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::node::{ExecContext, Node};

/// Variable aggregator node (Dify-compatible).
pub struct VariableAggregatorNode;

#[async_trait]
impl Node for VariableAggregatorNode {
    fn node_type(&self) -> &str {
        "variable-aggregator"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        // Determine the order to try inputs.
        let order: Option<Vec<String>> = ctx
            .data
            .get("inputs")
            .and_then(|v| serde_json::from_value(v.clone()).ok());

        let first_non_null = if let Some(keys) = order {
            keys.iter()
                .find_map(|k| ctx.inputs.get(k).filter(|v| !v.is_null()))
                .cloned()
        } else {
            // Alphabetical order for determinism when no explicit order given.
            let mut keys: Vec<&String> = ctx.inputs.keys().collect();
            keys.sort();
            keys.into_iter()
                .find_map(|k| ctx.inputs.get(k).filter(|v| !v.is_null()))
                .cloned()
        };

        Ok(json!({ "output": first_non_null }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ctx(inputs: HashMap<String, Value>, data: Value) -> ExecContext {
        ExecContext {
            data,
            inputs,
            variables: HashMap::new(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn returns_first_non_null_in_explicit_order() {
        let node = VariableAggregatorNode;
        let out = node
            .execute(ctx(
                HashMap::from([
                    ("a".into(), json!(null)),
                    ("b".into(), json!({ "v": 42 })),
                    ("c".into(), json!({ "v": 99 })),
                ]),
                json!({ "inputs": ["a", "b", "c"] }),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"]["v"], 42);
    }

    #[tokio::test]
    async fn returns_null_when_all_skipped() {
        let node = VariableAggregatorNode;
        let out = node
            .execute(ctx(
                HashMap::from([("a".into(), json!(null)), ("b".into(), json!(null))]),
                json!({ "inputs": ["a", "b"] }),
            ))
            .await
            .unwrap();
        assert!(out["output"].is_null());
    }

    #[tokio::test]
    async fn alphabetical_order_when_no_config() {
        let node = VariableAggregatorNode;
        let out = node
            .execute(ctx(
                HashMap::from([
                    ("z".into(), json!("last")),
                    ("a".into(), json!(null)),
                    ("m".into(), json!("first_non_null")),
                ]),
                json!({}),
            ))
            .await
            .unwrap();
        assert_eq!(out["output"], "first_non_null");
    }
}
