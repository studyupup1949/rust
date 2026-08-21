//! [`FlowEngine`] — the central API for managing workflow executions.
//!
//! `FlowEngine` wraps a [`NodeRegistry`] and an in-memory execution map.
//! It is the recommended entry point for applications that need lifecycle
//! control over running workflows.
//!
//! # Lifecycle
//!
//! ```text
//!                    ┌──────────┐
//!             start  │ Running  │ ←──────────── resume
//!           ────────►│          │
//!                    └────┬─────┘
//!               pause │   │ node error / all nodes done / terminate
//!                     ▼   ▼
//!                 ┌────────────┐   ┌───────────┐   ┌────────────┐
//!                 │   Paused   │   │ Completed │   │   Failed   │
//!                 └────────────┘   └───────────┘   └────────────┘
//!                                                  ┌────────────┐
//!                                                  │ Terminated │
//!                                                  └────────────┘
//! ```
//!
//! Pause takes effect at the **next wave boundary** — the current wave of
//! concurrently executing nodes runs to completion first. Terminate interrupts
//! the engine between or within waves and is reflected as soon as the runner
//! task observes the cancellation signal.

use std::collections::HashMap;
use std::sync::{Arc, RwLock as SyncRwLock};

use serde_json::Value;
use tokio::sync::{watch, RwLock};
use tokio_util::sync::CancellationToken;
use tracing::warn;
use uuid::Uuid;

use tokio::sync::broadcast;

use crate::capabilities::FlowCapabilities;
use crate::error::{FlowError, Result};
use crate::event::{ChannelEmitter, EventEmitter, FlowEvent, MulticastEmitter, NoopEventEmitter};
use crate::execution::{ExecutionHandle, ExecutionState};
use crate::flow_store::FlowStore;
use crate::graph::DagGraph;
use crate::node::Node;
use crate::registry::{NodeDescriptor, NodeRegistry};
use crate::runner::{FlowRunner, FlowSignal};
use crate::store::ExecutionStore;
use crate::validation::ValidationIssue;

/// Central entry point for managing workflow executions.
///
/// # Example
///
/// ```rust,no_run
/// use a3s_flow::{FlowEngine, NodeRegistry};
/// use serde_json::json;
/// use std::collections::HashMap;
///
/// #[tokio::main]
/// async fn main() -> a3s_flow::Result<()> {
///     let engine = FlowEngine::new(NodeRegistry::with_defaults());
///
///     // Query available node types.
///     println!("node types: {:?}", engine.node_types());
///
///     // Start a workflow and get its execution ID.
///     let definition = json!({
///         "nodes": [
///             { "id": "a", "type": "noop" },
///             { "id": "b", "type": "noop" }
///         ],
///         "edges": [{ "source": "a", "target": "b" }]
///     });
///     let id = engine.start(&definition, HashMap::new()).await?;
///
///     // Inspect state, pause, resume, or terminate.
///     println!("state: {:?}", engine.state(id).await?);
///     Ok(())
/// }
/// ```
pub struct FlowEngine {
    registry: Arc<SyncRwLock<NodeRegistry>>,
    executions: Arc<RwLock<HashMap<Uuid, ExecutionHandle>>>,
    /// Optional store — when set, completed results are automatically persisted.
    execution_store: Option<Arc<dyn ExecutionStore>>,
    /// Optional store — enables `start_named` to look up definitions by name.
    flow_store: Option<Arc<dyn FlowStore>>,
    /// Emitter passed to each runner; receives all node and flow lifecycle events.
    emitter: Arc<dyn EventEmitter>,
    /// When set, passed to every runner to cap per-wave node concurrency.
    max_concurrency: Option<usize>,
}

impl FlowEngine {
    /// Create a new engine with the given node registry.
    ///
    /// Uses [`NoopEventEmitter`] and no execution store by default. Use the
    /// builder methods [`with_execution_store`](Self::with_execution_store) and
    /// [`with_event_emitter`](Self::with_event_emitter) to customise behaviour.
    pub fn new(registry: NodeRegistry) -> Self {
        Self {
            registry: Arc::new(SyncRwLock::new(registry)),
            executions: Arc::new(RwLock::new(HashMap::new())),
            execution_store: None,
            flow_store: None,
            emitter: Arc::new(NoopEventEmitter),
            max_concurrency: None,
        }
    }

    /// Attach an execution store.
    ///
    /// When set, every successfully completed execution result is saved to the
    /// store automatically. Returns `self` for method chaining.
    pub fn with_execution_store(mut self, store: Arc<dyn ExecutionStore>) -> Self {
        self.execution_store = Some(store);
        self
    }

    /// Attach a flow definition store.
    ///
    /// Required for [`start_named`](Self::start_named). Allows any backend
    /// (in-memory, SQLite, remote API, …) by implementing [`FlowStore`].
    /// Returns `self` for method chaining.
    pub fn with_flow_store(mut self, store: Arc<dyn FlowStore>) -> Self {
        self.flow_store = Some(store);
        self
    }

    /// Attach a custom event emitter.
    ///
    /// The emitter is passed to every runner created by this engine and
    /// receives all node and flow lifecycle events. Returns `self` for chaining.
    pub fn with_event_emitter(mut self, emitter: Arc<dyn EventEmitter>) -> Self {
        self.emitter = emitter;
        self
    }

    /// Limit the number of nodes that may execute concurrently within a single
    /// wave across all executions started by this engine.
    ///
    /// Delegates to [`FlowRunner::with_max_concurrency`]. Returns `self` for chaining.
    pub fn with_max_concurrency(mut self, n: usize) -> Self {
        self.max_concurrency = Some(n);
        self
    }

    // ── Node type discovery ────────────────────────────────────────────────

    /// Return all registered node type strings, sorted alphabetically.
    ///
    /// Includes built-in types (e.g. `"noop"`) and any types registered via
    /// [`NodeRegistry::register`].
    pub fn node_types(&self) -> Vec<String> {
        self.registry.read().unwrap().list_types()
    }

