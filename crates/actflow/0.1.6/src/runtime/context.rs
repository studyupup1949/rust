//! Execution context for workflow processes.
//!
//! The context provides runtime state and utilities for node execution,
//! including environment variables, node outputs, and event emission.

use std::sync::Arc;

use crate::{
    common::{MemCache, Shutdown, Vars},
    events::{Event, Log},
    runtime::{Channel, ProcessId},
    utils,
    workflow::node::NodeId,
};

/// Execution context shared across all nodes in a workflow process.
///
/// The context maintains:
/// - Process ID for identification
/// - Environment variables accessible via `{{#env.KEY#}}`
/// - Node outputs accessible via `{{#nodeId.key#}}`
/// - Event channel for emitting logs and events
/// - Shutdown signal for graceful termination
///
/// # Thread Safety
///
/// Context is designed to be safely shared across async tasks using `Arc`.
#[derive(Clone)]
pub struct Context {
    /// Unique process identifier.
    pid: ProcessId,
    /// Environment variables cache.
    env: Arc<MemCache<String, String>>,
    /// Node outputs cache, keyed by node ID.
    outputs: Arc<MemCache<NodeId, Vars>>,
    /// Event channel for broadcasting events and logs.
    channel: Arc<Channel>,
    /// Shutdown coordinator for graceful termination.
    shutdown: Arc<Shutdown>,
}

impl Context {
    /// Creates a new execution context for a process.
    ///
    /// # Arguments
    ///
    /// * `pid` - Unique process identifier
    /// * `channel` - Event channel for broadcasting events
    pub fn new(
        pid: ProcessId,
        channel: Arc<Channel>,
    ) -> Self {
        Self {
            pid,
            env: Arc::new(MemCache::new(1024)),
            outputs: Arc::new(MemCache::new(1024)),
            channel,
            shutdown: Arc::new(Shutdown::new()),
        }
    }

    /// Returns the environment variables cache.
    pub fn env(&self) -> Arc<MemCache<String, String>> {
        self.env.clone()
    }

    /// Returns the node outputs cache.
    pub fn outputs(&self) -> Arc<MemCache<NodeId, Vars>> {
        self.outputs.clone()
    }

    /// Stores the output of a node execution.
    ///
    /// # Arguments
    ///
    /// * `nid` - Node identifier
    /// * `outputs` - Output variables from the node
    pub fn add_output(
        &self,
        nid: NodeId,
        outputs: Vars,
    ) {
        self.outputs.set(nid, outputs);
    }

    /// Returns the event channel.
    pub fn channel(&self) -> Arc<Channel> {
        self.channel.clone()
    }

    /// Emits a log message from a node.
    ///
    /// # Arguments
    ///
    /// * `nid` - Node identifier that generated the log
    /// * `content` - Log message content
    pub fn emit_log(
        &self,
        nid: NodeId,
        content: String,
    ) {
        let log = Log {
            pid: self.pid.clone(),
            nid,
            content,
            timestamp: utils::time::time_millis(),
        };
        let _ = self.channel.log_queue().send(Event::new(&log));
    }

    /// Returns the process identifier.
    pub fn pid(&self) -> ProcessId {
        self.pid.to_owned()
    }

    /// Signals that the context should shut down.
    ///
    /// This triggers the shutdown signal, causing any waiting tasks
    /// to be notified and terminate gracefully.
    pub fn done(&self) {
        self.shutdown.shutdown();
    }

    /// Returns a future that completes when shutdown is signaled.
    ///
    /// Use this with `tokio::select!` to handle graceful shutdown:
    ///
    /// ```rust,ignore
    /// tokio::select! {
    ///     _ = ctx.wait_shutdown() => {
    ///         // Handle shutdown
    ///     }
    ///     result = some_async_work() => {
    ///         // Handle result
    ///     }
    /// }
    /// ```
    pub fn wait_shutdown(&self) -> impl Future<Output = ()> + Send + 'static {
        self.shutdown.wait()
    }
}
