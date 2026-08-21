//! Flow execution engine.
//!
//! [`FlowRunner`] takes a [`DagGraph`] and a [`NodeRegistry`], executes each
//! node wave-by-wave, and returns a [`FlowResult`].
//!
//! Two execution modes are available:
//! - [`FlowRunner::run`] — fire-and-forget: run to completion with no external control
//! - [`FlowRunner::run_controlled`] — used by [`FlowEngine`] to support pause / resume / terminate

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Semaphore;

use serde_json::Value;
use tokio::sync::watch;
use tokio::task::JoinSet;
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, instrument, Instrument};
use uuid::Uuid;

use crate::error::{FlowError, Result};
use crate::event::{EventEmitter, NoopEventEmitter};
use crate::flow_store::FlowStore;
use crate::graph::DagGraph;
use crate::node::{ExecContext, Node, RetryPolicy};
use crate::registry::NodeRegistry;
use crate::result::FlowResult;

/// Signal used to control a running execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum FlowSignal {
    /// The flow should continue executing.
    Run,
    /// The flow should pause at the next wave boundary.
    Pause,
}

/// Executes a [`DagGraph`] using registered [`Node`](crate::node::Node) implementations.
///
/// For lifecycle control (pause / resume / terminate), use [`FlowEngine`](crate::engine::FlowEngine)
/// instead of constructing a `FlowRunner` directly.
///
/// # Example
///
/// ```rust,no_run
/// use a3s_flow::{DagGraph, FlowRunner, NodeRegistry};
/// use serde_json::json;
///
/// #[tokio::main]
/// async fn main() {
///     let def = json!({
///         "nodes": [
///             { "id": "start", "type": "noop" },
///             { "id": "end",   "type": "noop" }
///         ],
///         "edges": [{ "source": "start", "target": "end" }]
///     });
///     let dag = DagGraph::from_json(&def).unwrap();
///     let registry = NodeRegistry::with_defaults();
///     let runner = FlowRunner::new(dag, registry);
///     let result = runner.run(Default::default()).await.unwrap();
///     println!("{:?}", result.outputs);
/// }
/// ```
pub struct FlowRunner {
    dag: DagGraph,
    registry: Arc<NodeRegistry>,
    emitter: Arc<dyn EventEmitter>,
    flow_store: Option<Arc<dyn FlowStore>>,
    /// When set, at most this many nodes execute concurrently within a wave.
    max_concurrency: Option<usize>,
}

impl FlowRunner {
    /// Create a new runner from a validated DAG and a node registry.
    ///
    /// Uses [`NoopEventEmitter`] by default. Call
    /// [`.with_event_emitter`](Self::with_event_emitter) to register a custom
    /// listener before executing.
    pub fn new(dag: DagGraph, registry: NodeRegistry) -> Self {
        Self {
            dag,
            registry: Arc::new(registry),
            emitter: Arc::new(NoopEventEmitter),
            flow_store: None,
            max_concurrency: None,
        }
    }

    /// Create a new runner sharing an existing `Arc<NodeRegistry>`.
    ///
    /// Used by the `"iteration"` and `"sub-flow"` nodes so that sub-flow
    /// runners share the same registry without extra `Arc` wrapping.
    pub fn with_arc_registry(dag: DagGraph, registry: Arc<NodeRegistry>) -> Self {
        Self {
            dag,
            registry,
            emitter: Arc::new(NoopEventEmitter),
            flow_store: None,
            max_concurrency: None,
        }
    }

    /// Attach a custom event emitter to this runner.
    ///
    /// The emitter receives node and flow lifecycle events during execution.
    /// Returns `self` for method chaining.
    pub fn with_event_emitter(mut self, emitter: Arc<dyn EventEmitter>) -> Self {
        self.emitter = emitter;
        self
    }

    /// Attach a flow definition store to this runner.
    ///
    /// When set, the store is passed to every [`ExecContext`] so that nodes
    /// like `"sub-flow"` can load named flow definitions at execution time.
    /// Returns `self` for method chaining.
    pub fn with_flow_store(mut self, store: Arc<dyn FlowStore>) -> Self {
        self.flow_store = Some(store);
        self
    }

    /// Limit the number of nodes that may execute concurrently within a single
    /// wave.
    ///
    /// By default all ready nodes in a wave run in parallel. Setting
    /// `max_concurrency` to `n` caps this using a Tokio [`Semaphore`] so that
    /// at most `n` nodes are active at the same time. Useful when downstream
    /// services impose rate limits.
    ///
    /// Returns `self` for method chaining.
    pub fn with_max_concurrency(mut self, n: usize) -> Self {
        self.max_concurrency = Some(n);
        self
    }

    /// Execute the flow to completion with no external control signals.
    #[instrument(skip(self, variables), fields(execution_id))]
    pub async fn run(&self, variables: HashMap<String, Value>) -> Result<FlowResult> {
        let execution_id = Uuid::new_v4();
        tracing::Span::current().record("execution_id", execution_id.to_string());
        // No-op signal channel and a token that is never cancelled.
        let (_tx, rx) = watch::channel(FlowSignal::Run);
        let cancel = CancellationToken::new();
        self.run_seeded(
            execution_id,
            variables,
            rx,
            cancel,
            HashMap::new(),
            HashSet::new(),
            HashSet::new(),
        )
        .await
    }

