use std::{
    any::Any,
    collections::HashMap,
    convert::AsRef,
    sync::{Arc, RwLock},
};

use tracing::trace;

use crate::{ActflowError, Result, ShareLock, model::WorkflowModel, utils};

use super::{DbCollection, DbCollectionIden, StoreIden, data::*};

#[derive(Clone)]
pub struct DynDbSetRef<T>(Arc<dyn DbCollection<Item = T>>);

pub struct Store {
    collections: ShareLock<HashMap<StoreIden, Arc<dyn Any + Send + Sync + 'static>>>,
}

impl Default for Store {
    fn default() -> Self {
        Self::new()
    }
}

impl Store {
    pub fn new() -> Self {
        Self {
            collections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

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

    pub fn register<DATA>(
        &self,
        collection: Arc<dyn DbCollection<Item = DATA> + Send + Sync + 'static>,
    ) where
        DATA: DbCollectionIden + 'static,
    {
        let mut collections = self.collections.write().unwrap();
        collections.insert(DATA::iden(), Arc::new(DynDbSetRef::<DATA>(collection)));
    }

    pub fn workflows(&self) -> Arc<dyn DbCollection<Item = Workflow>> {
        self.collection()
    }

    pub fn procs(&self) -> Arc<dyn DbCollection<Item = Proc>> {
        self.collection()
    }

    pub fn nodes(&self) -> Arc<dyn DbCollection<Item = Node>> {
        self.collection()
    }

    pub fn logs(&self) -> Arc<dyn DbCollection<Item = Log>> {
        self.collection()
    }

    pub fn events(&self) -> Arc<dyn DbCollection<Item = Event>> {
        self.collection()
    }

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
