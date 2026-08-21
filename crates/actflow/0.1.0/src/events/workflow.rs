use std::collections::HashMap;

use crate::{common::Vars, workflow::node::NodeId};

#[derive(Debug, Clone)]
pub enum WorkflowEvent {
    Start(WorkflowStartEvent),
    Succeeded,
    Failed(WorkflowFailedEvent),
    Aborted(WorkflowAbortedEvent),
    Paused(WorkflowPausedEvent),
}

impl WorkflowEvent {
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

/// Event emitted when a workflow starts
#[derive(Debug, Clone)]
pub struct WorkflowStartEvent {
    /// All node IDs in the workflow for batch initialization
    pub node_ids: Vec<NodeId>,
}

#[derive(Debug, Clone)]
pub struct WorkflowFailedEvent {
    pub error: String,
}

#[derive(Debug, Clone)]
pub struct WorkflowAbortedEvent {
    pub reason: String,
    pub outputs: HashMap<NodeId, Vars>,
}

#[derive(Debug, Clone)]
pub struct WorkflowPausedEvent {
    pub reason: String,
    pub outputs: Vars,
}
