//! Agent trait definition.

use crate::types::{AgentOutput, Context, Inputs};
use std::future::Future;
use tokio::sync::mpsc;

/// A sender for streaming log messages during agent execution.
#[derive(Clone)]
pub struct LogSender {
    tx: mpsc::Sender<String>,
}

impl LogSender {
    pub(crate) fn new(tx: mpsc::Sender<String>) -> Self {
        Self { tx }
    }

    /// Send a log message.
    pub async fn send(&self, msg: impl Into<String>) {
        let _ = self.tx.send(msg.into()).await;
    }

    /// Send a log message (non-async, best-effort).
    pub fn try_send(&self, msg: impl Into<String>) {
        let _ = self.tx.try_send(msg.into());
    }
}

/// Trait for implementing an Actflow agent.
///
/// Implement this trait to create your own agent that can be executed
/// by the Actflow workflow engine.
///
/// # Example
///
/// ```rust,ignore
/// use actflow_agent_sdk::{Agent, AgentOutput, Context, Inputs, LogSender};
///
/// struct MyAgent;
///
/// impl Agent for MyAgent {
///     async fn run(
///         &self,
///         nid: String,
///         ctx: Context,
///         inputs: Inputs,
///         log: LogSender,
///     ) -> AgentOutput {
///         log.send("Processing...").await;
///         AgentOutput::success(serde_json::json!({"result": "done"}))
///     }
/// }
/// ```
pub trait Agent: Send + Sync + 'static {
    /// Execute the agent logic.
    ///
    /// # Arguments
    ///
    /// * `nid` - Node ID in the workflow
    /// * `ctx` - Execution context containing process ID, environment variables, and workflow variables
    /// * `inputs` - Input data for this agent
    /// * `log` - Log sender for streaming log messages
    ///
    /// # Returns
    ///
    /// Returns an `AgentOutput` containing the execution status and output data.
    fn run(
        &self,
        nid: String,
        ctx: Context,
        inputs: Inputs,
        log: LogSender,
    ) -> impl Future<Output = AgentOutput> + Send;

    /// Called when the agent should shut down.
    ///
    /// Override this method to perform cleanup operations.
    /// Default implementation does nothing.
    fn shutdown(&self) -> impl Future<Output = ()> + Send {
        async {}
    }
}