    /// Resume a flow from a prior (partial or complete) result, skipping nodes
    /// that already have outputs in `prior`.
    ///
    /// A new execution ID is assigned to the resumed run. Nodes listed in
    /// `prior.completed_nodes` are not re-executed; their outputs from `prior`
    /// are used directly as inputs for any downstream nodes that still need to run.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// # use a3s_flow::{DagGraph, FlowRunner, NodeRegistry};
    /// # use serde_json::json;
    /// # use std::collections::HashMap;
    /// # #[tokio::main] async fn main() {
    /// let def = json!({ "nodes": [{ "id": "a", "type": "noop" }], "edges": [] });
    /// let dag = DagGraph::from_json(&def).unwrap();
    /// let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
    /// let partial = runner.run(HashMap::new()).await.unwrap();
    /// // Resume with the partial result — completed nodes are skipped.
    /// let full = runner.resume_from(&partial, HashMap::new()).await.unwrap();
    /// # }
    /// ```
    pub async fn resume_from(
        &self,
        prior: &FlowResult,
        variables: HashMap<String, Value>,
    ) -> Result<FlowResult> {
        let execution_id = Uuid::new_v4();
        let (_tx, rx) = watch::channel(FlowSignal::Run);
        let cancel = CancellationToken::new();
        self.run_seeded(
            execution_id,
            variables,
            rx,
            cancel,
            prior.outputs.clone(),
            prior.completed_nodes.clone(),
            prior.skipped_nodes.clone(),
        )
        .await
    }

    /// Execute the flow with external pause / resume / terminate control.
    ///
    /// This is the method used by [`FlowEngine`](crate::engine::FlowEngine).
    /// Prefer using `FlowEngine` rather than calling this directly.
    pub(crate) async fn run_controlled(
        &self,
        execution_id: Uuid,
        variables: HashMap<String, Value>,
        signal_rx: watch::Receiver<FlowSignal>,
        cancel: CancellationToken,
    ) -> Result<FlowResult> {
        self.run_seeded(
            execution_id,
            variables,
            signal_rx,
            cancel,
            HashMap::new(),
            HashSet::new(),
            HashSet::new(),
        )
        .await
    }

    // ── Internal implementation ────────────────────────────────────────────

    /// Emits flow lifecycle events around [`execute_waves`].
    ///
    /// `initial_*` collections seed execution for partial-resume; pass empty
    /// collections for a fresh run.
    #[allow(clippy::too_many_arguments)]
    async fn run_seeded(
        &self,
        execution_id: Uuid,
        variables: HashMap<String, Value>,
        signal_rx: watch::Receiver<FlowSignal>,
        cancel: CancellationToken,
        initial_outputs: HashMap<String, Value>,
        initial_completed: HashSet<String>,
        initial_skipped: HashSet<String>,
    ) -> Result<FlowResult> {
        info!(%execution_id, "flow execution started");
        self.emitter.on_flow_started(execution_id).await;

        let outcome = self
            .execute_waves(
                execution_id,
                variables,
                signal_rx,
                cancel,
                initial_outputs,
                initial_completed,
                initial_skipped,
            )
            .await;

        match &outcome {
            Ok(result) => {
                info!(%execution_id, "flow execution complete");
                self.emitter.on_flow_completed(execution_id, result).await;
            }
            Err(FlowError::Terminated) => {
                info!(%execution_id, "flow execution terminated");
                self.emitter.on_flow_terminated(execution_id).await;
            }
            Err(e) => {
                tracing::warn!(%execution_id, error = %e, "flow execution failed");
                self.emitter
                    .on_flow_failed(execution_id, &e.to_string())
                    .await;
            }
        }

        outcome
    }

