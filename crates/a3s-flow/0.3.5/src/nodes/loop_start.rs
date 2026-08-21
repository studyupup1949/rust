//! `"loop-start"` node — control marker for the start of a loop sub-block.
//!
//! This node is a pure control-flow marker with no configuration and no execution
//! logic. It marks the entry point of a loop sub-block within a Dify-compatible
//! DAG. The actual loop logic (iterating until a condition is met) lives in the
//! parent `"loop"` node.
//!
//! The DAG parser treats `loop-start` as a single-entry node that passes all
//! inputs through to its single output, acting as a transparent anchor for the
//! loop sub-block boundary.
//!
//! # Config schema
//!
//! ```json
//! {}
//! ```
//!
//! # Output schema
//!
//! Passes through all inputs unchanged:
//! ```json
//! { "output": <passthrough> }
//! ```

use async_trait::async_trait;
use serde_json::{json, Value};

use crate::error::Result;
use crate::node::{ExecContext, Node};

/// Loop Start control node — marks the entry point of a loop sub-block.
pub struct LoopStartNode;

#[async_trait]
impl Node for LoopStartNode {
    fn node_type(&self) -> &str {
        "loop-start"
    }

    async fn execute(&self, _ctx: ExecContext) -> Result<Value> {
        // This is a pure control marker — it has no configuration and no logic.
        // It simply marks the start of a loop block and passes through inputs.
        // The parent "loop" node handles the actual loop logic.
        Ok(json!({ "output": null }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn ctx() -> ExecContext {
        ExecContext {
            data: json!({}),
            inputs: HashMap::new(),
            variables: HashMap::new(),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn loop_start_has_no_config() {
        let node = LoopStartNode;
        assert_eq!(node.node_type(), "loop-start");
        let out = node.execute(ctx()).await.unwrap();
        assert!(out["output"].is_null());
    }
}
