//! `"iteration-start"` node — control marker for the start of an iteration sub-block.
//!
//! This node is a pure control-flow marker with no configuration and no execution
//! logic. It marks the entry point of an iteration sub-block within a Dify-compatible
//! DAG. The actual iteration logic (iterating over an array and running a sub-flow
//! for each item) lives in the parent `"iteration"` node.
//!
//! The DAG parser treats `iteration-start` as a single-entry node that passes all
//! inputs through to its single output, acting as a transparent anchor for the
//! iteration sub-block boundary.
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

/// Iteration Start control node — marks the entry point of an iteration sub-block.
pub struct IterationStartNode;

#[async_trait]
impl Node for IterationStartNode {
    fn node_type(&self) -> &str {
        "iteration-start"
    }

    async fn execute(&self, _ctx: ExecContext) -> Result<Value> {
        // This is a pure control marker — it has no configuration and no logic.
        // It simply marks the start of an iteration block and passes through inputs.
        // The parent "iteration" node handles the actual iteration logic.
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
    async fn iteration_start_has_no_config() {
        let node = IterationStartNode;
        assert_eq!(node.node_type(), "iteration-start");
        let out = node.execute(ctx()).await.unwrap();
        // Passes through with null output (iteration parent handles the real output)
        assert!(out["output"].is_null());
    }
}
