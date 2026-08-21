//! [`EventEmitter`] — node and flow lifecycle event extension point.
//!
//! Implement [`EventEmitter`] to react to workflow execution events — e.g. to
//! stream progress to a UI, collect metrics, or integrate with `a3s-event`.
//!
//! Register a custom emitter via
//! [`FlowEngine::with_event_emitter`](crate::engine::FlowEngine::with_event_emitter) or
//! [`FlowRunner::with_event_emitter`](crate::runner::FlowRunner::with_event_emitter).
//! The built-in [`NoopEventEmitter`] is used when no custom emitter is set.
//!
//! For pull-based event consumption, use
//! [`FlowEngine::start_streaming`](crate::engine::FlowEngine::start_streaming)
//! which returns a [`tokio::sync::broadcast::Receiver<FlowEvent>`].

use std::sync::Arc;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::sync::broadcast;
use uuid::Uuid;

use crate::result::FlowResult;

/// A snapshot of a single lifecycle event emitted during flow execution.
///
/// Produced by [`FlowEngine::start_streaming`](crate::engine::FlowEngine::start_streaming)
/// via a [`tokio::sync::broadcast`] channel. All variants are `Clone` so they
/// can be forwarded to multiple subscribers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FlowEvent {
    /// A flow execution started.
    FlowStarted { execution_id: Uuid },
    /// A flow execution completed successfully.
    FlowCompleted {
        execution_id: Uuid,
        result: FlowResult,
    },
    /// A flow execution failed (node error or internal error).
    FlowFailed { execution_id: Uuid, reason: String },
    /// A flow execution was terminated externally.
    FlowTerminated { execution_id: Uuid },
    /// A node is about to execute.
    NodeStarted {
        execution_id: Uuid,
        node_id: String,
        node_type: String,
    },
    /// A node completed successfully.
    NodeCompleted {
        execution_id: Uuid,
        node_id: String,
        output: Value,
    },
    /// A node was skipped because its `run_if` guard evaluated to false.
    NodeSkipped { execution_id: Uuid, node_id: String },
    /// A node failed (all retry attempts exhausted).
    NodeFailed {
        execution_id: Uuid,
        node_id: String,
        reason: String,
    },
    /// A node completed with full output details (including node_type).
    NodeCompletedFull {
        execution_id: Uuid,
        node_id: String,
        node_type: String,
        inputs: Value,
        process_data: Option<Value>,
        outputs: Value,
        elapsed_time_ms: u64,
    },
    /// An iteration started within an iteration node.
    IterationStarted {
        execution_id: Uuid,
        node_id: String,
        iteration_id: String,
        iteration_index: u32,
    },
    /// An iteration produced the next item.
    IterationNext {
        execution_id: Uuid,
        node_id: String,
        iteration_id: String,
        iteration_index: u32,
    },
    /// An iteration completed.
    IterationCompleted {
        execution_id: Uuid,
        node_id: String,
        iteration_id: String,
    },
    /// A loop started within a loop node.
    LoopStarted {
        execution_id: Uuid,
        node_id: String,
        loop_id: String,
        loop_index: u32,
    },
    /// A loop completed.
    LoopCompleted {
        execution_id: Uuid,
        node_id: String,
        loop_id: String,
    },
    /// A parallel branch started.
    ParallelBranchStarted {
        execution_id: Uuid,
        node_id: String,
        parallel_id: String,
        branch_id: String,
    },
    /// A parallel branch completed.
    ParallelBranchCompleted {
        execution_id: Uuid,
        node_id: String,
        parallel_id: String,
        branch_id: String,
    },
    /// A node retry attempt started.
    NodeRetry {
        execution_id: Uuid,
        node_id: String,
        retry_index: u32,
        max_retries: u32,
    },
}

/// An [`EventEmitter`] that forwards all events into a broadcast channel.
///
/// Created internally by [`FlowEngine::start_streaming`](crate::engine::FlowEngine::start_streaming).
pub(crate) struct ChannelEmitter {
    tx: broadcast::Sender<FlowEvent>,
}

