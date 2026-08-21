//! [`FlowResult`] — the output of a completed flow execution.

use std::collections::{HashMap, HashSet};

use serde::{Deserialize, Serialize};
use serde_json::Value;
use uuid::Uuid;

/// Result of a complete or partial flow execution.
///
/// Returned by [`FlowRunner::run`](crate::runner::FlowRunner::run) and
/// [`FlowRunner::resume_from`](crate::runner::FlowRunner::resume_from).
/// Also stored by [`ExecutionStore`](crate::store::ExecutionStore) implementations.
///
/// `Serialize` / `Deserialize` are derived so that store implementations
/// (e.g. SQLite, Redis) can round-trip the result as JSON without extra glue code.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowResult {
    /// Unique ID for this execution run.
    pub execution_id: Uuid,
    /// Per-node outputs, keyed by node ID.
    ///
    /// Skipped nodes (whose `run_if` evaluated to false) are present with a
    /// `null` value; use `skipped_nodes` to distinguish them from nodes that
    /// genuinely produced `null`.
    pub outputs: HashMap<String, Value>,
    /// IDs of all nodes that were processed (executed or skipped).
    pub completed_nodes: HashSet<String>,
    /// IDs of nodes whose `run_if` guard evaluated to false and were skipped.
    pub skipped_nodes: HashSet<String>,
    /// Final snapshot of the shared mutable context after execution.
    ///
    /// Nodes may read and write this map via [`ExecContext::context`](crate::node::ExecContext::context)
    /// during execution. The final state is captured here when the flow completes.
    #[serde(default)]
    pub context: HashMap<String, Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn serialize_deserialize_round_trip() {
        let original = FlowResult {
            execution_id: Uuid::new_v4(),
            outputs: HashMap::from([
                ("a".into(), json!({ "ok": true })),
                ("b".into(), json!(null)),
            ]),
            completed_nodes: HashSet::from(["a".into(), "b".into()]),
            skipped_nodes: HashSet::from(["b".into()]),
            context: HashMap::from([("k".into(), json!("v"))]),
        };

        let json_str = serde_json::to_string(&original).expect("serialize");
        let restored: FlowResult = serde_json::from_str(&json_str).expect("deserialize");

        assert_eq!(restored.execution_id, original.execution_id);
        assert_eq!(restored.outputs, original.outputs);
        assert_eq!(restored.completed_nodes, original.completed_nodes);
        assert_eq!(restored.skipped_nodes, original.skipped_nodes);
        assert_eq!(restored.context, original.context);
    }

    #[test]
    fn context_defaults_to_empty_on_deserialize() {
        // Old serialized results without a "context" field should deserialize cleanly.
        let json_str = r#"{"execution_id":"00000000-0000-0000-0000-000000000000","outputs":{},"completed_nodes":[],"skipped_nodes":[]}"#;
        let result: FlowResult = serde_json::from_str(json_str).expect("deserialize");
        assert!(result.context.is_empty());
    }
}
