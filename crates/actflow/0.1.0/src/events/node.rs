use std::fmt;

#[derive(Debug, Clone)]
pub enum NodeEvent {
    Running(i64),
    Stopped(i64),
    Paused(i64),
    Skipped,
    Succeeded(i64),
    Error(ErrorReason),
    Retry,
}

impl NodeEvent {
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

#[derive(Debug, Clone)]
pub enum ErrorReason {
    Timeout,
    Failed(String),
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
