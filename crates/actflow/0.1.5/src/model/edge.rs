//! Edge model for workflow definition.

use serde::{Deserialize, Serialize};

/// Represents a directed edge connecting two nodes in a workflow.
///
/// Edges define the execution flow between nodes. The `source_handle` field
/// is used for conditional branching (e.g., in if_else nodes).
///
/// # Example
///
/// ```rust
/// use actflow::EdgeModel;
///
/// let edge = EdgeModel {
///     id: "e1".to_string(),
///     source: "node1".to_string(),
///     target: "node2".to_string(),
///     source_handle: "source".to_string(), // or "true"/"false" for if_else
/// };
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EdgeModel {
    /// Unique identifier for this edge.
    pub id: String,
    /// ID of the source node where this edge originates.
    pub source: String,
    /// ID of the target node where this edge leads to.
    pub target: String,
    /// Handle name on the source node (e.g., "source", "true", "false").
    /// Used for conditional branching in if_else nodes.
    pub source_handle: String,
}