impl ChannelEmitter {
    pub(crate) fn new(tx: broadcast::Sender<FlowEvent>) -> Self {
        Self { tx }
    }
}

#[async_trait]
impl EventEmitter for ChannelEmitter {
    async fn on_flow_started(&self, execution_id: Uuid) {
        let _ = self.tx.send(FlowEvent::FlowStarted { execution_id });
    }

    async fn on_flow_completed(&self, execution_id: Uuid, result: &FlowResult) {
        let _ = self.tx.send(FlowEvent::FlowCompleted {
            execution_id,
            result: result.clone(),
        });
    }

    async fn on_flow_failed(&self, execution_id: Uuid, reason: &str) {
        let _ = self.tx.send(FlowEvent::FlowFailed {
            execution_id,
            reason: reason.to_string(),
        });
    }

    async fn on_flow_terminated(&self, execution_id: Uuid) {
        let _ = self.tx.send(FlowEvent::FlowTerminated { execution_id });
    }

    async fn on_node_started(&self, execution_id: Uuid, node_id: &str, node_type: &str) {
        let _ = self.tx.send(FlowEvent::NodeStarted {
            execution_id,
            node_id: node_id.to_string(),
            node_type: node_type.to_string(),
        });
    }

    async fn on_node_completed(&self, execution_id: Uuid, node_id: &str, output: &Value) {
        let _ = self.tx.send(FlowEvent::NodeCompleted {
            execution_id,
            node_id: node_id.to_string(),
            output: output.clone(),
        });
    }

    async fn on_node_skipped(&self, execution_id: Uuid, node_id: &str) {
        let _ = self.tx.send(FlowEvent::NodeSkipped {
            execution_id,
            node_id: node_id.to_string(),
        });
    }

    async fn on_node_failed(&self, execution_id: Uuid, node_id: &str, reason: &str) {
        let _ = self.tx.send(FlowEvent::NodeFailed {
            execution_id,
            node_id: node_id.to_string(),
            reason: reason.to_string(),
        });
    }

    async fn on_node_completed_full(
        &self,
        execution_id: Uuid,
        node_id: &str,
        node_type: &str,
        inputs: &Value,
        process_data: Option<&Value>,
        outputs: &Value,
        elapsed_time_ms: u64,
    ) {
        let _ = self.tx.send(FlowEvent::NodeCompletedFull {
            execution_id,
            node_id: node_id.to_string(),
            node_type: node_type.to_string(),
            inputs: inputs.clone(),
            process_data: process_data.cloned(),
            outputs: outputs.clone(),
            elapsed_time_ms,
        });
    }

    async fn on_iteration_started(
        &self,
        execution_id: Uuid,
        node_id: &str,
        iteration_id: &str,
        iteration_index: u32,
    ) {
        let _ = self.tx.send(FlowEvent::IterationStarted {
            execution_id,
            node_id: node_id.to_string(),
            iteration_id: iteration_id.to_string(),
            iteration_index,
        });
    }

    async fn on_iteration_next(
        &self,
        execution_id: Uuid,
        node_id: &str,
        iteration_id: &str,
        iteration_index: u32,
    ) {
        let _ = self.tx.send(FlowEvent::IterationNext {
            execution_id,
            node_id: node_id.to_string(),
            iteration_id: iteration_id.to_string(),
            iteration_index,
        });
    }

    async fn on_iteration_completed(&self, execution_id: Uuid, node_id: &str, iteration_id: &str) {
        let _ = self.tx.send(FlowEvent::IterationCompleted {
            execution_id,
            node_id: node_id.to_string(),
            iteration_id: iteration_id.to_string(),
        });
    }

    async fn on_loop_started(
        &self,
        execution_id: Uuid,
        node_id: &str,
        loop_id: &str,
        loop_index: u32,
    ) {
        let _ = self.tx.send(FlowEvent::LoopStarted {
            execution_id,
            node_id: node_id.to_string(),
            loop_id: loop_id.to_string(),
            loop_index,
        });
    }

