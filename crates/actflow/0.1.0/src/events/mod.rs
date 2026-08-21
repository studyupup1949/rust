mod node;
mod workflow;

pub use node::*;
pub use workflow::*;

use crate::{runtime::ProcessId, workflow::node::NodeId};

#[derive(Debug, Clone)]
pub struct Event<T> {
    inner: T,
}

#[derive(Debug, Clone)]
pub enum GraphEvent {
    Workflow(WorkflowEvent),
    Node(NodeEvent),
}

#[derive(Debug, Clone)]
pub struct Message {
    /// process id
    pub pid: ProcessId,
    /// node id
    pub nid: NodeId,
    /// event
    pub event: GraphEvent,
}

#[derive(Debug, Clone)]
pub struct Log {
    pub pid: ProcessId,
    pub nid: NodeId,
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
