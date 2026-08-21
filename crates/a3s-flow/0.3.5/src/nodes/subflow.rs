//! `"sub-flow"` node — execute a named flow as an inline step.
//!
//! Loads the named flow definition from the engine's [`FlowStore`], parses it,
//! and runs it synchronously as part of the parent flow's wave. The sub-flow
//! inherits the parent's node registry and variables; the `data["variables"]`
//! map (if present) extends or overrides them.
//!
//! The node output is a JSON object whose keys are the sub-flow's node IDs and
//! whose values are those nodes' outputs — identical in shape to
//! [`FlowResult::outputs`](crate::result::FlowResult::outputs).
//!
//! # Flow definition example
//!
//! ```json
//! {
//!   "id": "call-summarizer",
//!   "type": "sub-flow",
//!   "data": {
//!     "name": "summarizer-flow",
//!     "variables": { "max_tokens": 256 }
//!   }
//! }
//! ```
//!
//! # Errors
//!
//! - [`FlowError::Internal`] — no [`FlowStore`] is configured on the engine.
//! - [`FlowError::FlowNotFound`] — no flow with `name` exists in the store.
//! - Any error propagated from the sub-flow's own execution.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::{FlowError, Result};
use crate::graph::DagGraph;
use crate::node::{ExecContext, Node};
use crate::runner::FlowRunner;

pub struct SubFlowNode;

#[async_trait]
impl Node for SubFlowNode {
    fn node_type(&self) -> &str {
        "sub-flow"
    }

    async fn execute(&self, ctx: ExecContext) -> Result<Value> {
        let name = ctx
            .data
            .get("name")
            .and_then(|v| v.as_str())
            .ok_or_else(|| {
                FlowError::InvalidDefinition(
                    "sub-flow node requires a \"name\" string in data".into(),
                )
            })?;

        let store = ctx.flow_store.as_ref().ok_or_else(|| {
            FlowError::Internal(
                "sub-flow node requires a FlowStore; configure one via \
                 FlowEngine::with_flow_store"
                    .into(),
            )
        })?;

        let definition = store
            .load(name)
            .await?
            .ok_or_else(|| FlowError::FlowNotFound(name.to_string()))?;

        let dag = DagGraph::from_json(&definition)?;

        // Inherit parent variables; let data["variables"] extend/override them.
        let mut variables: HashMap<String, Value> = ctx.variables.clone();
        if let Some(overrides) = ctx.data.get("variables").and_then(|v| v.as_object()) {
            for (k, v) in overrides {
                variables.insert(k.clone(), v.clone());
            }
        }

        let mut runner = FlowRunner::with_arc_registry(dag, Arc::clone(&ctx.registry));
        if let Some(fs) = ctx.flow_store {
            runner = runner.with_flow_store(fs);
        }

        let result = runner.run(variables).await?;

        // Return the sub-flow's per-node outputs as a single JSON object.
        Ok(Value::Object(result.outputs.into_iter().collect()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::flow_store::{FlowStore, MemoryFlowStore};
    use serde_json::json;
    use std::sync::Arc;

    #[tokio::test]
    async fn sub_flow_runs_named_flow_and_returns_outputs() {
        let store = Arc::new(MemoryFlowStore::new());
        let def = json!({
            "nodes": [{ "id": "x", "type": "noop" }],
            "edges": []
        });
        store.save("inner", &def).await.unwrap();

        let ctx = ExecContext {
            data: json!({ "name": "inner" }),
            flow_store: Some(store),
            ..Default::default()
        };

        let node = SubFlowNode;
        let output = node.execute(ctx).await.unwrap();
        // noop returns its inputs (empty here), so output["x"] == {}
        assert!(output.get("x").is_some());
    }

    #[tokio::test]
    async fn sub_flow_missing_name_returns_error() {
        let ctx = ExecContext {
            data: json!({}),
            ..Default::default()
        };
        let result = SubFlowNode.execute(ctx).await;
        assert!(matches!(result, Err(FlowError::InvalidDefinition(_))));
    }

    #[tokio::test]
    async fn sub_flow_no_store_returns_internal_error() {
        let ctx = ExecContext {
            data: json!({ "name": "any" }),
            flow_store: None,
            ..Default::default()
        };
        let result = SubFlowNode.execute(ctx).await;
        assert!(matches!(result, Err(FlowError::Internal(_))));
    }

    #[tokio::test]
    async fn sub_flow_unknown_name_returns_flow_not_found() {
        let store = Arc::new(MemoryFlowStore::new());
        let ctx = ExecContext {
            data: json!({ "name": "nonexistent" }),
            flow_store: Some(store),
            ..Default::default()
        };
        let result = SubFlowNode.execute(ctx).await;
        assert!(matches!(result, Err(FlowError::FlowNotFound(_))));
    }

    #[tokio::test]
    async fn sub_flow_inherits_and_overrides_variables() {
        use crate::node::Node as _;
        use crate::registry::NodeRegistry;

        // Inner flow: a "code" node that reads a variable and returns it.
        let store = Arc::new(MemoryFlowStore::new());
        let def = json!({
            "nodes": [{
                "id": "read",
                "type": "code",
                "data": {
                    "language": "rhai",
                    "code": "let v = variables[\"x\"]; #{result: v}"
                }
            }],
            "edges": []
        });
        store.save("var-flow", &def).await.unwrap();

        let mut parent_vars = HashMap::new();
        parent_vars.insert("x".to_string(), json!(10));

        let ctx = ExecContext {
            data: json!({
                "name": "var-flow",
                "variables": { "x": 42 }   // override parent's x=10
            }),
            variables: parent_vars,
            flow_store: Some(store),
            registry: Arc::new(NodeRegistry::with_defaults()),
            ..Default::default()
        };

        let output = SubFlowNode.execute(ctx).await.unwrap();
        assert_eq!(output["read"]["result"], json!(42));
    }
}
