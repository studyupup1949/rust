//! `"end"` node — Dify-compatible flow output collector.
//!
//! Gathers values from upstream node outputs and returns them as a named
//! output map. Place an `"end"` node at the sink of your DAG; its output
//! in `FlowResult::outputs["end"]` represents the flow's final answer.
//!
//! Values are located using JSON pointer strings
//! (`/upstream_node_id/field/nested`) resolved against the set of upstream
//! node outputs available via `ctx.inputs`.
//!
//! # Config schema
//!
//! ```json
//! {
//!   "outputs": {
//!     "answer":       "/llm/text",
//!     "total_tokens": "/llm/usage/total_tokens",
//!     "raw_result":   "/transform"
//!   }
//! }
//! ```
//!
//! | Field | Type | Required | Description |
//! |-------|------|----------|-------------|
//! | `outputs` | object | — | Map of output name → JSON pointer path. If absent, `ctx.inputs` is returned as-is. |
//!
//! The JSON pointer `/upstream_node/field` is resolved against a virtual
//! object whose top-level keys are the direct upstream node IDs. Missing
//! paths resolve to `null`.
//!
//! # Output schema
//!
//! ```json
//! { "answer": "...", "total_tokens": 512 }
//! ```
//!
//! # Example
//!
//! ```json
//! {
//!   "nodes": [
//!     { "id": "llm",   "type": "noop" },
//!     { "id": "end",   "type": "end",
//!       "data": { "outputs": { "reply": "/llm" } } }
//!   ],
//!   "edges": [{ "source": "llm", "target": "end" }]
//! }
//! ```

use async_trait::async_trait;
use serde_json::Value;

use crate::error::{FlowError, Result};
use crate::node::{ExecContext, Node};

pub struct EndNode;

#[async_trait]
impl Node for EndNode {
    fn node_type(&self) -> &str {
        "end"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let Some(outputs_cfg) = ctx.data.get("outputs") else {
            // No config → return all upstream inputs as-is.
            return Ok(Value::Object(ctx.inputs.into_iter().collect()));
        };

        let outputs_map = outputs_cfg.as_object().ok_or_else(|| {
            FlowError::InvalidDefinition(
                "end node: \"outputs\" must be a JSON object".into(),
            )
        })?;

        // Build a virtual object from ctx.inputs so JSON pointer resolution works.
        let source: Value = Value::Object(ctx.inputs.into_iter().collect());

        let mut result = serde_json::Map::new();
        for (name, ptr_val) in outputs_map {
            let ptr = ptr_val.as_str().ok_or_else(|| {
                FlowError::InvalidDefinition(format!(
                    "end node: output '{name}' path must be a string"
                ))
            })?;
            let value = source.pointer(ptr).cloned().unwrap_or(Value::Null);
            result.insert(name.clone(), value);
        }

        Ok(Value::Object(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::collections::HashMap;

    fn ctx(inputs: HashMap<String, Value>, data: Value) -> ExecContext {
        ExecContext {
            data,
            inputs,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn no_config_returns_all_inputs() {
        let node = EndNode;
        let inputs = HashMap::from([("a".into(), json!({ "x": 1 }))]);
        let out = node.execute(ctx(inputs, json!({}))).await.unwrap();
        assert_eq!(out["a"]["x"], json!(1));
    }

    #[tokio::test]
    async fn pointer_extracts_nested_field() {
        let node = EndNode;
        let inputs = HashMap::from([("llm".into(), json!({ "text": "hello", "usage": { "tokens": 42 } }))]);
        let out = node
            .execute(ctx(
                inputs,
                json!({ "outputs": { "reply": "/llm/text", "tokens": "/llm/usage/tokens" } }),
            ))
            .await
            .unwrap();
        assert_eq!(out["reply"], json!("hello"));
        assert_eq!(out["tokens"], json!(42));
    }

    #[tokio::test]
    async fn missing_path_resolves_to_null() {
        let node = EndNode;
        let inputs = HashMap::from([("a".into(), json!({}))]);
        let out = node
            .execute(ctx(
                inputs,
                json!({ "outputs": { "x": "/a/nonexistent" } }),
            ))
            .await
            .unwrap();
        assert!(out["x"].is_null());
    }

    #[tokio::test]
    async fn pointer_to_entire_upstream_node() {
        let node = EndNode;
        let upstream = json!({ "val": 99 });
        let inputs = HashMap::from([("step".into(), upstream.clone())]);
        let out = node
            .execute(ctx(
                inputs,
                json!({ "outputs": { "result": "/step" } }),
            ))
            .await
            .unwrap();
        assert_eq!(out["result"], upstream);
    }

    #[tokio::test]
    async fn invalid_outputs_config_returns_error() {
        let node = EndNode;
        let result = node
            .execute(ctx(
                HashMap::new(),
                json!({ "outputs": "not_an_object" }),
            ))
            .await;
        assert!(matches!(result, Err(FlowError::InvalidDefinition(_))));
    }

    #[tokio::test]
    async fn non_string_path_returns_error() {
        let node = EndNode;
        let result = node
            .execute(ctx(
                HashMap::new(),
                json!({ "outputs": { "x": 42 } }),
            ))
            .await;
        assert!(matches!(result, Err(FlowError::InvalidDefinition(_))));
    }
}