    async fn on_loop_completed(&self, execution_id: Uuid, node_id: &str, loop_id: &str) {
        let _ = self.tx.send(FlowEvent::LoopCompleted {
            execution_id,
            node_id: node_id.to_string(),
            loop_id: loop_id.to_string(),
        });
    }

    async fn on_parallel_branch_started(
        &self,
        execution_id: Uuid,
        node_id: &str,
        parallel_id: &str,
        branch_id: &str,
    ) {
        let _ = self.tx.send(FlowEvent::ParallelBranchStarted {
            execution_id,
            node_id: node_id.to_string(),
            parallel_id: parallel_id.to_string(),
            branch_id: branch_id.to_string(),
        });
    }

    async fn on_parallel_branch_completed(
        &self,
        execution_id: Uuid,
        node_id: &str,
        parallel_id: &str,
        branch_id: &str,
        _output: &Value,
    ) {
        let _ = self.tx.send(FlowEvent::ParallelBranchCompleted {
            execution_id,
            node_id: node_id.to_string(),
            parallel_id: parallel_id.to_string(),
            branch_id: branch_id.to_string(),
        });
    }

    async fn on_node_retry(
        &self,
        execution_id: Uuid,
        node_id: &str,
        retry_index: u32,
        max_retries: u32,
    ) {
        let _ = self.tx.send(FlowEvent::NodeRetry {
            execution_id,
            node_id: node_id.to_string(),
            retry_index,
            max_retries,
        });
    }
}

/// An [`EventEmitter`] that fans events out to two downstream emitters.
///
/// Used internally by [`FlowEngine::start_streaming`](crate::engine::FlowEngine::start_streaming)
/// to compose a [`ChannelEmitter`] with the engine's existing emitter.
pub(crate) struct MulticastEmitter {
    pub(crate) a: Arc<dyn EventEmitter>,
    pub(crate) b: Arc<dyn EventEmitter>,
}

#[async_trait]
impl EventEmitter for MulticastEmitter {
    async fn on_flow_started(&self, execution_id: Uuid) {
        self.a.on_flow_started(execution_id).await;
        self.b.on_flow_started(execution_id).await;
    }

    async fn on_flow_completed(&self, execution_id: Uuid, result: &FlowResult) {
        self.a.on_flow_completed(execution_id, result).await;
        self.b.on_flow_completed(execution_id, result).await;
    }

    async fn on_flow_failed(&self, execution_id: Uuid, reason: &str) {
        self.a.on_flow_failed(execution_id, reason).await;
        self.b.on_flow_failed(execution_id, reason).await;
    }

    async fn on_flow_terminated(&self, execution_id: Uuid) {
        self.a.on_flow_terminated(execution_id).await;
        self.b.on_flow_terminated(execution_id).await;
    }

    async fn on_node_started(&self, execution_id: Uuid, node_id: &str, node_type: &str) {
        self.a
            .on_node_started(execution_id, node_id, node_type)
            .await;
        self.b
            .on_node_started(execution_id, node_id, node_type)
            .await;
    }

    async fn on_node_completed(&self, execution_id: Uuid, node_id: &str, output: &Value) {
        self.a
            .on_node_completed(execution_id, node_id, output)
            .await;
        self.b
            .on_node_completed(execution_id, node_id, output)
            .await;
    }

    async fn on_node_skipped(&self, execution_id: Uuid, node_id: &str) {
        self.a.on_node_skipped(execution_id, node_id).await;
        self.b.on_node_skipped(execution_id, node_id).await;
    }

    async fn on_node_failed(&self, execution_id: Uuid, node_id: &str, reason: &str) {
        self.a.on_node_failed(execution_id, node_id, reason).await;
        self.b.on_node_failed(execution_id, node_id, reason).await;
    }

    async fn on_node_completed_full(
        &self,
        execution_id: Uuid,
        node_id: &str,
        node_type: &str,
        inputs: &Value,
        process_data: Option<&Value>,
        outputs: &Value,
        elapsed_time_ms: u64,
    ) {
        self.a
            .on_node_completed_full(
                execution_id,
                node_id,
                node_type,
                inputs,
                process_data,
                outputs,
                elapsed_time_ms,
            )
            .await;
        self.b
            .on_node_completed_full(
                execution_id,
                node_id,
                node_type,
                inputs,
                process_data,
                outputs,
                elapsed_time_ms,
            )
            .await;
    }