    /// Return structured descriptors for all registered node types.
    ///
    /// This is the preferred discovery API for building UIs, skill pickers,
    /// and progressive capability endpoints on top of `a3s-flow`.
    pub fn node_descriptors(&self) -> Vec<NodeDescriptor> {
        self.registry.read().unwrap().list_descriptors()
    }

    /// Return a transport-friendly capabilities document for this engine.
    ///
    /// Higher layers can serialize this value directly as JSON for progressive
    /// discovery APIs.
    pub fn capabilities(&self) -> FlowCapabilities {
        FlowCapabilities::from_nodes(self.node_descriptors())
    }

    // ── Node type management ───────────────────────────────────────────────

    /// Register or replace a node type for future executions started by this engine.
    pub fn register_node_type(&self, node: Arc<dyn Node>) {
        self.registry.write().unwrap().register(node);
    }

    /// Register or replace a node type with explicit discovery metadata.
    pub fn register_node_type_with_descriptor(
        &self,
        node: Arc<dyn Node>,
        descriptor: NodeDescriptor,
    ) {
        self.registry
            .write()
            .unwrap()
            .register_with_descriptor(node, descriptor);
    }

    /// Remove a node type from this engine's registry.
    ///
    /// Returns `Ok(true)` if the node type existed and was removed, `Ok(false)`
    /// if it was not registered, or an error if the type is protected.
    ///
    /// Removal only affects future validations and executions. Already running
    /// executions keep using the registry snapshot captured when they were started.
    pub fn unregister_node_type(&self, node_type: &str) -> Result<bool> {
        self.registry.write().unwrap().unregister(node_type)
    }

    // ── Pre-flight validation ──────────────────────────────────────────────

    /// Validate a flow definition without executing it.
    ///
    /// Returns a list of [`ValidationIssue`]s describing structural problems.
    /// An empty list means the definition is valid and ready to run.
    ///
    /// The following checks are performed:
    /// - DAG structural validity: no cycles, no unknown edge references,
    ///   no duplicate node IDs, at least one node.
    /// - All node types are registered in the engine's [`NodeRegistry`].
    /// - Every `run_if.from` field references an existing node ID.
    ///
    /// ```rust
    /// use a3s_flow::{FlowEngine, NodeRegistry};
    /// use serde_json::json;
    ///
    /// let engine = FlowEngine::new(NodeRegistry::with_defaults());
    /// let def = json!({
    ///     "nodes": [
    ///         { "id": "a", "type": "noop" },
    ///         { "id": "b", "type": "unknown-type" }
    ///     ],
    ///     "edges": []
    /// });
    /// let issues = engine.validate(&def);
    /// assert_eq!(issues.len(), 1);
    /// assert!(issues[0].message.contains("unknown node type"));
    /// ```
    pub fn validate(&self, definition: &Value) -> Vec<ValidationIssue> {
        let mut issues = Vec::new();

        // Parse the DAG — catches cycle, unknown refs, duplicate IDs, empty flow.
        let dag = match DagGraph::from_json(definition) {
            Ok(dag) => dag,
            Err(e) => {
                issues.push(ValidationIssue {
                    node_id: None,
                    message: e.to_string(),
                });
                return issues;
            }
        };

        for node_def in dag.nodes_in_order() {
            // Check node type is registered.
            if self
                .registry
                .read()
                .unwrap()
                .get(&node_def.node_type)
                .is_err()
            {
                issues.push(ValidationIssue {
                    node_id: Some(node_def.id.clone()),
                    message: format!("unknown node type '{}'", node_def.node_type),
                });
            }

            // Check run_if.from references an existing node.
            if let Some(ref cond) = node_def.run_if {
                if !dag.nodes.contains_key(&cond.from) {
                    issues.push(ValidationIssue {
                        node_id: Some(node_def.id.clone()),
                        message: format!("run_if references unknown node '{}'", cond.from),
                    });
                }
            }
        }

        issues
    }

    // ── Execution lifecycle ────────────────────────────────────────────────

    /// Start a new workflow execution from a JSON DAG definition.
    ///
    /// The definition is parsed and validated synchronously. If valid, the
    /// execution is launched in a background Tokio task and the execution ID
    /// is returned immediately — the flow runs concurrently with the caller.
    ///
    /// # Errors
    ///
    /// Returns an error if the definition is invalid (cycle, unknown node ID,
    /// bad JSON, unregistered node type).
    pub async fn start(
        &self,
        definition: &Value,
        variables: HashMap<String, Value>,
    ) -> Result<Uuid> {
        let (id, _rx) = self.start_inner(definition, variables).await?;
        Ok(id)
    }

    /// Start a workflow and return a live event stream alongside the execution ID.
    ///
    /// The returned [`broadcast::Receiver<FlowEvent>`] is created **before** the
    /// execution task is spawned, guaranteeing that no events are missed —
    /// including `FlowStarted`. Multiple subscribers can be created by calling
    /// [`broadcast::Receiver::resubscribe`].
    ///
    /// The stream closes (returns `Err(RecvError::Closed)`) when the execution
    /// reaches a terminal state (`Completed`, `Failed`, or `Terminated`).
    ///
    /// If the engine also has a custom [`EventEmitter`] configured via
    /// [`with_event_emitter`](Self::with_event_emitter), both the emitter and
    /// the broadcast channel receive every event.
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use a3s_flow::{FlowEngine, FlowEvent, NodeRegistry};
    /// use serde_json::json;
    /// use std::collections::HashMap;
    ///
    /// #[tokio::main]
    /// async fn main() -> a3s_flow::Result<()> {
    ///     let engine = FlowEngine::new(NodeRegistry::with_defaults());
    ///     let def = json!({
    ///         "nodes": [{ "id": "a", "type": "noop" }],
    ///         "edges": []
    ///     });
    ///
    ///     let (id, mut rx) = engine.start_streaming(&def, HashMap::new()).await?;
    ///
    ///     while let Ok(event) = rx.recv().await {
    ///         match event {
    ///             FlowEvent::NodeCompleted { node_id, .. } => println!("done: {node_id}"),
    ///             FlowEvent::FlowCompleted { .. } => break,
    ///             _ => {}
    ///         }
    ///     }
    ///     Ok(())
    /// }
    /// ```
    pub async fn start_streaming(
        &self,
        definition: &Value,
        variables: HashMap<String, Value>,
    ) -> Result<(Uuid, broadcast::Receiver<FlowEvent>)> {
        let (id, rx) = self.start_inner(definition, variables).await?;
        Ok((id, rx))
    }

