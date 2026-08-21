//! [`FlowStore`] — flow definition storage extension point.
//!
//! Implement [`FlowStore`] to store and retrieve named JSON flow definitions
//! using any backend (in-memory, SQLite, PostgreSQL, remote API, …).
//!
//! Register a store on [`FlowEngine`](crate::engine::FlowEngine) via
//! [`with_flow_store`](crate::engine::FlowEngine::with_flow_store) to enable
//! [`start_named`](crate::engine::FlowEngine::start_named) — which loads a
//! definition by name and starts it in one step.
//!
//! [`MemoryFlowStore`] is provided as an in-process default.

use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;
use serde_json::Value;
use tokio::sync::RwLock;

use crate::error::Result;

/// Named storage for JSON flow definitions.
///
/// # Example
///
/// ```rust
/// use a3s_flow::{FlowStore, MemoryFlowStore};
/// use serde_json::json;
/// use std::sync::Arc;
///
/// # #[tokio::main] async fn main() {
/// let store = MemoryFlowStore::new();
///
/// let def = json!({ "nodes": [{ "id": "a", "type": "noop" }], "edges": [] });
/// store.save("my-flow", &def).await.unwrap();
///
/// let loaded = store.load("my-flow").await.unwrap().unwrap();
/// assert_eq!(loaded, def);
/// # }
/// ```
#[async_trait]
pub trait FlowStore: Send + Sync {
    /// Persist a flow definition under the given name (upsert semantics).
    async fn save(&self, name: &str, definition: &Value) -> Result<()>;

    /// Load a flow definition by name.
    ///
    /// Returns `None` if no definition exists with that name.
    async fn load(&self, name: &str) -> Result<Option<Value>>;

    /// List all stored flow names.
    async fn list(&self) -> Result<Vec<String>>;

    /// Delete a stored flow definition. No-op if the name is not found.
    async fn delete(&self, name: &str) -> Result<()>;
}

/// An in-memory [`FlowStore`] backed by a `HashMap` under an `RwLock`.
///
/// Suitable for testing and short-lived processes. Data is lost on restart.
pub struct MemoryFlowStore {
    inner: Arc<RwLock<HashMap<String, Value>>>,
}

impl MemoryFlowStore {
    /// Create a new empty store.
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl Default for MemoryFlowStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl FlowStore for MemoryFlowStore {
    async fn save(&self, name: &str, definition: &Value) -> Result<()> {
        self.inner
            .write()
            .await
            .insert(name.to_string(), definition.clone());
        Ok(())
    }

    async fn load(&self, name: &str) -> Result<Option<Value>> {
        Ok(self.inner.read().await.get(name).cloned())
    }

    async fn list(&self) -> Result<Vec<String>> {
        Ok(self.inner.read().await.keys().cloned().collect())
    }

    async fn delete(&self, name: &str) -> Result<()> {
        self.inner.write().await.remove(name);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[tokio::test]
    async fn save_and_load_round_trip() {
        let store = MemoryFlowStore::new();
        let def = json!({ "nodes": [{ "id": "a", "type": "noop" }], "edges": [] });

        store.save("my-flow", &def).await.unwrap();
        let loaded = store.load("my-flow").await.unwrap().unwrap();
        assert_eq!(loaded, def);
    }

    #[tokio::test]
    async fn load_unknown_name_returns_none() {
        let store = MemoryFlowStore::new();
        let result = store.load("nonexistent").await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn list_returns_all_saved_names() {
        let store = MemoryFlowStore::new();
        let def = json!({ "nodes": [{ "id": "a", "type": "noop" }], "edges": [] });

        store.save("flow-a", &def).await.unwrap();
        store.save("flow-b", &def).await.unwrap();

        let mut names = store.list().await.unwrap();
        names.sort();
        assert_eq!(names, vec!["flow-a", "flow-b"]);
    }

    #[tokio::test]
    async fn delete_removes_entry() {
        let store = MemoryFlowStore::new();
        let def = json!({ "nodes": [{ "id": "a", "type": "noop" }], "edges": [] });

        store.save("flow-a", &def).await.unwrap();
        store.delete("flow-a").await.unwrap();

        assert!(store.load("flow-a").await.unwrap().is_none());
        assert!(store.list().await.unwrap().is_empty());
    }

    #[tokio::test]
    async fn save_overwrites_existing_definition() {
        let store = MemoryFlowStore::new();
        let v1 = json!({ "nodes": [{ "id": "a", "type": "noop" }], "edges": [] });
        let v2 = json!({ "nodes": [{ "id": "b", "type": "noop" }], "edges": [] });

        store.save("flow", &v1).await.unwrap();
        store.save("flow", &v2).await.unwrap();

        let loaded = store.load("flow").await.unwrap().unwrap();
        assert_eq!(loaded, v2);
        assert_eq!(store.list().await.unwrap().len(), 1);
    }
}
