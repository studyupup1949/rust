//! Built-in `"noop"` node — passes all upstream inputs through unchanged.
//!
//! Useful as a placeholder node, a fan-in join point, or for testing.

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::node::{ExecContext, Node};

/// A no-operation node that merges all upstream inputs into a single object
/// and returns it as output.
///
/// If there are no inputs the output is an empty JSON object `{}`.
pub struct NoopNode;

#[async_trait]
impl Node for NoopNode {
    fn node_type(&self) -> &str {
        "noop"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        // Merge all upstream inputs into one flat object.
        let mut merged = json!({});
        for (key, value) in &ctx.inputs {
            merged[key] = value.clone();
        }
        Ok(merged)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[tokio::test]
    async fn merges_inputs() {
        let node = NoopNode;
        let ctx = ExecContext {
            data: Value::Null,
            inputs: HashMap::from([("a".into(), json!({"x": 1})), ("b".into(), json!({"y": 2}))]),
            variables: HashMap::new(),
            ..Default::default()
        };
        let out = node.execute(ctx).await.unwrap();
        assert_eq!(out["a"]["x"], 1);
        assert_eq!(out["b"]["y"], 2);
    }

    #[tokio::test]
    async fn empty_inputs_returns_empty_object() {
        let node = NoopNode;
        let ctx = ExecContext {
            data: Value::Null,
            inputs: HashMap::new(),
            variables: HashMap::new(),
            ..Default::default()
        };
        let out = node.execute(ctx).await.unwrap();
        assert!(out.is_object());
        assert!(out.as_object().unwrap().is_empty());
    }
}