    /// Subscribe to live events for an existing execution.
    ///
    /// The returned receiver attaches to the execution's dedicated broadcast
    /// channel. Events emitted before subscription are not replayed.
    pub async fn subscribe(&self, id: Uuid) -> Result<broadcast::Receiver<FlowEvent>> {
        let executions = self.executions.read().await;
        let handle = executions
            .get(&id)
            .ok_or(FlowError::ExecutionNotFound(id))?;
        let receiver = handle
            .event_tx
            .read()
            .unwrap()
            .as_ref()
            .map(|tx| tx.subscribe())
            .ok_or_else(|| FlowError::InvalidTransition {
                action: "subscribe".into(),
                from: "finished".into(),
            })?;
        Ok(receiver)
    }

    /// Internal start implementation shared by both `start` and
    /// `start_streaming`.
    async fn start_inner(
        &self,
        definition: &Value,
        variables: HashMap<String, Value>,
    ) -> Result<(Uuid, broadcast::Receiver<FlowEvent>)> {
        let dag = DagGraph::from_json(definition)?;
        let registry = self.registry.read().unwrap().clone();
        let (event_tx, event_rx) = broadcast::channel(256);
        let channel_emitter = Arc::new(ChannelEmitter::new(event_tx.clone()));
        let emitter: Arc<dyn EventEmitter> = Arc::new(MulticastEmitter {
            a: Arc::clone(&self.emitter),
            b: channel_emitter,
        });
        let mut runner = FlowRunner::new(dag, registry).with_event_emitter(emitter);
        if let Some(ref fs) = self.flow_store {
            runner = runner.with_flow_store(Arc::clone(fs));
        }
        if let Some(n) = self.max_concurrency {
            runner = runner.with_max_concurrency(n);
        }

        let execution_id = Uuid::new_v4();
        let cancel = CancellationToken::new();
        let (signal_tx, signal_rx) = watch::channel(FlowSignal::Run);
        let state = Arc::new(RwLock::new(ExecutionState::Running));
        let context: Arc<SyncRwLock<HashMap<String, Value>>> =
            Arc::new(SyncRwLock::new(HashMap::new()));
        let event_tx_handle = Arc::new(SyncRwLock::new(Some(event_tx)));

        let handle = ExecutionHandle {
            state: Arc::clone(&state),
            signal_tx,
            cancel: cancel.clone(),
            context: Arc::clone(&context),
            event_tx: Arc::clone(&event_tx_handle),
        };

        self.executions.write().await.insert(execution_id, handle);

        // Spawn the execution task; it updates state on terminal transitions.
        // Logging and event emission are handled by FlowRunner::run_seeded.
        let state_for_task = Arc::clone(&state);
        let event_tx_for_task = Arc::clone(&event_tx_handle);
        let execution_store = self.execution_store.clone();
        tokio::spawn(async move {
            match runner
                .run_controlled(execution_id, variables, signal_rx, cancel, context)
                .await
            {
                Ok(result) => {
                    // Persist the result if a store is configured.
                    if let Some(ref store) = execution_store {
                        if let Err(e) = store.save(&result).await {
                            warn!(%execution_id, error = %e, "failed to persist execution result");
                        }
                    }
                    *state_for_task.write().await = ExecutionState::Completed(result);
                }
                Err(FlowError::Terminated) => {
                    *state_for_task.write().await = ExecutionState::Terminated;
                }
                Err(e) => {
                    *state_for_task.write().await = ExecutionState::Failed(e.to_string());
                }
            }
            let _ = event_tx_for_task.write().unwrap().take();
        });

        Ok((execution_id, event_rx))
    }

    /// Start a workflow by loading its definition from the configured
    /// [`FlowStore`] by name.
    ///
    /// Equivalent to:
    /// ```rust,ignore
    /// let def = flow_store.load(name).await?.ok_or(...)?;
    /// engine.start(&def, variables).await
    /// ```
    ///
    /// # Errors
    ///
    /// - [`FlowError::Internal`] if no `FlowStore` was configured via
    ///   [`with_flow_store`](Self::with_flow_store).
    /// - [`FlowError::FlowNotFound`] if no definition exists under `name`.
    /// - Any error returned by [`start`](Self::start) (invalid definition, etc.).
    pub async fn start_named(&self, name: &str, variables: HashMap<String, Value>) -> Result<Uuid> {
        let store = self.flow_store.as_ref().ok_or_else(|| {
            FlowError::Internal("no FlowStore configured; call with_flow_store first".into())
        })?;

        let definition = store
            .load(name)
            .await?
            .ok_or_else(|| FlowError::FlowNotFound(name.to_string()))?;

        self.start(&definition, variables).await
    }

    /// Pause a running execution at the next wave boundary.
    ///
    /// Nodes in the **current wave** continue until they finish. No new wave
    /// starts until [`resume`](Self::resume) is called.
    ///
    /// # Errors
    ///
    /// - [`FlowError::ExecutionNotFound`] if the ID is unknown.
    /// - [`FlowError::InvalidTransition`] if the execution is not `Running`.
    pub async fn pause(&self, id: Uuid) -> Result<()> {
        let executions = self.executions.read().await;
        let handle = executions
            .get(&id)
            .ok_or(FlowError::ExecutionNotFound(id))?;

        let mut state = handle.state.write().await;
        match *state {
            ExecutionState::Running => {
                handle.signal_tx.send(FlowSignal::Pause).ok();
                *state = ExecutionState::Paused;
                Ok(())
            }
            ref s => Err(FlowError::InvalidTransition {
                action: "pause".into(),
                from: s.as_str().into(),
            }),
        }
    }

