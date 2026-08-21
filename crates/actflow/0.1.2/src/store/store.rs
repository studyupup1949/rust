//! Central store for managing database collections.

use std::{
    any::Any,
    collections::HashMap,
    convert::AsRef,
    sync::{Arc, RwLock},
};

use tracing::trace;

use crate::{ActflowError, Result, ShareLock, model::WorkflowModel, utils};

use super::{DbCollection, DbCollectionIden, StoreIden, data::*};

/// Type-erased reference to a database collection.
#[derive(Clone)]
pub struct DynDbSetRef<T>(Arc<dyn DbCollection<Item = T>>);

/// Central store managing all database collections.
///
/// The store provides a unified interface for accessing different
/// collections (workflows, processes, nodes, events, logs) regardless
/// of the underlying storage backend.
pub struct Store {
    /// Map of collection identifiers to type-erased collection references.
    collections: ShareLock<HashMap<StoreIden, Arc<dyn Any + Send + Sync + 'static>>>,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    /// Creates a new empty store.
    pub fn new() -> Self {
        Self {
            collections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Gets a typed collection by its data type.
    pub fn collection<DATA>(&self) -> Arc<dyn DbCollection<Item = DATA>>
    where
        DATA: DbCollectionIden + Send + Sync + 'static,
    {
        let collections = self.collections.read().unwrap();

        #[allow(clippy::expect_fun_call)]
        let collection = collections.get(&DATA::iden()).expect(&format!("fail to get collection: {}", DATA::iden().as_ref()));

        #[allow(clippy::expect_fun_call)]
        collection.downcast_ref::<DynDbSetRef<DATA>>().map(|v| v.0.clone()).expect(&format!("fail to get collection: {}", DATA::iden().as_ref()))
    }

    /// Registers a collection with the store.
    pub fn register<DATA>(
        &self,
        collection: Arc<dyn DbCollection<Item = DATA> + Send + Sync + 'static>,
    ) where
        DATA: DbCollectionIden + 'static,
    {
        let mut collections = self.collections.write().unwrap();
        collections.insert(DATA::iden(), Arc::new(DynDbSetRef::<DATA>(collection)));
    }

    /// Returns the workflows collection.
    pub fn workflows(&self) -> Arc<dyn DbCollection<Item = Workflow>> {
        self.collection()
    }

    /// Returns the processes collection.
    pub fn procs(&self) -> Arc<dyn DbCollection<Item = Proc>> {
        self.collection()
    }

    /// Returns the nodes collection.
    pub fn nodes(&self) -> Arc<dyn DbCollection<Item = Node>> {
        self.collection()
    }

    /// Returns the logs collection.
    pub fn logs(&self) -> Arc<dyn DbCollection<Item = Log>> {
        self.collection()
    }

    /// Returns the events collection.
    pub fn events(&self) -> Arc<dyn DbCollection<Item = Event>> {
        self.collection()
    }

    /// Deploys a workflow definition to the store.
    ///
    /// If the workflow already exists, it will be updated.
    /// Otherwise, a new workflow will be created.
    pub fn deploy(
        &self,
        workflow: &WorkflowModel,
    ) -> Result<bool> {
        trace!("store::deploy({})", workflow.id);
        if workflow.id.is_empty() {
            return Err(ActflowError::Workflow("missing id in workflow".into()));
        }
        let workflows = self.workflows();
        match workflows.find(&workflow.id) {
            Ok(m) => {
                let text = serde_json::to_string(workflow).unwrap();
                let data = Workflow {
                    id: workflow.id.clone(),
                    name: workflow.name.clone(),
                    desc: workflow.desc.clone(),
                    data: text.clone(),
                    create_time: m.create_time,
                    update_time: utils::time::time_millis(),
                };
                workflows.update(&data)
            }
            Err(_) => {
                let text = serde_json::to_string(workflow).unwrap();
                let data = Workflow {
                    id: workflow.id.clone(),
                    name: workflow.name.clone(),
                    desc: workflow.desc.clone(),
                    data: text.clone(),
                    create_time: utils::time::time_millis(),
                    update_time: 0,
                };
                workflows.create(&data)
            }
        }
    }
}
