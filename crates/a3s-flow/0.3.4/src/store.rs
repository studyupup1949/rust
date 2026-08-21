//! [`ExecutionStore`] — execution history persistence extension point.
//!
//! Implement [`ExecutionStore`] to persist [`FlowResult`] objects across process
//! restarts, enabling auditing, replay, and partial-resume workflows.
//!
//! Register a store via
//! [`FlowEngine::with_execution_store`](crate::engine::FlowEngine::with_execution_store).
//! The engine automatically saves completed results to the store.
//! [`MemoryExecutionStore`] is provided as an in-process default.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::error::Result;
use crate::result::FlowResult;

/// Persistence layer for completed flow execution results.
///
/// # Example
///
/// ```rust
/// use a3s_flow::{ExecutionStore, FlowResult, MemoryExecutionStore};
/// use std::sync::Arc;
///
/// # #[tokio::main] async fn main() {
/// let store = Arc::new(MemoryExecutionStore::new());
/// let ids = store.list().await.unwrap();
/// assert!(ids.is_empty());
/// # }
/// ```
#[async_trait]
pub trait ExecutionStore: Send + Sync {
    /// Persist a completed execution result, keyed by `result.execution_id`.
    async fn save(&self, result: &FlowResult) -> Result<()>;

    /// Load a previously saved result by execution ID.
    ///
    /// Returns `None` if no result exists for the given ID.
    async fn load(&self, id: Uuid) -> Result<Option<FlowResult>>;

    /// List all stored execution IDs.
    async fn list(&self) -> Result<Vec<Uuid>>;

    /// Delete a stored result. No-op if the ID is not found.
    async fn delete(&self, id: Uuid) -> Result<()>;
}

/// An in-memory [`ExecutionStore`] backed by a `HashMap` under an `RwLock`.
///
/// Suitable for testing and short-lived processes. Data is lost on restart.
pub struct MemoryExecutionStore {
    inner: Arc<RwLock<HashMap<Uuid, FlowResult>>>,
}

impl MemoryExecutionStore {
    /// Create a new empty store.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryExecutionStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl ExecutionStore for MemoryExecutionStore {
    async fn save(&self, result: &FlowResult) -> Result<()> {
        self.inner
            .write()
            .await
            .insert(result.execution_id, result.clone());
        Ok(())
    }

    async fn load(&self, id: Uuid) -> Result<Option<FlowResult>> {
        Ok(self.inner.read().await.get(&id).cloned())
    }

    async fn list(&self) -> Result<Vec<Uuid>> {
        Ok(self.inner.read().await.keys().cloned().collect())
    }

    async fn delete(&self, id: Uuid) -> Result<()> {
        self.inner.write().await.remove(&id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::{HashMap, HashSet};

    fn make_result() -> FlowResult {
        FlowResult {
            execution_id: Uuid::new_v4(),
            outputs: HashMap::new(),
            completed_nodes: HashSet::new(),
            skipped_nodes: HashSet::new(),
            context: HashMap::new(),
        }
    }

    #[tokio::test]
    async fn save_and_load_round_trip() {
        let store = MemoryExecutionStore::new();
        let r = make_result();
        let id = r.execution_id;

        store.save(&r).await.unwrap();
        let loaded = store.load(id).await.unwrap().unwrap();
        assert_eq!(loaded.execution_id, id);
    }

    #[tokio::test]
    async fn load_unknown_id_returns_none() {
        let store = MemoryExecutionStore::new();
        let result = store.load(Uuid::new_v4()).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_returns_all_saved_ids() {
        let store = MemoryExecutionStore::new();
        let r1 = make_result();
        let r2 = make_result();
        let id1 = r1.execution_id;
        let id2 = r2.execution_id;

        store.save(&r1).await.unwrap();
        store.save(&r2).await.unwrap();

        let ids = store.list().await.unwrap();
        assert_eq!(ids.len(), 2);
        assert!(ids.contains(&id1));
        assert!(ids.contains(&id2));
    }

    #[tokio::test]
    async fn delete_removes_entry() {
        let store = MemoryExecutionStore::new();
        let r = make_result();
        let id = r.execution_id;

        store.save(&r).await.unwrap();
        store.delete(id).await.unwrap();

        assert!(store.load(id).await.unwrap().is_none());
        assert!(store.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn save_overwrites_existing_entry() {
        let store = MemoryExecutionStore::new();
        let mut r = make_result();
        let id = r.execution_id;

        store.save(&r).await.unwrap();

        // Save again with a modified outputs map — should overwrite.
        r.outputs.insert("x".into(), serde_json::json!(42));
        store.save(&r).await.unwrap();

        let loaded = store.load(id).await.unwrap().unwrap();
        assert_eq!(loaded.outputs["x"], serde_json::json!(42));

        // Still only one entry.
        assert_eq!(store.list().await.unwrap().len(), 1);
    }
}