    /// Resume a paused execution.
    ///
    /// # Errors
    ///
    /// - [`FlowError::ExecutionNotFound`] if the ID is unknown.
    /// - [`FlowError::InvalidTransition`] if the execution is not `Paused`.
    pub async fn resume(&self, id: Uuid) -> Result<()> {
        let executions = self.executions.read().await;
        let handle = executions
            .get(&id)
            .ok_or(FlowError::ExecutionNotFound(id))?;

        let mut state = handle.state.write().await;
        match *state {
            ExecutionState::Paused => {
                handle.signal_tx.send(FlowSignal::Run).ok();
                *state = ExecutionState::Running;
                Ok(())
            }
            ref s => Err(FlowError::InvalidTransition {
                action: "resume".into(),
                from: s.as_str().into(),
            }),
        }
    }

    /// Terminate an execution immediately.
    ///
    /// Sends a cancellation signal. The execution task stops at the next
    /// cancellation checkpoint (between waves, or within a wave's result
    /// collection). If the execution is currently paused it is unblocked so
    /// it can observe the cancellation.
    ///
    /// # Errors
    ///
    /// - [`FlowError::ExecutionNotFound`] if the ID is unknown.
    /// - [`FlowError::InvalidTransition`] if the execution is already in a
    ///   terminal state (`Completed`, `Failed`, `Terminated`).
    pub async fn terminate(&self, id: Uuid) -> Result<()> {
        let executions = self.executions.read().await;
        let handle = executions
            .get(&id)
            .ok_or(FlowError::ExecutionNotFound(id))?;

        let state = handle.state.read().await;
        if state.is_terminal() {
            return Err(FlowError::InvalidTransition {
                action: "terminate".into(),
                from: state.as_str().into(),
            });
        }
        drop(state);

        handle.cancel.cancel();
        // Unblock a paused runner so it can observe the cancellation.
        handle.signal_tx.send(FlowSignal::Run).ok();
        Ok(())
    }

    /// Return a snapshot of the current state of an execution.
    ///
    /// # Errors
    ///
    /// - [`FlowError::ExecutionNotFound`] if the ID is unknown.
    pub async fn state(&self, id: Uuid) -> Result<ExecutionState> {
        let executions = self.executions.read().await;
        let handle = executions
            .get(&id)
            .ok_or(FlowError::ExecutionNotFound(id))?;
        // Clone while the guard is still held, then drop the guard before returning.
        let snapshot = handle.state.read().await.clone();
        Ok(snapshot)
    }

    // ── Context CRUD ───────────────────────────────────────────────────────

    /// Return a snapshot of the shared mutable context for a running execution.
    ///
    /// The context is a `HashMap<String, Value>` that nodes may read and write
    /// via [`ExecContext::context`](crate::node::ExecContext::context) during
    /// execution. This method lets the caller inspect (or react to) the
    /// accumulated state from outside the runner.
    ///
    /// # Errors
    ///
    /// - [`FlowError::ExecutionNotFound`] if the ID is unknown.
    pub async fn get_context(&self, id: Uuid) -> Result<HashMap<String, Value>> {
        let executions = self.executions.read().await;
        let handle = executions
            .get(&id)
            .ok_or(FlowError::ExecutionNotFound(id))?;
        let snapshot = handle.context.read().unwrap().clone();
        Ok(snapshot)
    }

    /// Insert or overwrite a single entry in the shared context of a running
    /// execution.
    ///
    /// The change is immediately visible to any node that reads the context
    /// after this call returns.
    ///
    /// # Errors
    ///
    /// - [`FlowError::ExecutionNotFound`] if the ID is unknown.
    pub async fn set_context_entry(&self, id: Uuid, key: String, value: Value) -> Result<()> {
        let executions = self.executions.read().await;
        let handle = executions
            .get(&id)
            .ok_or(FlowError::ExecutionNotFound(id))?;
        handle.context.write().unwrap().insert(key, value);
        Ok(())
    }

