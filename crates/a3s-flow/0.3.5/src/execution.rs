//! Execution handle and state types.
//!
//! [`ExecutionState`] is the public snapshot of a workflow execution.
//! [`ExecutionHandle`] is an internal struct held by [`FlowEngine`] to control
//! a running execution via a signal channel and a cancellation token.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use serde_json::Value;
use tokio::sync::{broadcast, watch, RwLock as AsyncRwLock};
use tokio_util::sync::CancellationToken;

use crate::event::FlowEvent;
use crate::result::FlowResult;
use crate::runner::FlowSignal;

/// Snapshot of a workflow execution's current lifecycle state.
#[derive(Debug, Clone)]
pub enum ExecutionState {
    /// Actively executing nodes.
    Running,
    /// Paused at a wave boundary; waiting for a resume signal.
    Paused,
    /// Finished successfully. Contains the per-node outputs.
    Completed(FlowResult),
    /// Aborted due to a node error. Contains the error message.
    Failed(String),
    /// Stopped by an external [`FlowEngine::terminate`](crate::engine::FlowEngine::terminate) call.
    Terminated,
}

impl ExecutionState {
    /// Short label used in error messages.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Paused => "paused",
            Self::Completed(_) => "completed",
            Self::Failed(_) => "failed",
            Self::Terminated => "terminated",
        }
    }

    /// Returns `true` if the execution has reached a terminal state and will
    /// not change again.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            Self::Completed(_) | Self::Failed(_) | Self::Terminated
        )
    }
}

/// Internal control handle for a single execution managed by [`FlowEngine`].
pub(crate) struct ExecutionHandle {
    /// Shared, mutable execution state. Updated by the runner task (terminal
    /// states) and by the engine (pause / resume).
    pub(crate) state: Arc<AsyncRwLock<ExecutionState>>,
    /// Sends pause / run signals to the runner task.
    pub(crate) signal_tx: watch::Sender<FlowSignal>,
    /// Cancellation token — call `.cancel()` to terminate the execution.
    pub(crate) cancel: CancellationToken,
    /// Shared mutable context, accessible from both nodes (via ExecContext)
    /// and the engine (via CRUD methods). Uses std::sync::RwLock so nodes
    /// can access it in non-async contexts.
    pub(crate) context: Arc<RwLock<HashMap<String, Value>>>,
    /// Broadcast sender used for live event subscriptions on this execution.
    pub(crate) event_tx: Arc<RwLock<Option<broadcast::Sender<FlowEvent>>>>,
}
