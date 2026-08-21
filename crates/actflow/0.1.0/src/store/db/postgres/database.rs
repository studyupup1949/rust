use std::sync::Arc;

use tokio::runtime::Runtime;

use crate::store::{DbCollection, DbStore, Store, data::*};

use super::{DbInit, collection::*, synclient::SynClient};

pub struct PostgresStore {
    workflows: Arc<WorkflowCollection>,
    procs: Arc<ProcCollection>,
    nodes: Arc<NodeCollection>,
    logs: Arc<LogCollection>,
    events: Arc<EventCollection>,
}

impl DbStore for PostgresStore {
    fn init(
        &self,
        s: &Store,
    ) {
        self.workflows.init();
        self.procs.init();
        self.nodes.init();
        self.logs.init();
        self.events.init();

        s.register(self.workflows());
        s.register(self.procs());
        s.register(self.nodes());
        s.register(self.logs());
        s.register(self.events());
    }
}

impl PostgresStore {
    pub fn new(
        db_url: &str,
        runtime: Arc<Runtime>,
    ) -> Self {
        let conn = Arc::new(SynClient::connect(db_url, runtime.clone()));
        let workflows = WorkflowCollection::new(&conn);
        let procs = ProcCollection::new(&conn);
        let nodes = NodeCollection::new(&conn);
        let logs = LogCollection::new(&conn, runtime.clone());
        let events = EventCollection::new(&conn, runtime.clone());

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
