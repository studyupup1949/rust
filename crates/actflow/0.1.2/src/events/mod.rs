//! Event types for workflow execution.
//!
//! Events are emitted during workflow execution to notify subscribers
//! about state changes, completions, errors, and logs.

mod node;
mod workflow;

pub use node::*;
pub use workflow::*;

use crate::{runtime::ProcessId, workflow::node::NodeId};

/// Generic event wrapper.
#[derive(Debug, Clone)]
pub struct Event<T> {
    inner: T,
}

/// Top-level event type for workflow graph events.
#[derive(Debug, Clone)]
pub enum GraphEvent {
    /// Workflow-level events (start, complete, failed, etc.).
    Workflow(WorkflowEvent),
    /// Node-level events (running, succeeded, error, etc.).
    Node(NodeEvent),
}

/// Event message containing process and node context.
#[derive(Debug, Clone)]
pub struct Message {
    /// Process ID that generated this event.
    pub pid: ProcessId,
    /// Node ID that generated this event (empty for workflow events).
    pub nid: NodeId,
    /// The actual event data.
    pub event: GraphEvent,
}

/// Log entry emitted during node execution.
#[derive(Debug, Clone)]
pub struct Log {
    /// Process ID that generated this log.
    pub pid: ProcessId,
    /// Node ID that generated this log.
    pub nid: NodeId,
    /// Log message content.
    pub content: String,
    /// Timestamp in milliseconds of the log entry.
    pub timestamp: i64,
}

impl<T> std::ops::Deref for Event<T>
where
    T: std::fmt::Debug + Clone,
{
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<T> Event<T>
where
    T: std::fmt::Debug + Clone,
{
    pub fn new(inner: &T) -> Self {
        Self {
            inner: inner.clone(),
        }
    }

    pub fn inner(&self) -> &T {
        &self.inner
    }
}

impl GraphEvent {
    pub fn is_complete(&self) -> bool {
        matches!(self, GraphEvent::Workflow(WorkflowEvent::Succeeded))
    }

    pub fn is_error(&self) -> bool {
        matches!(self, GraphEvent::Workflow(WorkflowEvent::Failed(_)))
    }
}
