//! Workflow-level events.

use std::collections::HashMap;

use crate::{common::Vars, workflow::node::NodeId};

/// Events emitted during workflow execution.
#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    /// Workflow has started execution.
    Start(WorkflowStartEvent),
    /// Workflow completed successfully.
    Succeeded,
    /// Workflow failed with an error.
    Failed(WorkflowFailedEvent),
    /// Workflow was aborted.
    Aborted(WorkflowAbortedEvent),
    /// Workflow was paused.
    Paused(WorkflowPausedEvent),
}

impl WorkflowEvent {
    /// Returns a string representation of the event type.
    pub fn str(&self) -> &str {
        match self {
            WorkflowEvent::Start(_) => "Running",
            WorkflowEvent::Succeeded => "Succeeded",
            WorkflowEvent::Failed(_) => "Failed",
            WorkflowEvent::Aborted(_) => "Aborted",
            WorkflowEvent::Paused(_) => "Paused",
        }
    }
}

/// Event emitted when a workflow starts.
#[derive(Debug, Clone)]
pub struct WorkflowStartEvent {
    /// All node IDs in the workflow for batch initialization.
    pub node_ids: Vec<NodeId>,
}

/// Event emitted when a workflow fails.
#[derive(Debug, Clone)]
pub struct WorkflowFailedEvent {
    /// Error message describing the failure.
    pub error: String,
}

/// Event emitted when a workflow is aborted.
#[derive(Debug, Clone)]
pub struct WorkflowAbortedEvent {
    /// Reason for the abort.
    pub reason: String,
    /// Outputs collected before abort.
    pub outputs: HashMap<NodeId, Vars>,
}

/// Event emitted when a workflow is paused.
#[derive(Debug, Clone)]
pub struct WorkflowPausedEvent {
    /// Reason for the pause.
    pub reason: String,
    /// Outputs collected before pause.
    pub outputs: Vars,
}
