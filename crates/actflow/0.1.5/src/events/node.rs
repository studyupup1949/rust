//! Node execution events.

use std::fmt;

/// Events emitted during node execution.
///
/// Each variant includes a timestamp (i64 milliseconds) where applicable.
#[derive(Debug, Clone)]
pub enum NodeEvent {
    /// Node has started executing (timestamp).
    Running(i64),
    /// Node was stopped (timestamp).
    Stopped(i64),
    /// Node was paused (timestamp).
    Paused(i64),
    /// Node was skipped (e.g., due to conditional branching).
    Skipped,
    /// Node completed successfully (timestamp).
    Succeeded(i64),
    /// Node encountered an error.
    Error(ErrorReason),
    /// Node is retrying after a failure.
    Retry,
}

impl NodeEvent {
    /// Returns a string representation of the event type.
    pub fn str(&self) -> &str {
        match self {
            NodeEvent::Running(_) => "Running",
            NodeEvent::Stopped(_) => "Stopped",
            NodeEvent::Paused(_) => "Paused",
            NodeEvent::Skipped => "Skipped",
            NodeEvent::Succeeded(_) => "Succeeded",
            NodeEvent::Error(_) => "Error",
            NodeEvent::Retry => "Retry",
        }
    }
}

/// Reason for a node execution error.
#[derive(Debug, Clone)]
pub enum ErrorReason {
    /// Node execution timed out.
    Timeout,
    /// Node action returned a failure.
    Failed(String),
    /// Node encountered an unexpected exception.
    Exception(String),
}

impl fmt::Display for ErrorReason {
    fn fmt(
        &self,
        f: &mut fmt::Formatter<'_>,
    ) -> fmt::Result {
        match self {
            ErrorReason::Timeout => write!(f, "Timeout"),
            ErrorReason::Failed(msg) => write!(f, "Failed: {}", msg),
            ErrorReason::Exception(msg) => write!(f, "Exception: {}", msg),
        }
    }
}
