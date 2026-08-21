use serde::{Deserialize, Serialize};

use crate::{
    ActflowError, Result,
    common::Vars,
    workflow::node::{NodeId, NodeState},
};

/// edge id
pub type EdgeId = String;

/// Fixed source handle types
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq, Eq, strum::AsRefStr, strum::EnumString)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum FixedHandle {
    #[default]
    Source,
    True,
    False,
    FailBranch,
}

/// Source handle can be either a fixed type or a node ID
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum SourceHandle {
    /// Fixed handle types: source, true, false, fail_branch
    Fixed(FixedHandle),
    /// Node handle: node ID (e.g., case_id from if-else)
    Node(NodeId),
}

impl Default for SourceHandle {
    fn default() -> Self {
        SourceHandle::Fixed(FixedHandle::default())
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Edge {
    /// edge id
    pub id: EdgeId,
    /// source node id
    pub source: NodeId,
    /// target node id
    pub target: NodeId,
    /// source handle for conditional branching
    pub source_handle: SourceHandle,
    /// edge execution state
    #[serde(default)]
    pub status: NodeState,
}

impl Edge {
    pub fn new(input: Vars) -> Result<Self> {
        let edge = serde_json::from_value(input.into()).map_err(|e| ActflowError::Edge(format!("invalid edge input: {}", e)))?;
        Ok(edge)
    }
}

#[derive(Debug, Clone, Default)]
pub struct EdgeSelectOptions {
    pub source_handle: SourceHandle,
}
