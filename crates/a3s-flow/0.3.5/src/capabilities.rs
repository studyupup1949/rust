//! Structured capability discovery for `a3s-flow`.
//!
//! This module provides a transport-friendly view of the engine's built-in and
//! custom node catalog so higher layers can expose progressive discovery APIs
//! without hard-coding node metadata.

use serde::{Deserialize, Serialize};

use crate::registry::NodeDescriptor;

/// Stable capabilities document for a `FlowEngine` instance.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FlowCapabilities {
    /// Schema/version marker for downstream clients.
    pub version: String,
    /// Whether clients should progressively expand into finer-grained
    /// capabilities instead of assuming a fixed surface.
    pub progressive_disclosure: bool,
    /// Short human-readable summary of the current engine capability set.
    pub summary: String,
    /// Structured node catalog, sorted by `node_type`.
    pub nodes: Vec<NodeDescriptor>,
}

impl FlowCapabilities {
    /// Build a capabilities document from a node catalog.
    pub fn from_nodes(nodes: Vec<NodeDescriptor>) -> Self {
        Self {
            version: "2026-03-22".to_string(),
            progressive_disclosure: true,
            summary: "A3S Flow exposes a discoverable catalog of workflow node capabilities."
                .to_string(),
            nodes,
        }
    }
}
