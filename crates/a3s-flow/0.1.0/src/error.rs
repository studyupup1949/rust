//! Error types for a3s-flow.

use thiserror::Error;
use uuid::Uuid;

/// All errors produced by the flow engine.
#[derive(Debug, Error)]
pub enum FlowError {
    /// The JSON DAG definition is invalid (missing fields, bad structure).
    #[error("invalid flow definition: {0}")]
    InvalidDefinition(String),

    /// The DAG contains a cycle, which is forbidden.
    #[error("flow graph contains a cycle")]
    CyclicGraph,

    /// A referenced node ID does not exist in the flow definition.
    #[error("unknown node id: {0}")]
    UnknownNode(String),

    /// A node execution failed.
    #[error("node '{node_id}' failed during execution '{execution_id}': {reason}")]
    NodeFailed {
        node_id: String,
        execution_id: Uuid,
        reason: String,
    },

    /// The execution was terminated by an external signal.
    #[error("execution was terminated")]
    Terminated,

    /// No execution found for the given ID.
    #[error("execution not found: {0}")]
    ExecutionNotFound(Uuid),

    /// No flow definition found for the given name in the configured [`FlowStore`](crate::flow_store::FlowStore).
    #[error("flow not found: {0}")]
    FlowNotFound(String),

    /// The requested state transition is not valid for the current state.
    #[error("cannot {action} a {from} execution")]
    InvalidTransition { action: String, from: String },

    /// JSON serialization / deserialization error.
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),

    /// An unexpected internal error.
    #[error("internal error: {0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, FlowError>;
