//! Workflow edge definitions for connecting nodes.
//!
//! Edges define the execution flow between nodes, supporting conditional
//! branching through source handles (e.g., true/false for if_else nodes).

use serde::{Deserialize, Serialize};

use crate::{
    ActflowError, Result,
    common::Vars,
    workflow::node::{NodeId, NodeState},
};

/// Unique identifier for an edge within a workflow.
pub type EdgeId = String;

/// Fixed source handle types for common branching scenarios.
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, strum::AsRefStr, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FixedHandle {
    /// Default output handle for sequential flow.
    #[default]
    Source,
    /// True branch for conditional nodes.
    True,
    /// False branch for conditional nodes.
    False,
    /// Error/failure branch for error handling.
    FailBranch,
}

/// Source handle identifying which output port of a node an edge originates from.
///
/// Edges can originate from:
/// - Fixed handles (source, true, false, fail_branch)
/// - Dynamic handles (case IDs from if_else nodes)
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum SourceHandle {
    /// Fixed handle types: source, true, false, fail_branch.
    Fixed(FixedHandle),
    /// Dynamic handle: node ID or case ID (e.g., from if-else conditions).
    Node(NodeId),
}

impl Default for SourceHandle {
    fn default() -> Self {
        SourceHandle::Fixed(FixedHandle::default())
    }
}

/// Runtime edge representation connecting two nodes.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    /// Unique edge identifier.
    pub id: EdgeId,
    /// ID of the source node.
    pub source: NodeId,
    /// ID of the target node.
    pub target: NodeId,
    /// Which output handle this edge connects from.
    pub source_handle: SourceHandle,
    /// Current execution state of this edge.
    #[serde(default)]
    pub status: NodeState,
}

impl Edge {
    /// Creates a new edge from input variables.
    pub fn new(input: Vars) -> Result<Self> {
        let edge = serde_json::from_value(input.into()).map_err(|e| ActflowError::Edge(format!("invalid edge input: {}", e)))?;
        Ok(edge)
    }
}

/// Options for selecting which edges to follow from a node.
#[derive(Debug, Clone, Default)]
pub struct EdgeSelectOptions {
    /// The source handle to match when selecting edges.
    pub source_handle: SourceHandle,
}