    async fn on_iteration_started(
        &self,
        execution_id: Uuid,
        node_id: &str,
        iteration_id: &str,
        iteration_index: u32,
    ) {
        self.a
            .on_iteration_started(execution_id, node_id, iteration_id, iteration_index)
            .await;
        self.b
            .on_iteration_started(execution_id, node_id, iteration_id, iteration_index)
            .await;
    }

    async fn on_iteration_next(
        &self,
        execution_id: Uuid,
        node_id: &str,
        iteration_id: &str,
        iteration_index: u32,
    ) {
        self.a
            .on_iteration_next(execution_id, node_id, iteration_id, iteration_index)
            .await;
        self.b
            .on_iteration_next(execution_id, node_id, iteration_id, iteration_index)
            .await;
    }

    async fn on_iteration_completed(&self, execution_id: Uuid, node_id: &str, iteration_id: &str) {
        self.a
            .on_iteration_completed(execution_id, node_id, iteration_id)
            .await;
        self.b
            .on_iteration_completed(execution_id, node_id, iteration_id)
            .await;
    }

    async fn on_loop_started(
        &self,
        execution_id: Uuid,
        node_id: &str,
        loop_id: &str,
        loop_index: u32,
    ) {
        self.a
            .on_loop_started(execution_id, node_id, loop_id, loop_index)
            .await;
        self.b
            .on_loop_started(execution_id, node_id, loop_id, loop_index)
            .await;
    }

    async fn on_loop_completed(&self, execution_id: Uuid, node_id: &str, loop_id: &str) {
        self.a
            .on_loop_completed(execution_id, node_id, loop_id)
            .await;
        self.b
            .on_loop_completed(execution_id, node_id, loop_id)
            .await;
    }

    async fn on_parallel_branch_started(
        &self,
        execution_id: Uuid,
        node_id: &str,
        parallel_id: &str,
        branch_id: &str,
    ) {
        self.a
            .on_parallel_branch_started(execution_id, node_id, parallel_id, branch_id)
            .await;
        self.b
            .on_parallel_branch_started(execution_id, node_id, parallel_id, branch_id)
            .await;
    }

    async fn on_parallel_branch_completed(
        &self,
        execution_id: Uuid,
        node_id: &str,
        parallel_id: &str,
        branch_id: &str,
        output: &Value,
    ) {
        self.a
            .on_parallel_branch_completed(execution_id, node_id, parallel_id, branch_id, output)
            .await;
        self.b
            .on_parallel_branch_completed(execution_id, node_id, parallel_id, branch_id, output)
            .await;
    }

