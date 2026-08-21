//! Node trait and execution context.
//!
//! Every workflow step implements [`Node`]. The engine calls [`Node::execute`]
//! with an [`ExecContext`] containing the node's configuration, upstream outputs,
//! global variables, and a reference to the active [`NodeRegistry`] — which
//! lets nodes like `"iteration"` spin up sub-flow runners without holding their
//! own registry reference.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use serde::Deserialize;
use serde_json::Value;

use crate::error::Result;
use crate::flow_store::FlowStore;
use crate::registry::NodeRegistry;

/// Per-node retry configuration, parsed from `data["retry"]`.
///
/// When present, the runner will re-attempt a failed node up to `max_attempts`
/// times (including the first attempt) with an optional exponential backoff.
///
/// # Example (in flow definition)
/// ```json
/// {
///   "id": "fetch",
///   "type": "http-request",
///   "data": {
///     "url": "https://api.example.com/items",
///     "retry": { "max_attempts": 3, "backoff_ms": 500 }
///   }
/// }
/// ```
#[derive(Debug, Clone, Deserialize)]
pub struct RetryPolicy {
    /// Maximum number of attempts (including the first). Minimum effective value: 1.
    pub max_attempts: u32,
    /// Base delay in milliseconds between attempts. Each subsequent retry waits
    /// `backoff_ms * 2^(attempt-1)` milliseconds (capped at 64× the base).
    /// Defaults to 0 (no delay).
    #[serde(default)]
    pub backoff_ms: u64,
}

/// Runtime context passed to every node during execution.
///
/// - `data` — the static configuration declared in the flow definition's `data` field.
/// - `inputs` — outputs of all upstream nodes, keyed by node ID.
/// - `variables` — global flow-level variables (env, secrets, user inputs).
/// - `context` — shared mutable context for cross-node state (similar to Dify's global context).
///   Nodes can read and write to this context using `context.read()` and `context.write()`.
/// - `registry` — the active node registry; available to nodes that need to
///   execute sub-flows (e.g. `"iteration"`, `"sub-flow"`).
/// - `flow_store` — optional named flow definition store; required by the
///   `"sub-flow"` node to load its target definition by name.
pub struct ExecContext {
    /// Node configuration from the flow definition's `data` field.
    pub data: Value,
    /// Outputs of upstream nodes, keyed by node ID.
    pub inputs: HashMap<String, Value>,
    /// Global flow variables (env, secrets, user inputs).
    pub variables: HashMap<String, Value>,
    /// Shared mutable context for cross-node state (Dify-style global context).
    /// Use `context.read()` to read and `context.write()` to modify.
    pub context: Arc<RwLock<HashMap<String, Value>>>,
    /// Active node registry — allows nodes to run sub-flows.
    pub registry: Arc<NodeRegistry>,
    /// Named flow definition store — available when the engine has one configured.
    pub flow_store: Option<Arc<dyn FlowStore>>,
}

impl Clone for ExecContext {
    fn clone(&self) -> Self {
        Self {
            data: self.data.clone(),
            inputs: self.inputs.clone(),
            variables: self.variables.clone(),
            context: Arc::clone(&self.context),
            registry: Arc::clone(&self.registry),
            flow_store: self.flow_store.clone(),
        }
    }
}

impl std::fmt::Debug for ExecContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ExecContext")
            .field("data", &self.data)
            .field("inputs", &self.inputs)
            .field("variables", &self.variables)
            .field("context", &"<RwLock>")
            .finish_non_exhaustive()
    }
}

impl Default for ExecContext {
    fn default() -> Self {
        Self {
            data: Value::Null,
            inputs: HashMap::new(),
            variables: HashMap::new(),
            context: Arc::new(RwLock::new(HashMap::new())),
            registry: Arc::new(NodeRegistry::with_defaults()),
            flow_store: None,
        }
    }
}

/// The extension point for workflow nodes.
///
/// Implement this trait to add custom node types (HTTP call, LLM prompt,
/// script, condition branch, sub-flow, etc.). Every implementation must be
/// `Send + Sync` so the runner can execute nodes concurrently across threads.
#[async_trait]
pub trait Node: Send + Sync {
    /// The node type identifier matched against the `"type"` field in the
    /// flow definition and looked up in [`NodeRegistry`].
    fn node_type(&self) -> &str;

    /// Execute the node and return a JSON output value.
    ///
    /// The output is stored under this node's ID and passed as `inputs` to
    /// all downstream nodes.
    async fn execute(&self, ctx: ExecContext) -> Result<Value>;
}