    /// Wave-based execution engine — emits node events, no flow lifecycle events.
    #[allow(clippy::too_many_arguments)]
    async fn execute_waves(
        &self,
        execution_id: Uuid,
        variables: HashMap<String, Value>,
        mut signal_rx: watch::Receiver<FlowSignal>,
        cancel: CancellationToken,
        initial_outputs: HashMap<String, Value>,
        initial_completed: HashSet<String>,
        initial_skipped: HashSet<String>,
    ) -> Result<FlowResult> {
        // `variables` is mutable so that `"assign"` nodes can inject new values
        // into the running variable scope between waves.
        let mut variables = variables;
        let mut outputs = initial_outputs;
        let mut completed = initial_completed;
        // Nodes whose `run_if` evaluated to false — used to propagate skips.
        let mut skipped = initial_skipped;
        // Only include nodes that haven't completed yet.
        let mut remaining: Vec<String> = self
            .dag
            .nodes_in_order()
            .map(|n| n.id.clone())
            .filter(|id| !completed.contains(id))
            .collect();

        while !remaining.is_empty() {
            // ── Pause / cancel checkpoint (between waves) ──────────────────
            loop {
                if cancel.is_cancelled() {
                    return Err(FlowError::Terminated);
                }
                // Copy the signal value before matching so the borrow is
                // released before we call signal_rx.changed() below.
                let signal = *signal_rx.borrow();
                match signal {
                    FlowSignal::Run => break,
                    FlowSignal::Pause => {
                        // Block until the signal changes or we are cancelled.
                        tokio::select! {
                            _ = signal_rx.changed() => continue,
                            _ = cancel.cancelled()  => return Err(FlowError::Terminated),
                        }
                    }
                }
            }

            // ── Find nodes ready to run ────────────────────────────────────
            let (ready, not_ready): (Vec<_>, Vec<_>) = remaining.into_iter().partition(|id| {
                self.dag
                    .dependencies_of(id)
                    .iter()
                    .all(|dep| completed.contains(dep))
            });

            if ready.is_empty() {
                return Err(FlowError::Internal(
                    "execution stalled: no nodes are ready but not all nodes are done".into(),
                ));
            }

            remaining = not_ready;

            // ── Collect assign-node IDs before consuming `ready` ──────────
            // After the wave completes, these nodes' outputs are merged into
            // the live variable map so that downstream nodes see the new values.
            let assign_node_ids: Vec<String> = ready
                .iter()
                .filter(|id| {
                    self.dag
                        .nodes
                        .get(*id)
                        .map(|n| n.write_to_variables)
                        .unwrap_or(false)
                })
                .cloned()
                .collect();

            // ── Concurrency limiter for this wave ─────────────────────────
            let semaphore = self
                .max_concurrency
                .map(|n| Arc::new(Semaphore::new(n)));

            // ── Launch ready nodes concurrently ───────────────────────────
            let mut join_set: JoinSet<(String, Result<Value>)> = JoinSet::new();

            for node_id in ready {
                let node_def = self.dag.nodes[&node_id].clone();

                // Check run_if guard: if the condition fails, skip this node.
                if let Some(ref cond) = node_def.run_if {
                    if !cond.evaluate(&outputs, &skipped) {
                        debug!(%node_id, "node skipped (run_if condition false)");
                        self.emitter.on_node_skipped(execution_id, &node_id).await;
                        outputs.insert(node_id.clone(), Value::Null);
                        skipped.insert(node_id.clone());
                        completed.insert(node_id);
                        continue;
                    }
                }

                let node = self.registry.get(&node_def.node_type)?;

                let inputs: HashMap<String, Value> = self
                    .dag
                    .dependencies_of(&node_id)
                    .iter()
                    .filter_map(|dep| outputs.get(dep).map(|v| (dep.clone(), v.clone())))
                    .collect();

                let ctx = ExecContext {
                    data: node_def.data.clone(),
                    inputs,
                    variables: variables.clone(),
                    registry: Arc::clone(&self.registry),
                    flow_store: self.flow_store.clone(),
                };

                let retry = node_def.retry.clone();
                let timeout_ms = node_def.timeout_ms;
                let continue_on_error = node_def.continue_on_error;
                let emitter = Arc::clone(&self.emitter);
                let sem = semaphore.clone();

                debug!(
                    %node_id,
                    node_type = %node_def.node_type,
                    retry = ?retry.as_ref().map(|r| r.max_attempts),
                    timeout_ms,
                    continue_on_error,
                    "executing node"
                );

                // ── Per-node OTel-compatible span ──────────────────────────
                let span = tracing::info_span!(
                    "node.execute",
                    node_id = node_id.as_str(),
                    node_type = node_def.node_type.as_str(),
                    %execution_id,
                );

                join_set.spawn(
                    async move {
                        // Acquire concurrency permit inside the task so all
                        // tasks are spawned immediately but only `max_concurrency`
                        // run at the same time. The permit is released on drop.
                        let _permit = if let Some(ref s) = sem {
                            Some(Arc::clone(s).acquire_owned().await.ok())
                        } else {
                            None
                        };

                        emitter
                            .on_node_started(execution_id, &node_id, &node_def.node_type)
                            .await;

                        let result: Result<Value> = execute_with_policy(node, ctx, retry, timeout_ms)
                            .await
                            .map_err(|e| FlowError::NodeFailed {
                                node_id: node_id.clone(),
                                execution_id,
                                reason: e.to_string(),
                            });

                        // If continue_on_error is set, absorb failure and emit
                        // a completed event with an `__error__` sentinel output.
                        let result: Result<Value> = if continue_on_error {
                            result.or_else(|e| {
                                Ok(serde_json::json!({ "__error__": e.to_string() }))
                            })
                        } else {
                            result
                        };

                        match &result {
                            Ok(v) => {
                                emitter.on_node_completed(execution_id, &node_id, v).await;
                            }
                            Err(e) => {
                                emitter
                                    .on_node_failed(execution_id, &node_id, &e.to_string())
                                    .await;
                            }
                        }

                        (node_id, result)
                    }
                    .instrument(span),
                );
            }

            // ── Collect results (cancel-aware) ─────────────────────────────
            loop {
                tokio::select! {
                    // Termination signal takes priority over pending node results.
                    _ = cancel.cancelled() => {
                        // Remaining tasks are aborted when join_set is dropped.
                        return Err(FlowError::Terminated);
                    }
                    maybe = join_set.join_next() => {
                        match maybe {
                            None => break, // all nodes in this wave done
                            Some(Ok((node_id, Ok(value)))) => {
                                debug!(%node_id, "node completed");
                                outputs.insert(node_id.clone(), value);
                                completed.insert(node_id);
                            }
                            Some(Ok((_, Err(e)))) => return Err(e),
                            Some(Err(join_err)) if join_err.is_cancelled() => {
                                return Err(FlowError::Terminated);
                            }
                            Some(Err(e)) => return Err(FlowError::Internal(e.to_string())),
                        }
                    }
                }
            }

            // ── Merge assign-node outputs into the live variable map ───────
            // Only non-error outputs are merged (skip `continue_on_error` sentinels).
            for node_id in &assign_node_ids {
                if let Some(Value::Object(obj)) = outputs.get(node_id) {
                    if !obj.contains_key("__error__") {
                        for (k, v) in obj {
                            variables.insert(k.clone(), v.clone());
                        }
                    }
                }
            }
        }

        Ok(FlowResult {
            execution_id,
            outputs,
            completed_nodes: completed,
            skipped_nodes: skipped,
        })
    }
}