    async fn on_node_retry(
        &self,
        execution_id: Uuid,
        node_id: &str,
        retry_index: u32,
        max_retries: u32,
    ) {
        self.a
            .on_node_retry(execution_id, node_id, retry_index, max_retries)
            .await;
        self.b
            .on_node_retry(execution_id, node_id, retry_index, max_retries)
            .await;
    }
}
///
/// All methods default to no-ops in [`NoopEventEmitter`]. Implement only the
/// events you care about by delegating to your own struct.
///
/// # Example
///
/// ```rust
/// use a3s_flow::{EventEmitter, FlowResult, NoopEventEmitter};
/// use async_trait::async_trait;
/// use serde_json::Value;
/// use uuid::Uuid;
///
/// struct PrintEmitter;
///
/// #[async_trait]
/// impl EventEmitter for PrintEmitter {
///     async fn on_flow_started(&self, _: Uuid) {}
///     async fn on_flow_completed(&self, _: Uuid, _: &FlowResult) {}
///     async fn on_flow_failed(&self, _: Uuid, _: &str) {}
///     async fn on_flow_terminated(&self, _: Uuid) {}
///     async fn on_node_started(&self, _: Uuid, _: &str, _: &str) {}
///     async fn on_node_completed(&self, _exec: Uuid, node_id: &str, _out: &Value) {
///         println!("node {node_id} completed");
///     }
///     async fn on_node_skipped(&self, _: Uuid, _: &str) {}
///     async fn on_node_failed(&self, _: Uuid, _: &str, _: &str) {}
///     async fn on_node_completed_full(&self, _: Uuid, _: &str, _: &str, _: &Value, _: Option<&Value>, _: &Value, _: u64) {}
///     async fn on_iteration_started(&self, _: Uuid, _: &str, _: &str, _: u32) {}
///     async fn on_iteration_next(&self, _: Uuid, _: &str, _: &str, _: u32) {}
///     async fn on_iteration_completed(&self, _: Uuid, _: &str, _: &str) {}
///     async fn on_loop_started(&self, _: Uuid, _: &str, _: &str, _: u32) {}
///     async fn on_loop_completed(&self, _: Uuid, _: &str, _: &str) {}
///     async fn on_parallel_branch_started(&self, _: Uuid, _: &str, _: &str, _: &str) {}
///     async fn on_parallel_branch_completed(&self, _: Uuid, _: &str, _: &str, _: &str, _: &Value) {}
///     async fn on_node_retry(&self, _: Uuid, _: &str, _: u32, _: u32) {}
/// }
/// ```
#[async_trait]
pub trait EventEmitter: Send + Sync {
    /// A flow execution has started.
    async fn on_flow_started(&self, execution_id: Uuid);

    /// A flow execution completed successfully.
    async fn on_flow_completed(&self, execution_id: Uuid, result: &FlowResult);

    /// A flow execution failed (node error or internal error).
    async fn on_flow_failed(&self, execution_id: Uuid, reason: &str);

    /// A flow execution was terminated via [`FlowEngine::terminate`](crate::engine::FlowEngine::terminate).
    async fn on_flow_terminated(&self, execution_id: Uuid);

    /// A node is about to execute (before the first attempt).
    async fn on_node_started(&self, execution_id: Uuid, node_id: &str, node_type: &str);

    /// A node completed successfully.
    async fn on_node_completed(&self, execution_id: Uuid, node_id: &str, output: &Value);

    /// A node was skipped because its `run_if` guard evaluated to false.
    async fn on_node_skipped(&self, execution_id: Uuid, node_id: &str);

    /// A node failed (all retry attempts exhausted).
    async fn on_node_failed(&self, execution_id: Uuid, node_id: &str, reason: &str);

    /// A node completed with full details including elapsed time.
    async fn on_node_completed_full(
        &self,
        execution_id: Uuid,
        node_id: &str,
        node_type: &str,
        inputs: &Value,
        process_data: Option<&Value>,
        outputs: &Value,
        elapsed_time_ms: u64,
    );

    /// An iteration started.
    async fn on_iteration_started(
        &self,
        execution_id: Uuid,
        node_id: &str,
        iteration_id: &str,
        iteration_index: u32,
    );

    /// An iteration produced the next item.
    async fn on_iteration_next(
        &self,
        execution_id: Uuid,
        node_id: &str,
        iteration_id: &str,
        iteration_index: u32,
    );

    /// An iteration completed.
    async fn on_iteration_completed(&self, execution_id: Uuid, node_id: &str, iteration_id: &str);

    /// A loop started.
    async fn on_loop_started(
        &self,
        execution_id: Uuid,
        node_id: &str,
        loop_id: &str,
        loop_index: u32,
    );

    /// A loop completed.
    async fn on_loop_completed(&self, execution_id: Uuid, node_id: &str, loop_id: &str);

    /// A parallel branch started.
    async fn on_parallel_branch_started(
        &self,
        execution_id: Uuid,
        node_id: &str,
        parallel_id: &str,
        branch_id: &str,
    );

    /// A parallel branch completed.
    async fn on_parallel_branch_completed(
        &self,
        execution_id: Uuid,
        node_id: &str,
        parallel_id: &str,
        branch_id: &str,
        output: &Value,
    );