    /// Remove a single entry from the shared context of a running execution.
    ///
    /// Returns `true` if the key existed and was removed, `false` if it was
    /// not present.
    ///
    /// # Errors
    ///
    /// - [`FlowError::ExecutionNotFound`] if the ID is unknown.
    pub async fn delete_context_entry(&self, id: Uuid, key: &str) -> Result<bool> {
        let executions = self.executions.read().await;
        let handle = executions
            .get(&id)
            .ok_or(FlowError::ExecutionNotFound(id))?;
        let removed = handle.context.write().unwrap().remove(key).is_some();
        Ok(removed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::node::{ExecContext, Node};
    use async_trait::async_trait;
    use serde_json::{json, Value};
    use std::time::Duration;

    // ── Helpers ────────────────────────────────────────────────────────────

    /// A node that sleeps for the given duration before returning.
    struct SlowNode(Duration);

    #[async_trait]
    impl Node for SlowNode {
        fn node_type(&self) -> &str {
            "slow"
        }

        async fn execute(&self, _ctx: ExecContext) -> crate::error::Result<Value> {
            tokio::time::sleep(self.0).await;
            Ok(json!({}))
        }
    }

    fn slow_engine(delay: Duration) -> FlowEngine {
        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(SlowNode(delay)));
        FlowEngine::new(registry)
    }

    fn simple_def() -> Value {
        json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "b", "type": "noop" }
            ],
            "edges": [{ "source": "a", "target": "b" }]
        })
    }

    fn slow_def() -> Value {
        json!({
            "nodes": [
                { "id": "a", "type": "slow" },
                { "id": "b", "type": "slow" }
            ],
            "edges": [{ "source": "a", "target": "b" }]
        })
    }

    // ── node_types ─────────────────────────────────────────────────────────

    #[test]
    fn node_types_includes_builtins() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let types = engine.node_types();
        assert!(types.contains(&"noop".to_string()));
    }

    #[test]
    fn node_types_includes_custom_nodes() {
        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(SlowNode(Duration::from_millis(1))));
        let engine = FlowEngine::new(registry);

        let types = engine.node_types();
        assert!(types.contains(&"noop".to_string()));
        assert!(types.contains(&"slow".to_string()));
    }

    #[test]
    fn node_types_is_sorted() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let types = engine.node_types();
        let mut sorted = types.clone();
        sorted.sort();
        assert_eq!(types, sorted);
    }

    #[test]
    fn node_descriptors_include_builtin_metadata() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let descriptors = engine.node_descriptors();
        let llm = descriptors
            .iter()
            .find(|descriptor| descriptor.node_type == "llm")
            .unwrap();
        assert_eq!(llm.display_name, "LLM");
        assert_eq!(llm.category, "ai");
        assert!(llm.summary.contains("OpenAI-compatible"));
        assert!(llm.default_data.is_object());
        assert!(!llm.fields.is_empty());
    }

    #[test]
    fn node_descriptors_include_custom_nodes() {
        let mut registry = NodeRegistry::with_defaults();
        registry.register(Arc::new(SlowNode(Duration::from_millis(1))));
        let engine = FlowEngine::new(registry);

        let descriptors = engine.node_descriptors();
        let slow = descriptors
            .iter()
            .find(|descriptor| descriptor.node_type == "slow")
            .unwrap();
        assert_eq!(slow.display_name, "slow");
        assert_eq!(slow.category, "custom");
    }

    #[test]
    fn register_node_type_adds_custom_type_at_runtime() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        engine.register_node_type(Arc::new(SlowNode(Duration::from_millis(1))));

        let types = engine.node_types();
        assert!(types.contains(&"slow".to_string()));
    }

    #[test]
    fn register_node_type_with_descriptor_updates_catalog() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        engine.register_node_type_with_descriptor(
            Arc::new(SlowNode(Duration::from_millis(1))),
            NodeDescriptor {
                node_type: "ignored".to_string(),
                display_name: "Slow Node".to_string(),
                category: "testing".to_string(),
                summary: "Sleeps briefly during tests.".to_string(),
                default_data: json!({ "delay_ms": 1 }),
                fields: vec![],
            },
        );

        let slow = engine
            .node_descriptors()
            .into_iter()
            .find(|descriptor| descriptor.node_type == "slow")
            .unwrap();
        assert_eq!(slow.display_name, "Slow Node");
        assert_eq!(slow.category, "testing");
        assert_eq!(slow.default_data["delay_ms"], 1);
    }

    #[test]
    fn unregister_node_type_removes_runtime_type() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        engine.register_node_type(Arc::new(SlowNode(Duration::from_millis(1))));

        assert!(engine.unregister_node_type("slow").unwrap());
        assert!(!engine.node_types().contains(&"slow".to_string()));

        let def = json!({
            "nodes": [{ "id": "a", "type": "slow" }],
            "edges": []
        });
        let issues = engine.validate(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].message.contains("unknown node type"));
    }

    #[test]
    fn unregister_node_type_rejects_builtin_types() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());

        let err = engine.unregister_node_type("noop").unwrap_err();
        assert!(matches!(err, FlowError::ProtectedNodeType(ref ty) if ty == "noop"));
        assert!(engine.node_types().contains(&"noop".to_string()));
    }

    #[test]
    fn capabilities_include_node_catalog() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let capabilities = engine.capabilities();
        assert_eq!(capabilities.version, "2026-03-22");
        assert!(capabilities.progressive_disclosure);
        assert!(capabilities
            .nodes
            .iter()
            .any(|node| node.node_type == "llm"));
    }

    #[test]
    fn http_request_descriptor_carries_editor_metadata() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let descriptors = engine.node_descriptors();
        let http = descriptors
            .iter()
            .find(|descriptor| descriptor.node_type == "http-request")
            .unwrap();
        assert_eq!(http.default_data["method"], "GET");
        assert!(http.fields.iter().any(|field| field.key == "url"));
    }

    // ── start ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn start_returns_execution_id() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let id = engine.start(&simple_def(), HashMap::new()).await.unwrap();
        // ID is non-nil.
        assert!(!id.is_nil());
    }

    #[tokio::test]
    async fn start_rejects_invalid_definition() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let bad = json!({
            "nodes": [{ "id": "a", "type": "noop" }],
            "edges": [{ "source": "ghost", "target": "a" }]
        });
        assert!(matches!(
            engine.start(&bad, HashMap::new()).await,
            Err(FlowError::UnknownNode(_))
        ));
    }

    #[tokio::test]
    async fn completed_flow_has_outputs() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let id = engine.start(&simple_def(), HashMap::new()).await.unwrap();

        // Wait for the background task to finish.
        tokio::time::sleep(Duration::from_millis(50)).await;

        let state = engine.state(id).await.unwrap();
        if let ExecutionState::Completed(result) = state {
            assert!(result.outputs.contains_key("a"));
            assert!(result.outputs.contains_key("b"));
        } else {
            panic!("expected Completed, got {}", state.as_str());
        }
    }

    // ── state ──────────────────────────────────────────────────────────────

    #[tokio::test]
    async fn state_returns_not_found_for_unknown_id() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let err = engine.state(Uuid::new_v4()).await.unwrap_err();
        assert!(matches!(err, FlowError::ExecutionNotFound(_)));
    }

    // ── pause / resume ─────────────────────────────────────────────────────

    #[tokio::test]
    async fn pause_transitions_to_paused() {
        let engine = slow_engine(Duration::from_millis(200));
        let id = engine.start(&slow_def(), HashMap::new()).await.unwrap();

        // Give the runner a moment to start wave 1.
        tokio::time::sleep(Duration::from_millis(10)).await;
        engine.pause(id).await.unwrap();

        assert!(matches!(
            engine.state(id).await.unwrap(),
            ExecutionState::Paused
        ));
    }

    #[tokio::test]
    async fn resume_transitions_to_running() {
        let engine = slow_engine(Duration::from_millis(200));
        let id = engine.start(&slow_def(), HashMap::new()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await;
        engine.pause(id).await.unwrap();
        engine.resume(id).await.unwrap();

        assert!(matches!(
            engine.state(id).await.unwrap(),
            ExecutionState::Running
        ));
    }

    #[tokio::test]
    async fn pause_on_completed_flow_returns_invalid_transition() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let id = engine.start(&simple_def(), HashMap::new()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        // Flow should be completed by now.
        let err = engine.pause(id).await.unwrap_err();
        assert!(matches!(err, FlowError::InvalidTransition { .. }));
    }

    #[tokio::test]
    async fn resume_on_running_flow_returns_invalid_transition() {
        let engine = slow_engine(Duration::from_millis(200));
        let id = engine.start(&slow_def(), HashMap::new()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await;
        // Still running — not paused.
        let err = engine.resume(id).await.unwrap_err();
        assert!(matches!(err, FlowError::InvalidTransition { .. }));

        engine.terminate(id).await.unwrap();
    }

    // ── terminate ──────────────────────────────────────────────────────────

    #[tokio::test]
    async fn terminate_stops_slow_execution() {
        let engine = slow_engine(Duration::from_millis(500));
        let id = engine.start(&slow_def(), HashMap::new()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await;
        engine.terminate(id).await.unwrap();

        // The runner task should observe the cancellation quickly.
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert!(matches!(
            engine.state(id).await.unwrap(),
            ExecutionState::Terminated
        ));
    }

    #[tokio::test]
    async fn terminate_unblocks_paused_execution() {
        let engine = slow_engine(Duration::from_millis(500));
        let id = engine.start(&slow_def(), HashMap::new()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(10)).await;
        engine.pause(id).await.unwrap();

        // Flow is paused — now terminate it.
        engine.terminate(id).await.unwrap();

        tokio::time::sleep(Duration::from_millis(600)).await;

        assert!(matches!(
            engine.state(id).await.unwrap(),
            ExecutionState::Terminated
        ));
    }

    #[tokio::test]
    async fn terminate_on_completed_flow_returns_invalid_transition() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let id = engine.start(&simple_def(), HashMap::new()).await.unwrap();

        tokio::time::sleep(Duration::from_millis(50)).await;
        let err = engine.terminate(id).await.unwrap_err();
        assert!(matches!(err, FlowError::InvalidTransition { .. }));
    }

    #[tokio::test]
    async fn unknown_execution_id_returns_not_found() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let id = Uuid::new_v4();
        assert!(matches!(
            engine.pause(id).await,
            Err(FlowError::ExecutionNotFound(_))
        ));
        assert!(matches!(
            engine.resume(id).await,
            Err(FlowError::ExecutionNotFound(_))
        ));
        assert!(matches!(
            engine.terminate(id).await,
            Err(FlowError::ExecutionNotFound(_))
        ));
    }

    // ── ExecutionStore integration ──────────────────────────────────────────

    #[tokio::test]
    async fn execution_store_saves_completed_result() {
        use crate::store::MemoryExecutionStore;

        let store = Arc::new(MemoryExecutionStore::new());
        let engine = FlowEngine::new(NodeRegistry::with_defaults())
            .with_execution_store(Arc::clone(&store) as Arc<dyn crate::store::ExecutionStore>);

        let id = engine.start(&simple_def(), HashMap::new()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Engine should have saved the result.
        let ids = store.list().await.unwrap();
        assert!(ids.contains(&id), "stored execution id not found");

        let saved = store.load(id).await.unwrap().unwrap();
        assert_eq!(saved.execution_id, id);
        assert!(saved.outputs.contains_key("a"));
        assert!(saved.outputs.contains_key("b"));
    }

    #[tokio::test]
    async fn execution_store_not_used_on_terminated_execution() {
        use crate::store::MemoryExecutionStore;

        let store = Arc::new(MemoryExecutionStore::new());
        let engine = slow_engine(Duration::from_millis(500))
            .with_execution_store(Arc::clone(&store) as Arc<dyn crate::store::ExecutionStore>);

        let id = engine.start(&slow_def(), HashMap::new()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        engine.terminate(id).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        // Terminated executions are not saved.
        assert!(
            store.list().await.unwrap().is_empty(),
            "terminated result should not be stored"
        );
    }

    // ── EventEmitter integration (via engine) ───────────────────────────────

    #[tokio::test]
    async fn engine_emitter_receives_flow_and_node_events() {
        use crate::event::EventEmitter;
        use std::sync::atomic::{AtomicU32, Ordering};

        struct CountEmitter {
            flow_started: Arc<AtomicU32>,
            flow_completed: Arc<AtomicU32>,
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

        #[async_trait::async_trait]
        impl EventEmitter for CountEmitter {
            async fn on_flow_started(&self, _: Uuid) {
                self.flow_started.fetch_add(1, Ordering::SeqCst);
            }
            async fn on_flow_completed(&self, _: Uuid, _: &crate::result::FlowResult) {
                self.flow_completed.fetch_add(1, Ordering::SeqCst);
            }
            async fn on_flow_failed(&self, _: Uuid, _: &str) {}
            async fn on_flow_terminated(&self, _: Uuid) {}
            async fn on_node_started(&self, _: Uuid, _: &str, _: &str) {
                self.node_started.fetch_add(1, Ordering::SeqCst);
            }
            async fn on_node_completed(&self, _: Uuid, _: &str, _: &serde_json::Value) {
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
                _: &serde_json::Value,
                _: Option<&serde_json::Value>,
                _: &serde_json::Value,
                _: u64,
            ) {
                self.node_completed_full.fetch_add(1, Ordering::SeqCst);
            }
            async fn on_iteration_started(&self, _: Uuid, _: &str, _: &str, _: u32) {}
            async fn on_iteration_next(&self, _: Uuid, _: &str, _: &str, _: u32) {}
            async fn on_iteration_completed(&self, _: Uuid, _: &str, _: &str) {}
            async fn on_loop_started(&self, _: Uuid, _: &str, _: &str, _: u32) {}
            async fn on_loop_completed(&self, _: Uuid, _: &str, _: &str) {}
            async fn on_parallel_branch_started(&self, _: Uuid, _: &str, _: &str, _: &str) {}
            async fn on_parallel_branch_completed(
                &self,
                _: Uuid,
                _: &str,
                _: &str,
                _: &str,
                _: &serde_json::Value,
            ) {
            }
            async fn on_node_retry(&self, _: Uuid, _: &str, _: u32, _: u32) {}
        }

        let flow_started = Arc::new(AtomicU32::new(0));
        let flow_completed = Arc::new(AtomicU32::new(0));
        let node_started = Arc::new(AtomicU32::new(0));
        let node_completed = Arc::new(AtomicU32::new(0));
        let node_skipped = Arc::new(AtomicU32::new(0));
        let node_failed = Arc::new(AtomicU32::new(0));
        let node_completed_full = Arc::new(AtomicU32::new(0));
        let iteration_started = Arc::new(AtomicU32::new(0));
        let iteration_next = Arc::new(AtomicU32::new(0));
        let iteration_completed = Arc::new(AtomicU32::new(0));
        let loop_started = Arc::new(AtomicU32::new(0));
        let loop_completed = Arc::new(AtomicU32::new(0));
        let parallel_branch_started = Arc::new(AtomicU32::new(0));
        let parallel_branch_completed = Arc::new(AtomicU32::new(0));
        let node_retry = Arc::new(AtomicU32::new(0));

        let emitter = Arc::new(CountEmitter {
            flow_started: Arc::clone(&flow_started),
            flow_completed: Arc::clone(&flow_completed),
            node_started: Arc::clone(&node_started),
            node_completed: Arc::clone(&node_completed),
            node_skipped: Arc::clone(&node_skipped),
            node_failed: Arc::clone(&node_failed),
            node_completed_full: Arc::clone(&node_completed_full),
            iteration_started: Arc::clone(&iteration_started),
            iteration_next: Arc::clone(&iteration_next),
            iteration_completed: Arc::clone(&iteration_completed),
            loop_started: Arc::clone(&loop_started),
            loop_completed: Arc::clone(&loop_completed),
            parallel_branch_started: Arc::clone(&parallel_branch_started),
            parallel_branch_completed: Arc::clone(&parallel_branch_completed),
            node_retry: Arc::clone(&node_retry),
        });

        let engine = FlowEngine::new(NodeRegistry::with_defaults())
            .with_event_emitter(emitter as Arc<dyn EventEmitter>);

        // simple_def has nodes a and b.
        engine.start(&simple_def(), HashMap::new()).await.unwrap();
        tokio::time::sleep(Duration::from_millis(50)).await;

        assert_eq!(flow_started.load(Ordering::SeqCst), 1, "flow_started");
        assert_eq!(flow_completed.load(Ordering::SeqCst), 1, "flow_completed");
        assert_eq!(node_started.load(Ordering::SeqCst), 2, "node_started (a+b)");
        assert_eq!(
            node_completed.load(Ordering::SeqCst),
            2,
            "node_completed (a+b)"
        );
    }

    // ── start_named ────────────────────────────────────────────────────────

    #[tokio::test]
    async fn start_named_loads_and_runs_from_flow_store() {
        use crate::flow_store::MemoryFlowStore;

        let flow_store = Arc::new(MemoryFlowStore::new());
        flow_store.save("greet", &simple_def()).await.unwrap();

        let engine = FlowEngine::new(NodeRegistry::with_defaults())
            .with_flow_store(Arc::clone(&flow_store) as Arc<dyn crate::flow_store::FlowStore>);

        let id = engine.start_named("greet", HashMap::new()).await.unwrap();
        assert!(!id.is_nil());

        tokio::time::sleep(Duration::from_millis(50)).await;
        assert!(matches!(
            engine.state(id).await.unwrap(),
            ExecutionState::Completed(_)
        ));
    }

    #[tokio::test]
    async fn start_named_returns_flow_not_found_for_unknown_name() {
        use crate::flow_store::MemoryFlowStore;

        let engine = FlowEngine::new(NodeRegistry::with_defaults())
            .with_flow_store(
                Arc::new(MemoryFlowStore::new()) as Arc<dyn crate::flow_store::FlowStore>
            );

        let err = engine
            .start_named("nonexistent", HashMap::new())
            .await
            .unwrap_err();

        assert!(
            matches!(err, FlowError::FlowNotFound(ref n) if n == "nonexistent"),
            "expected FlowNotFound, got: {err}"
        );
    }

    #[tokio::test]
    async fn start_named_returns_internal_when_no_store_configured() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());

        let err = engine
            .start_named("anything", HashMap::new())
            .await
            .unwrap_err();

        assert!(
            matches!(err, FlowError::Internal(_)),
            "expected Internal, got: {err}"
        );
    }

    // ── start_streaming ────────────────────────────────────────────────────

    #[tokio::test]
    async fn start_streaming_delivers_flow_started_and_completed_events() {
        use crate::event::FlowEvent;

        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let (_, mut rx) = engine
            .start_streaming(&simple_def(), HashMap::new())
            .await
            .unwrap();

        let mut saw_started = false;
        let mut saw_completed = false;

        loop {
            match rx.recv().await {
                Ok(FlowEvent::FlowStarted { .. }) => saw_started = true,
                Ok(FlowEvent::FlowCompleted { .. }) => {
                    saw_completed = true;
                    break;
                }
                Ok(_) => {}
                Err(_) => break,
            }
        }

        assert!(saw_started, "FlowStarted not received");
        assert!(saw_completed, "FlowCompleted not received");
    }

    #[tokio::test]
    async fn start_streaming_delivers_node_events_for_each_node() {
        use crate::event::FlowEvent;
        use std::collections::HashSet;

        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let (_, mut rx) = engine
            .start_streaming(&simple_def(), HashMap::new())
            .await
            .unwrap();

        let mut completed_nodes: HashSet<String> = HashSet::new();

        loop {
            match rx.recv().await {
                Ok(FlowEvent::NodeCompleted { node_id, .. }) => {
                    completed_nodes.insert(node_id);
                }
                Ok(FlowEvent::FlowCompleted { .. }) | Err(_) => break,
                Ok(_) => {}
            }
        }

        assert!(completed_nodes.contains("a"), "node 'a' not in stream");
        assert!(completed_nodes.contains("b"), "node 'b' not in stream");
    }

    #[tokio::test]
    async fn start_streaming_zero_events_lost_on_fast_flow() {
        // Sanity check: even on an instantaneously completing flow the
        // receiver created before spawn misses no events.
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let def = json!({ "nodes": [{ "id": "x", "type": "noop" }], "edges": [] });

        let (_id, mut rx) = engine.start_streaming(&def, HashMap::new()).await.unwrap();

        let mut event_count = 0u32;
        loop {
            match rx.recv().await {
                Ok(_) => event_count += 1,
                Err(_) => break,
            }
        }
        // FlowStarted + NodeStarted + NodeCompleted + FlowCompleted = 4 minimum
        assert!(event_count >= 4, "expected ≥4 events, got {event_count}");
    }

    #[tokio::test]
    async fn start_streaming_existing_emitter_also_fires() {
        use crate::event::{EventEmitter, FlowEvent};
        use std::sync::atomic::{AtomicU32, Ordering};

        struct CountEmitter(Arc<AtomicU32>);

        #[async_trait::async_trait]
        impl EventEmitter for CountEmitter {
            async fn on_flow_started(&self, _: Uuid) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
            async fn on_flow_completed(&self, _: Uuid, _: &crate::result::FlowResult) {}
            async fn on_flow_failed(&self, _: Uuid, _: &str) {}
            async fn on_flow_terminated(&self, _: Uuid) {}
            async fn on_node_started(&self, _: Uuid, _: &str, _: &str) {}
            async fn on_node_completed(&self, _: Uuid, _: &str, _: &serde_json::Value) {}
            async fn on_node_skipped(&self, _: Uuid, _: &str) {}
            async fn on_node_failed(&self, _: Uuid, _: &str, _: &str) {}
            async fn on_node_completed_full(
                &self,
                _: Uuid,
                _: &str,
                _: &str,
                _: &serde_json::Value,
                _: Option<&serde_json::Value>,
                _: &serde_json::Value,
                _: u64,
            ) {
            }
            async fn on_iteration_started(&self, _: Uuid, _: &str, _: &str, _: u32) {}
            async fn on_iteration_next(&self, _: Uuid, _: &str, _: &str, _: u32) {}
            async fn on_iteration_completed(&self, _: Uuid, _: &str, _: &str) {}
            async fn on_loop_started(&self, _: Uuid, _: &str, _: &str, _: u32) {}
            async fn on_loop_completed(&self, _: Uuid, _: &str, _: &str) {}
            async fn on_parallel_branch_started(&self, _: Uuid, _: &str, _: &str, _: &str) {}
            async fn on_parallel_branch_completed(
                &self,
                _: Uuid,
                _: &str,
                _: &str,
                _: &str,
                _: &serde_json::Value,
            ) {
            }
            async fn on_node_retry(&self, _: Uuid, _: &str, _: u32, _: u32) {}
        }

        let counter = Arc::new(AtomicU32::new(0));
        let engine = FlowEngine::new(NodeRegistry::with_defaults())
            .with_event_emitter(
                Arc::new(CountEmitter(Arc::clone(&counter))) as Arc<dyn EventEmitter>
            );

        let (_id, mut rx) = engine
            .start_streaming(&simple_def(), HashMap::new())
            .await
            .unwrap();

        // Drain the stream.
        loop {
            match rx.recv().await {
                Ok(FlowEvent::FlowCompleted { .. }) | Err(_) => break,
                Ok(_) => {}
            }
        }

        // The existing CountEmitter should also have received FlowStarted.
        assert_eq!(
            counter.load(Ordering::SeqCst),
            1,
            "existing emitter did not fire"
        );
    }

    // ── validate ───────────────────────────────────────────────────────────

    #[test]
    fn validate_returns_empty_for_valid_flow() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "b", "type": "noop" }
            ],
            "edges": [{ "source": "a", "target": "b" }]
        });
        assert!(engine.validate(&def).is_empty());
    }

    #[test]
    fn validate_catches_unknown_node_type() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "b", "type": "does-not-exist" }
            ],
            "edges": []
        });
        let issues = engine.validate(&def);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].node_id.as_deref(), Some("b"));
        assert!(issues[0].message.contains("unknown node type"));
    }

    #[test]
    fn validate_catches_cyclic_graph() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                { "id": "b", "type": "noop" }
            ],
            "edges": [
                { "source": "a", "target": "b" },
                { "source": "b", "target": "a" }
            ]
        });
        let issues = engine.validate(&def);
        assert_eq!(issues.len(), 1);
        assert!(issues[0].node_id.is_none());
    }

    #[test]
    fn validate_catches_run_if_referencing_unknown_node() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let def = json!({
            "nodes": [
                { "id": "a", "type": "noop" },
                {
                    "id": "b",
                    "type": "noop",
                    "data": {
                        "run_if": { "from": "ghost", "path": "", "op": "eq", "value": true }
                    }
                }
            ],
            "edges": [{ "source": "a", "target": "b" }]
        });
        let issues = engine.validate(&def);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].node_id.as_deref(), Some("b"));
        assert!(issues[0].message.contains("ghost"));
    }

    #[test]
    fn validate_reports_multiple_issues() {
        let engine = FlowEngine::new(NodeRegistry::with_defaults());
        let def = json!({
            "nodes": [
                { "id": "a", "type": "bad-type-1" },
                { "id": "b", "type": "bad-type-2" }
            ],
            "edges": []
        });
        assert_eq!(engine.validate(&def).len(), 2);
    }
}
