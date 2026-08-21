//! `"loop-end"` node — control marker for the end of a loop sub-block.
//!
//! This node is a pure control-flow marker with no configuration and no execution
//! logic. It marks the exit point of a loop sub-block within a Dify-compatible
//! DAG. The actual loop logic lives in the parent `"loop"` node.
//!
//! The DAG parser treats `loop-end` as a single-exit node that collects the
//! output from the loop body and returns it to the parent loop node for
//! condition evaluation.
//!
//! # Config schema
//!
//! ```json
//! {}
//! ```
//!
//! # Output schema
//!
//! ```json
//! { "output": <loop_body_result> }
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::node::{ExecContext, Node};

/// Loop End control node — marks the exit point of a loop sub-block.
pub struct LoopEndNode;

#[async_trait]
impl Node for LoopEndNode {
    fn node_type(&self) -> &str {
        "loop-end"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        // Collect the output from the loop body (passed via inputs from the last node in the loop body).
        // The loop condition check happens in the parent "loop" node.
        // For now, pass through the first available input as the loop result.
        let output = ctx.inputs.values().next().cloned().unwrap_or(Value::Null);

        Ok(json!({ "output": output }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ctx_with_input(input: Value) -> ExecContext {
        ExecContext {
            data: json!({}),
            inputs: HashMap::from([("body".into(), input)]),
            variables: HashMap::new(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn loop_end_returns_input() {
        let node = LoopEndNode;
        assert_eq!(node.node_type(), "loop-end");
        let out = node
            .execute(ctx_with_input(json!({ "result": "done" })))
            .await
            .unwrap();
        assert_eq!(out["output"]["result"], "done");
    }

    #[tokio::test]
    async fn loop_end_returns_null_when_no_input() {
        let node = LoopEndNode;
        let out = node
            .execute(ExecContext {
                data: json!({}),
                inputs: HashMap::new(),
                variables: HashMap::new(),
                ..Default::default()
            })
            .await
            .unwrap();
        assert!(out["output"].is_null());
    }
}