    /// A node retry attempt started.
    async fn on_node_retry(
        &self,
        execution_id: Uuid,
        node_id: &str,
        retry_index: u32,
        max_retries: u32,
    );
}

/// A no-op [`EventEmitter`] — the default when no custom emitter is registered.
pub struct NoopEventEmitter;

#[async_trait]
impl EventEmitter for NoopEventEmitter {
    async fn on_flow_started(&self, _: Uuid) {}
    async fn on_flow_completed(&self, _: Uuid, _: &FlowResult) {}
    async fn on_flow_failed(&self, _: Uuid, _: &str) {}
    async fn on_flow_terminated(&self, _: Uuid) {}
    async fn on_node_started(&self, _: Uuid, _: &str, _: &str) {}
    async fn on_node_completed(&self, _: Uuid, _: &str, _: &Value) {}
    async fn on_node_skipped(&self, _: Uuid, _: &str) {}
    async fn on_node_failed(&self, _: Uuid, _: &str, _: &str) {}
    async fn on_node_completed_full(
        &self,
        _: Uuid,
        _: &str,
        _: &str,
        _: &Value,
        _: Option<&Value>,
        _: &Value,
        _: u64,
    ) {
    }
    async fn on_iteration_started(&self, _: Uuid, _: &str, _: &str, _: u32) {}
    async fn on_iteration_next(&self, _: Uuid, _: &str, _: &str, _: u32) {}
    async fn on_iteration_completed(&self, _: Uuid, _: &str, _: &str) {}
    async fn on_loop_started(&self, _: Uuid, _: &str, _: &str, _: u32) {}
    async fn on_loop_completed(&self, _: Uuid, _: &str, _: &str) {}
    async fn on_parallel_branch_started(&self, _: Uuid, _: &str, _: &str, _: &str) {}
    async fn on_parallel_branch_completed(&self, _: Uuid, _: &str, _: &str, _: &str, _: &Value) {}
    async fn on_node_retry(&self, _: Uuid, _: &str, _: u32, _: u32) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    };

    // A test emitter that counts each event type.
    struct CountEmitter {
        flow_started: Arc<AtomicU32>,
        flow_completed: Arc<AtomicU32>,
        flow_failed: Arc<AtomicU32>,
        flow_terminated: Arc<AtomicU32>,
        node_started: Arc<AtomicU32>,
        node_completed: Arc<AtomicU32>,
        node_skipped: Arc<AtomicU32>,
        node_failed: Arc<AtomicU32>,
        node_completed_full: Arc<AtomicU32>,
        iteration_started: Arc<AtomicU32>,
        iteration_next: Arc<AtomicU32>,
        iteration_completed: Arc<AtomicU32>,
        loop_started: Arc<AtomicU32>,
        loop_completed: Arc<AtomicU32>,
        parallel_branch_started: Arc<AtomicU32>,
        parallel_branch_completed: Arc<AtomicU32>,
        node_retry: Arc<AtomicU32>,
    }

    impl CountEmitter {
        fn new() -> (Arc<Self>, Counts) {
            let s = Arc::new(AtomicU32::new(0));
            let c = Arc::new(AtomicU32::new(0));
            let fa = Arc::new(AtomicU32::new(0));
            let t = Arc::new(AtomicU32::new(0));
            let ns = Arc::new(AtomicU32::new(0));
            let nc = Arc::new(AtomicU32::new(0));
            let nsk = Arc::new(AtomicU32::new(0));
            let nf = Arc::new(AtomicU32::new(0));
            let ncf = Arc::new(AtomicU32::new(0));
            let is = Arc::new(AtomicU32::new(0));
            let inx = Arc::new(AtomicU32::new(0));
            let ic = Arc::new(AtomicU32::new(0));
            let ls = Arc::new(AtomicU32::new(0));
            let lc = Arc::new(AtomicU32::new(0));
            let pbs = Arc::new(AtomicU32::new(0));
            let pbc = Arc::new(AtomicU32::new(0));
            let nr = Arc::new(AtomicU32::new(0));
            let emitter = Arc::new(CountEmitter {
                flow_started: Arc::clone(&s),
                flow_completed: Arc::clone(&c),
                flow_failed: Arc::clone(&fa),
                flow_terminated: Arc::clone(&t),
                node_started: Arc::clone(&ns),
                node_completed: Arc::clone(&nc),
                node_skipped: Arc::clone(&nsk),
                node_failed: Arc::clone(&nf),
                node_completed_full: Arc::clone(&ncf),
                iteration_started: Arc::clone(&is),
                iteration_next: Arc::clone(&inx),
                iteration_completed: Arc::clone(&ic),
                loop_started: Arc::clone(&ls),
                loop_completed: Arc::clone(&lc),
                parallel_branch_started: Arc::clone(&pbs),
                parallel_branch_completed: Arc::clone(&pbc),
                node_retry: Arc::clone(&nr),
            });
            let counts = Counts {
                s,
                c,
                fa,
                t,
                ns,
                nc,
                nsk,
                nf,
                ncf,
                is,
                inx,
                ic,
                ls,
                lc,
                pbs,
                pbc,
                nr,
            };
            (emitter, counts)
        }
    }

    #[allow(dead_code)]
    struct Counts {
        s: Arc<AtomicU32>,
        c: Arc<AtomicU32>,
        fa: Arc<AtomicU32>,
        t: Arc<AtomicU32>,
        ns: Arc<AtomicU32>,
        nc: Arc<AtomicU32>,
        nsk: Arc<AtomicU32>,
        nf: Arc<AtomicU32>,
        ncf: Arc<AtomicU32>,
        is: Arc<AtomicU32>,
        inx: Arc<AtomicU32>,
        ic: Arc<AtomicU32>,
        ls: Arc<AtomicU32>,
        lc: Arc<AtomicU32>,
        pbs: Arc<AtomicU32>,
        pbc: Arc<AtomicU32>,
        nr: Arc<AtomicU32>,
    }

    #[async_trait]
    impl EventEmitter for CountEmitter {
        async fn on_flow_started(&self, _: Uuid) {
            self.flow_started.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_flow_completed(&self, _: Uuid, _: &FlowResult) {
            self.flow_completed.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_flow_failed(&self, _: Uuid, _: &str) {
            self.flow_failed.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_flow_terminated(&self, _: Uuid) {
            self.flow_terminated.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_node_started(&self, _: Uuid, _: &str, _: &str) {
            self.node_started.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_node_completed(&self, _: Uuid, _: &str, _: &Value) {
            self.node_completed.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_node_skipped(&self, _: Uuid, _: &str) {
            self.node_skipped.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_node_failed(&self, _: Uuid, _: &str, _: &str) {
            self.node_failed.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_node_completed_full(
            &self,
            _: Uuid,
            _: &str,
            _: &str,
            _: &Value,
            _: Option<&Value>,
            _: &Value,
            _: u64,
        ) {
            self.node_completed_full.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_iteration_started(&self, _: Uuid, _: &str, _: &str, _: u32) {
            self.iteration_started.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_iteration_next(&self, _: Uuid, _: &str, _: &str, _: u32) {
            self.iteration_next.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_iteration_completed(&self, _: Uuid, _: &str, _: &str) {
            self.iteration_completed.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_loop_started(&self, _: Uuid, _: &str, _: &str, _: u32) {
            self.loop_started.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_loop_completed(&self, _: Uuid, _: &str, _: &str) {
            self.loop_completed.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_parallel_branch_started(&self, _: Uuid, _: &str, _: &str, _: &str) {
            self.parallel_branch_started.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_parallel_branch_completed(&self, _: Uuid, _: &str, _: &str, _: &str, _: &Value) {
            self.parallel_branch_completed.fetch_add(1, Ordering::SeqCst);
        }
        async fn on_node_retry(&self, _: Uuid, _: &str, _: u32, _: u32) {
            self.node_retry.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn noop_emitter_compiles_and_runs() {
        // Verify the no-op emitter can be called without panicking.
        let e = NoopEventEmitter;
        let id = Uuid::new_v4();
        let result = FlowResult {
            execution_id: id,
            outputs: Default::default(),
            completed_nodes: Default::default(),
            skipped_nodes: Default::default(),
            context: Default::default(),
        };
        e.on_flow_started(id).await;
        e.on_flow_completed(id, &result).await;
        e.on_flow_failed(id, "err").await;
        e.on_flow_terminated(id).await;
        e.on_node_started(id, "n", "noop").await;
        e.on_node_completed(id, "n", &serde_json::json!({})).await;
        e.on_node_skipped(id, "n").await;
        e.on_node_failed(id, "n", "err").await;
    }

    #[tokio::test]
    async fn emitter_receives_flow_and_node_events() {
        use crate::graph::DagGraph;
        use crate::registry::NodeRegistry;
        use crate::runner::FlowRunner;
        use serde_json::json;
        use std::collections::HashMap;

        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "b", "type": "noop" }
            ],
            "edges": [{ "source": "a", "target": "b" }]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let (emitter, counts) = CountEmitter::new();
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults())
            .with_event_emitter(emitter as Arc<dyn EventEmitter>);

        runner.run(HashMap::new()).await.unwrap();

        assert_eq!(counts.s.load(Ordering::SeqCst), 1, "flow_started");
        assert_eq!(counts.c.load(Ordering::SeqCst), 1, "flow_completed");
        assert_eq!(counts.fa.load(Ordering::SeqCst), 0, "flow_failed");
        assert_eq!(counts.ns.load(Ordering::SeqCst), 2, "node_started (a + b)");
        assert_eq!(
            counts.nc.load(Ordering::SeqCst),
            2,
            "node_completed (a + b)"
        );
        assert_eq!(counts.nsk.load(Ordering::SeqCst), 0, "no skipped nodes");
    }

    #[tokio::test]
    async fn emitter_receives_node_skipped_event() {
        use crate::graph::DagGraph;
        use crate::registry::NodeRegistry;
        use crate::runner::FlowRunner;
        use serde_json::json;
        use std::collections::HashMap;

        // "b" is always skipped via a run_if that never matches.
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                {
                    "id": "b", "type": "noop",
                    "data": { "run_if": { "from": "a", "path": "nonexistent", "op": "eq", "value": true } }
                }
            ],
            "edges": [{ "source": "a", "target": "b" }]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let (emitter, counts) = CountEmitter::new();
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults())
            .with_event_emitter(emitter as Arc<dyn EventEmitter>);

        runner.run(HashMap::new()).await.unwrap();

        assert_eq!(counts.nsk.load(Ordering::SeqCst), 1, "one skipped node");
        assert_eq!(counts.nc.load(Ordering::SeqCst), 1, "only 'a' completed");
    }

    #[tokio::test]
    async fn emitter_receives_node_failed_and_flow_failed() {
        use crate::error::FlowError;
        use crate::graph::DagGraph;
        use crate::node::{ExecContext, Node};
        use crate::registry::NodeRegistry;
        use crate::runner::FlowRunner;
        use serde_json::json;
        use std::collections::HashMap;

        struct FailNode;
        #[async_trait]
        impl Node for FailNode {
            fn node_type(&self) -> &str {
                "fail-always"
            }
            async fn execute(&self, _: ExecContext) -> crate::error::Result<Value> {
                Err(FlowError::Internal("boom".into()))
            }
        }

        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(FailNode));

        let def = json!({ "nodes": [{ "id": "x", "type": "fail-always" }], "edges": [] });
        let dag = DagGraph::from_json(&def).unwrap();
        let (emitter, counts) = CountEmitter::new();
        let runner =
            FlowRunner::new(dag, registry).with_event_emitter(emitter as Arc<dyn EventEmitter>);

        let _ = runner.run(HashMap::new()).await;

        assert_eq!(counts.nf.load(Ordering::SeqCst), 1, "node_failed");
        assert_eq!(counts.fa.load(Ordering::SeqCst), 1, "flow_failed");
        assert_eq!(counts.c.load(Ordering::SeqCst), 0, "not completed");
    }
}
