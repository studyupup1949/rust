mod collect;
mod r#impl;

use std::{collections::HashMap, sync::Arc};

use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value as JsonValue;

use crate::{
    Result,
    store::{DbCollection, DbStore, Store, data::*},
};
pub use collect::Collect;

#[derive(Debug, Clone)]
pub struct MemStore {
    workflows: Arc<Collect<Workflow>>,
    procs: Arc<Collect<Proc>>,
    nodes: Arc<Collect<Node>>,
    logs: Arc<Collect<Log>>,
    events: Arc<Collect<Event>>,
}

trait DbDocument: Serialize + DeserializeOwned {
    fn id(&self) -> &str;
    fn doc(&self) -> Result<HashMap<String, JsonValue>>;
}

impl DbStore for MemStore {
    fn init(
        &self,
        s: &Store,
    ) {
        s.register(self.workflows());
        s.register(self.procs());
        s.register(self.nodes());
        s.register(self.logs());
        s.register(self.events());
    }
}

impl MemStore {
    pub fn new() -> Self {
        let workflows = Collect::new("workflows");
        let procs = Collect::new("procs");
        let nodes = Collect::new("nodes");
        let logs = Collect::new("logs");
        let events = Collect::new("events");

        Self {
            workflows: Arc::new(workflows),
            procs: Arc::new(procs),
            nodes: Arc::new(nodes),
            logs: Arc::new(logs),
            events: Arc::new(events),
        }
    }

    pub fn workflows(&self) -> Arc<dyn DbCollection<Item = Workflow> + Send + Sync> {
        self.workflows.clone()
    }

    pub fn procs(&self) -> Arc<dyn DbCollection<Item = Proc> + Send + Sync> {
        self.procs.clone()
    }

    pub fn nodes(&self) -> Arc<dyn DbCollection<Item = Node> + Send + Sync> {
        self.nodes.clone()
    }

    pub fn logs(&self) -> Arc<dyn DbCollection<Item = Log> + Send + Sync> {
        self.logs.clone()
    }

    pub fn events(&self) -> Arc<dyn DbCollection<Item = Event> + Send + Sync> {
        self.events.clone()
    }
}