// ── Node execution helper ──────────────────────────────────────────────────

/// Execute a node with optional retry and timeout policies.
///
/// - Retries up to `retry.max_attempts` times (first attempt included).
/// - Each retry waits `backoff_ms * 2^(attempt-1)` ms (capped at 64× base).
/// - Each individual attempt is bounded by `timeout_ms` if set.
async fn execute_with_policy(
    node: Arc<dyn Node>,
    ctx: ExecContext,
    retry: Option<RetryPolicy>,
    timeout_ms: Option<u64>,
) -> Result<Value> {
    let max_attempts = retry.as_ref().map(|r| r.max_attempts.max(1)).unwrap_or(1);
    let backoff_ms = retry.as_ref().map(|r| r.backoff_ms).unwrap_or(0);

    let mut last_err = FlowError::Internal("no attempts made".into());

    for attempt in 0..max_attempts {
        if attempt > 0 && backoff_ms > 0 {
            // Exponential backoff: base * 2^(attempt-1), capped at base * 64.
            let multiplier = 1u64 << (attempt - 1).min(6);
            let delay = backoff_ms.saturating_mul(multiplier);
            tokio::time::sleep(Duration::from_millis(delay)).await;
        }

        let fut = node.execute(ctx.clone());

        let result = if let Some(ms) = timeout_ms {
            tokio::time::timeout(Duration::from_millis(ms), fut)
                .await
                .unwrap_or_else(|_| Err(FlowError::Internal(format!("timed out after {ms}ms"))))
        } else {
            fut.await
        };

        match result {
            Ok(v) => return Ok(v),
            Err(e) => last_err = e,
        }
    }

    Err(last_err)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::DagGraph;
    use crate::registry::NodeRegistry;
    use serde_json::json;

    #[tokio::test]
    async fn runs_linear_flow() {
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "b", "type": "noop" },
                { "id": "c", "type": "noop" }
            ],
            "edges": [
                { "source": "a", "target": "b" },
                { "source": "b", "target": "c" }
            ]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let registry = NodeRegistry::with_defaults();
        let runner = FlowRunner::new(dag, registry);
        let result = runner.run(HashMap::new()).await.unwrap();

        assert!(result.outputs.contains_key("a"));
        assert!(result.outputs.contains_key("b"));
        assert!(result.outputs.contains_key("c"));
    }

    #[tokio::test]
    async fn runs_parallel_fan_out() {
        let def = json!({
            "nodes": [
                { "id": "start", "type": "noop" },
                { "id": "b",     "type": "noop" },
                { "id": "c",     "type": "noop" },
                { "id": "end",   "type": "noop" }
            ],
            "edges": [
                { "source": "start", "target": "b" },
                { "source": "start", "target": "c" },
                { "source": "b",     "target": "end" },
                { "source": "c",     "target": "end" }
            ]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let registry = NodeRegistry::with_defaults();
        let runner = FlowRunner::new(dag, registry);
        let result = runner.run(HashMap::new()).await.unwrap();
        assert_eq!(result.outputs.len(), 4);
    }

    #[tokio::test]
    async fn variables_available_in_context() {
        let def = json!({ "nodes": [{ "id": "only", "type": "noop" }], "edges": [] });
        let dag = DagGraph::from_json(&def).unwrap();
        let registry = NodeRegistry::with_defaults();
        let runner = FlowRunner::new(dag, registry);

        let vars = HashMap::from([("env".into(), json!("production"))]);
        let result = runner.run(vars).await.unwrap();
        assert!(result.outputs.contains_key("only"));
    }

    #[tokio::test]
    async fn run_if_skips_node_when_if_else_falls_to_else() {
        // "route" if-else: data == 999 → no match → branch = "else"
        // "process" run_if checks branch == "hit" → skipped
        let def = json!({
            "nodes": [
                { "id": "data", "type": "noop" },
                {
                    "id": "route", "type": "if-else",
                    "data": { "cases": [{ "id": "hit", "conditions": [{ "from": "data", "path": "", "op": "eq", "value": 999 }] }] }
                },
                {
                    "id": "process", "type": "noop",
                    "data": { "run_if": { "from": "route", "path": "branch", "op": "eq", "value": "hit" } }
                }
            ],
            "edges": [
                { "source": "data",  "target": "route" },
                { "source": "route", "target": "process" }
            ]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
        let result = runner.run(HashMap::new()).await.unwrap();

        assert_eq!(result.outputs["process"], json!(null));
    }

    #[tokio::test]
    async fn run_if_executes_node_when_if_else_matches() {
        // noop outputs {} — if-else matches {} == {} → branch = "hit"
        let def = json!({
            "nodes": [
                { "id": "src", "type": "noop" },
                {
                    "id": "gate", "type": "if-else",
                    "data": { "cases": [{ "id": "hit", "conditions": [{ "from": "src", "path": "", "op": "eq", "value": {} }] }] }
                },
                {
                    "id": "sink", "type": "noop",
                    "data": { "run_if": { "from": "gate", "path": "branch", "op": "eq", "value": "hit" } }
                }
            ],
            "edges": [
                { "source": "src",  "target": "gate" },
                { "source": "gate", "target": "sink" }
            ]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
        let result = runner.run(HashMap::new()).await.unwrap();

        assert!(result.outputs["sink"].is_object());
        assert_ne!(result.outputs["sink"], json!(null));
    }

    #[tokio::test]
    async fn skip_propagates_through_chain() {
        // A → B (run_if fails on missing field) → C (run_if on B which is in skipped set)
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                {
                    "id": "b", "type": "noop",
                    "data": { "run_if": { "from": "a", "path": "nonexistent_field", "op": "eq", "value": true } }
                },
                {
                    "id": "c", "type": "noop",
                    "data": { "run_if": { "from": "b", "path": "x", "op": "eq", "value": 1 } }
                }
            ],
            "edges": [
                { "source": "a", "target": "b" },
                { "source": "b", "target": "c" }
            ]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
        let result = runner.run(HashMap::new()).await.unwrap();

        assert_eq!(result.outputs["b"], json!(null));
        assert_eq!(result.outputs["c"], json!(null));
    }

    #[tokio::test]
    async fn if_else_with_variable_aggregator_fan_in() {
        // route → path_ok (run if "ok") / path_err (run if "else") → merge
        let def = json!({
            "nodes": [
                { "id": "src", "type": "noop" },
                {
                    "id": "route", "type": "if-else",
                    "data": { "cases": [{ "id": "ok", "conditions": [{ "from": "src", "path": "", "op": "eq", "value": {} }] }] }
                },
                {
                    "id": "path_ok", "type": "noop",
                    "data": { "run_if": { "from": "route", "path": "branch", "op": "eq", "value": "ok" } }
                },
                {
                    "id": "path_err", "type": "noop",
                    "data": { "run_if": { "from": "route", "path": "branch", "op": "eq", "value": "else" } }
                },
                {
                    "id": "merge", "type": "variable-aggregator",
                    "data": { "inputs": ["path_ok", "path_err"] }
                }
            ],
            "edges": [
                { "source": "src",      "target": "route" },
                { "source": "route",    "target": "path_ok" },
                { "source": "route",    "target": "path_err" },
                { "source": "path_ok",  "target": "merge" },
                { "source": "path_err", "target": "merge" }
            ]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
        let result = runner.run(HashMap::new()).await.unwrap();

        // path_ok ran (src == {}), path_err was skipped → merge returns path_ok's output
        assert!(!result.outputs["merge"]["output"].is_null());
        assert_eq!(result.outputs["path_err"], json!(null));
    }

    // ── completed_nodes / skipped_nodes tracking ───────────────────────────

    #[tokio::test]
    async fn completed_nodes_tracks_all_executed_nodes() {
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "b", "type": "noop" }
            ],
            "edges": [{ "source": "a", "target": "b" }]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
        let result = runner.run(HashMap::new()).await.unwrap();

        assert!(result.completed_nodes.contains("a"));
        assert!(result.completed_nodes.contains("b"));
        assert!(result.skipped_nodes.is_empty());
    }

    #[tokio::test]
    async fn skipped_nodes_tracks_run_if_skipped_nodes() {
        // "a" → "b" with run_if that always fails → "b" is skipped
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
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
        let result = runner.run(HashMap::new()).await.unwrap();

        assert!(result.completed_nodes.contains("a"));
        assert!(result.completed_nodes.contains("b"));
        assert!(result.skipped_nodes.contains("b"));
        assert!(!result.skipped_nodes.contains("a"));
    }

    // ── retry policy ───────────────────────────────────────────────────────

    #[tokio::test]
    async fn retry_succeeds_after_transient_failures() {
        use crate::node::{ExecContext, Node};
        use async_trait::async_trait;
        use std::sync::atomic::{AtomicU32, Ordering};

        // Fails twice, succeeds on the third attempt.
        struct FlakyNode {
            call_count: Arc<AtomicU32>,
        }

        #[async_trait]
        impl Node for FlakyNode {
            fn node_type(&self) -> &str {
                "flaky"
            }

            async fn execute(&self, _ctx: ExecContext) -> Result<Value> {
                let n = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;
                if n < 3 {
                    Err(FlowError::Internal(format!("transient failure #{n}")))
                } else {
                    Ok(json!({ "ok": true }))
                }
            }
        }

        let call_count = Arc::new(AtomicU32::new(0));
        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(FlakyNode {
            call_count: Arc::clone(&call_count),
        }));

        let def = json!({
            "nodes": [{
                "id": "step",
                "type": "flaky",
                "data": { "retry": { "max_attempts": 3, "backoff_ms": 0 } }
            }],
            "edges": []
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, registry);
        let result = runner.run(HashMap::new()).await.unwrap();

        assert_eq!(result.outputs["step"]["ok"], json!(true));
        assert_eq!(call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn retry_exhausted_returns_last_error() {
        use crate::node::{ExecContext, Node};
        use async_trait::async_trait;

        // Always fails.
        struct AlwaysFailNode;

        #[async_trait]
        impl Node for AlwaysFailNode {
            fn node_type(&self) -> &str {
                "always-fail"
            }

            async fn execute(&self, _ctx: ExecContext) -> Result<Value> {
                Err(FlowError::Internal("permanent failure".into()))
            }
        }

        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(AlwaysFailNode));

        let def = json!({
            "nodes": [{
                "id": "step",
                "type": "always-fail",
                "data": { "retry": { "max_attempts": 2, "backoff_ms": 0 } }
            }],
            "edges": []
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, registry);
        let err = runner.run(HashMap::new()).await.unwrap_err();

        assert!(matches!(err, FlowError::NodeFailed { .. }));
        let msg = err.to_string();
        assert!(msg.contains("permanent failure"));
    }

    // ── timeout ────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn timeout_kills_slow_node() {
        use crate::node::{ExecContext, Node};
        use async_trait::async_trait;

        struct SlowNode;

        #[async_trait]
        impl Node for SlowNode {
            fn node_type(&self) -> &str {
                "slow-timeout"
            }

            async fn execute(&self, _ctx: ExecContext) -> Result<Value> {
                tokio::time::sleep(Duration::from_millis(500)).await;
                Ok(json!({}))
            }
        }

        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(SlowNode));

        // timeout_ms (50ms) is well below node sleep (500ms).
        let def = json!({
            "nodes": [{
                "id": "step",
                "type": "slow-timeout",
                "data": { "timeout_ms": 50 }
            }],
            "edges": []
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, registry);
        let err = runner.run(HashMap::new()).await.unwrap_err();

        assert!(matches!(err, FlowError::NodeFailed { .. }));
        assert!(err.to_string().contains("timed out"));
    }

    #[tokio::test]
    async fn timeout_does_not_affect_fast_node() {
        // noop is instant — a 200ms timeout should never trigger.
        let def = json!({
            "nodes": [{
                "id": "step",
                "type": "noop",
                "data": { "timeout_ms": 200 }
            }],
            "edges": []
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
        let result = runner.run(HashMap::new()).await.unwrap();
        assert!(result.outputs.contains_key("step"));
    }

    // ── partial execution resume ────────────────────────────────────────────

    #[tokio::test]
    async fn resume_from_skips_already_completed_nodes() {
        use crate::node::{ExecContext, Node};
        use async_trait::async_trait;
        use std::sync::atomic::{AtomicU32, Ordering};

        // Counts how many times it is called.
        struct CountingNode {
            call_count: Arc<AtomicU32>,
        }

        #[async_trait]
        impl Node for CountingNode {
            fn node_type(&self) -> &str {
                "counting"
            }

            async fn execute(&self, _ctx: ExecContext) -> Result<Value> {
                self.call_count.fetch_add(1, Ordering::SeqCst);
                Ok(json!({ "counted": true }))
            }
        }

        let count_a = Arc::new(AtomicU32::new(0));
        let count_b = Arc::new(AtomicU32::new(0));
        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(CountingNode {
            call_count: Arc::clone(&count_a),
        }));

        // We can't register two distinct "counting" nodes, so use noop for b.
        let def = json!({
            "nodes": [
                { "id": "a", "type": "counting" },
                { "id": "b", "type": "noop" }
            ],
            "edges": [{ "source": "a", "target": "b" }]
        });

        let dag = DagGraph::from_json(&def).unwrap();
        let _ = count_b; // unused — b is noop
        let runner = FlowRunner::new(dag, registry);

        // Full first run — counting node executes once.
        let first = runner.run(HashMap::new()).await.unwrap();
        assert_eq!(count_a.load(Ordering::SeqCst), 1);

        // Resume: "a" is already completed — should NOT re-execute.
        let resumed = runner.resume_from(&first, HashMap::new()).await.unwrap();
        assert_eq!(count_a.load(Ordering::SeqCst), 1); // still 1
        assert!(resumed.outputs.contains_key("a"));
        assert!(resumed.outputs.contains_key("b"));
    }

    #[tokio::test]
    async fn resume_from_runs_only_pending_nodes() {
        // Simulate a partial result where only "a" has completed.
        // "b" has not run yet.  resume_from should run "b" only.
        use crate::node::{ExecContext, Node};
        use async_trait::async_trait;
        use std::sync::atomic::{AtomicU32, Ordering};

        struct CountNode(Arc<AtomicU32>);

        #[async_trait]
        impl Node for CountNode {
            fn node_type(&self) -> &str {
                "count-b"
            }
            async fn execute(&self, _ctx: ExecContext) -> Result<Value> {
                self.0.fetch_add(1, Ordering::SeqCst);
                Ok(json!({ "ran": true }))
            }
        }

        let count_b = Arc::new(AtomicU32::new(0));
        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(CountNode(Arc::clone(&count_b))));

        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "b", "type": "count-b" }
            ],
            "edges": [{ "source": "a", "target": "b" }]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, registry);

        // Build a partial result where only "a" is done.
        let partial = FlowResult {
            execution_id: uuid::Uuid::new_v4(),
            outputs: HashMap::from([("a".into(), json!({}))]),
            completed_nodes: HashSet::from(["a".into()]),
            skipped_nodes: HashSet::new(),
        };

        let result = runner.resume_from(&partial, HashMap::new()).await.unwrap();
        assert_eq!(count_b.load(Ordering::SeqCst), 1);
        assert!(result.outputs["b"]["ran"].as_bool().unwrap());

        // Resuming a fully-completed result should not re-run any node.
        let full = runner.run(HashMap::new()).await.unwrap();
        count_b.store(0, Ordering::SeqCst);
        let _ = runner.resume_from(&full, HashMap::new()).await.unwrap();
        assert_eq!(count_b.load(Ordering::SeqCst), 0);

        let _ = partial; // suppress unused warning
    }

    // ── continue_on_error ──────────────────────────────────────────────────

    #[tokio::test]
    async fn continue_on_error_keeps_flow_running_after_node_failure() {
        use crate::node::{ExecContext, Node};
        use async_trait::async_trait;

        struct FailNode;

        #[async_trait]
        impl Node for FailNode {
            fn node_type(&self) -> &str {
                "always-fail-coe"
            }
            async fn execute(&self, _: ExecContext) -> Result<Value> {
                Err(FlowError::Internal("boom".into()))
            }
        }

        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(FailNode));

        let def = json!({
            "nodes": [
                {
                    "id": "fail",
                    "type": "always-fail-coe",
                    "data": { "continue_on_error": true }
                },
                { "id": "after", "type": "noop" }
            ],
            "edges": [{ "source": "fail", "target": "after" }]
        });

        let dag = DagGraph::from_json(&def).unwrap();
        let result = FlowRunner::new(dag, registry)
            .run(HashMap::new())
            .await
            .unwrap();

        // "fail" should have an __error__ key in its output.
        assert!(result.outputs["fail"]["__error__"].is_string());
        // "after" should still have run.
        assert!(result.completed_nodes.contains("after"));
    }

    #[tokio::test]
    async fn continue_on_error_false_halts_flow_on_failure() {
        use crate::node::{ExecContext, Node};
        use async_trait::async_trait;

        struct FailNode2;

        #[async_trait]
        impl Node for FailNode2 {
            fn node_type(&self) -> &str {
                "always-fail-halt"
            }
            async fn execute(&self, _: ExecContext) -> Result<Value> {
                Err(FlowError::Internal("halt".into()))
            }
        }

        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(FailNode2));

        let def = json!({
            "nodes": [
                { "id": "fail", "type": "always-fail-halt" },
                { "id": "after", "type": "noop" }
            ],
            "edges": [{ "source": "fail", "target": "after" }]
        });

        let dag = DagGraph::from_json(&def).unwrap();
        let err = FlowRunner::new(dag, registry)
            .run(HashMap::new())
            .await
            .unwrap_err();

        assert!(matches!(err, FlowError::NodeFailed { .. }));
    }

    // ── max_concurrency ────────────────────────────────────────────────────

    #[tokio::test]
    async fn max_concurrency_limits_parallel_execution() {
        use crate::node::{ExecContext, Node};
        use async_trait::async_trait;
        use std::sync::atomic::{AtomicU32, Ordering};

        // Tracks the peak number of concurrently-running nodes.
        let active = Arc::new(AtomicU32::new(0));
        let peak = Arc::new(AtomicU32::new(0));

        struct PeakNode {
            active: Arc<AtomicU32>,
            peak: Arc<AtomicU32>,
        }

        #[async_trait]
        impl Node for PeakNode {
            fn node_type(&self) -> &str {
                "peak-tracker"
            }
            async fn execute(&self, _: ExecContext) -> Result<Value> {
                let current = self.active.fetch_add(1, Ordering::SeqCst) + 1;
                // Update peak.
                let mut prev = self.peak.load(Ordering::SeqCst);
                while current > prev {
                    match self.peak.compare_exchange_weak(
                        prev,
                        current,
                        Ordering::SeqCst,
                        Ordering::SeqCst,
                    ) {
                        Ok(_) => break,
                        Err(actual) => prev = actual,
                    }
                }
                tokio::time::sleep(Duration::from_millis(20)).await;
                self.active.fetch_sub(1, Ordering::SeqCst);
                Ok(json!({}))
            }
        }

        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(PeakNode {
            active: Arc::clone(&active),
            peak: Arc::clone(&peak),
        }));

        // 5 independent nodes, max_concurrency = 2.
        let def = json!({
            "nodes": [
                { "id": "n1", "type": "peak-tracker" },
                { "id": "n2", "type": "peak-tracker" },
                { "id": "n3", "type": "peak-tracker" },
                { "id": "n4", "type": "peak-tracker" },
                { "id": "n5", "type": "peak-tracker" }
            ],
            "edges": []
        });

        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, registry).with_max_concurrency(2);
        let result = runner.run(HashMap::new()).await.unwrap();

        assert_eq!(result.completed_nodes.len(), 5);
        assert!(
            peak.load(Ordering::SeqCst) <= 2,
            "peak concurrency {} exceeded max of 2",
            peak.load(Ordering::SeqCst)
        );
    }

    #[tokio::test]
    async fn max_concurrency_unlimited_by_default() {
        // With no max_concurrency, all 5 independent nodes should be able to
        // run concurrently (peak may be ≤ 5, just verify flow completes).
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "b", "type": "noop" },
                { "id": "c", "type": "noop" }
            ],
            "edges": []
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let result = FlowRunner::new(dag, NodeRegistry::with_defaults())
            .run(HashMap::new())
            .await
            .unwrap();
        assert_eq!(result.completed_nodes.len(), 3);
    }

    // ── start / end nodes ──────────────────────────────────────────────────

    #[tokio::test]
    async fn start_node_resolves_variables_and_end_node_gathers_output() {
        let def = json!({
            "nodes": [
                {
                    "id": "start",
                    "type": "start",
                    "data": {
                        "inputs": [
                            { "name": "greeting", "type": "string" },
                            { "name": "repeat",   "type": "number", "default": 1 }
                        ]
                    }
                },
                {
                    "id": "end",
                    "type": "end",
                    "data": {
                        "outputs": {
                            "greeting": "/start/greeting",
                            "repeat":   "/start/repeat"
                        }
                    }
                }
            ],
            "edges": [{ "source": "start", "target": "end" }]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let mut vars = HashMap::new();
        vars.insert("greeting".to_string(), json!("hello"));
        let result = FlowRunner::new(dag, NodeRegistry::with_defaults())
            .run(vars)
            .await
            .unwrap();

        // start node resolves greeting and applies default for repeat.
        assert_eq!(result.outputs["start"]["greeting"], json!("hello"));
        assert_eq!(result.outputs["start"]["repeat"], json!(1));

        // end node gathers via JSON pointer.
        assert_eq!(result.outputs["end"]["greeting"], json!("hello"));
        assert_eq!(result.outputs["end"]["repeat"], json!(1));
    }

    // ── assign node — variable scope mutation ──────────────────────────────

    #[tokio::test]
    async fn assign_node_makes_value_visible_to_downstream_nodes() {
        // "init" assigns greeting; "read" is a code node that reads it from variables.
        let def = json!({
            "nodes": [
                {
                    "id": "init",
                    "type": "assign",
                    "data": { "assigns": { "greeting": "hello from assign" } }
                },
                {
                    "id": "read",
                    "type": "code",
                    "data": { "language": "rhai", "code": "variables.greeting" }
                }
            ],
            "edges": [{ "source": "init", "target": "read" }]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
        let result = runner.run(HashMap::new()).await.unwrap();

        assert_eq!(result.outputs["read"]["output"], json!("hello from assign"));
    }

    #[tokio::test]
    async fn assign_node_overwrites_existing_variable() {
        let def = json!({
            "nodes": [
                {
                    "id": "overwrite",
                    "type": "assign",
                    "data": { "assigns": { "x": "new_value" } }
                },
                {
                    "id": "read",
                    "type": "code",
                    "data": { "language": "rhai", "code": "variables.x" }
                }
            ],
            "edges": [{ "source": "overwrite", "target": "read" }]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
        let mut vars = HashMap::new();
        vars.insert("x".to_string(), json!("old_value"));
        let result = runner.run(vars).await.unwrap();

        assert_eq!(result.outputs["read"]["output"], json!("new_value"));
    }

    #[tokio::test]
    async fn assign_node_does_not_affect_parallel_siblings() {
        // "assign_a" and "noop_b" run in the same wave (no edges between them).
        // "read" runs after both — sees the assigned value.
        let def = json!({
            "nodes": [
                {
                    "id": "assign_a",
                    "type": "assign",
                    "data": { "assigns": { "flag": "set" } }
                },
                { "id": "noop_b", "type": "noop" },
                {
                    "id": "read",
                    "type": "code",
                    "data": { "language": "rhai", "code": "variables.flag" }
                }
            ],
            "edges": [
                { "source": "assign_a", "target": "read" },
                { "source": "noop_b",   "target": "read" }
            ]
        });
        let dag = DagGraph::from_json(&def).unwrap();
        let runner = FlowRunner::new(dag, NodeRegistry::with_defaults());
        let result = runner.run(HashMap::new()).await.unwrap();

        // "read" runs in wave 2; the assign happened in wave 1, so it's visible.
        assert_eq!(result.outputs["read"]["output"], json!("set"));
    }
}
