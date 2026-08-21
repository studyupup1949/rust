use std::sync::Arc;

use crate::{
    common::{MemCache, Shutdown, Vars},
    events::{Event, Log},
    runtime::{Channel, ProcessId},
    utils,
    workflow::node::NodeId,
};

#[derive(Clone)]
pub struct Context {
    pid: ProcessId,
    env: Arc<MemCache<String, String>>,
    outputs: Arc<MemCache<NodeId, Vars>>,
    channel: Arc<Channel>,

    shutdown: Arc<Shutdown>,
}

impl Context {
    pub fn new(
        pid: ProcessId,
        channel: Arc<Channel>,
    ) -> Self {
        Self {
            pid,
            env: Arc::new(MemCache::new(1024)),
            outputs: Arc::new(MemCache::new(1024)),
            channel,
            shutdown: Arc::new(Shutdown::new()),
        }
    }

    pub fn env(&self) -> Arc<MemCache<String, String>> {
        self.env.clone()
    }

    pub fn outputs(&self) -> Arc<MemCache<NodeId, Vars>> {
        self.outputs.clone()
    }

    pub fn add_output(
        &self,
        nid: NodeId,
        outputs: Vars,
    ) {
        self.outputs.set(nid, outputs);
    }

    pub fn channel(&self) -> Arc<Channel> {
        self.channel.clone()
    }

    pub fn emit_log(
        &self,
        nid: NodeId,
        content: String,
    ) {
        let log = Log {
            pid: self.pid.clone(),
            nid,
            content,
            timestamp: utils::time::time_millis(),
        };
        let _ = self.channel.log_queue().send(Event::new(&log));
    }

    pub fn pid(&self) -> ProcessId {
        self.pid.to_owned()
    }

    pub fn done(&self) {
        self.shutdown.shutdown();
    }

    pub fn wait_shutdown(&self) -> impl Future<Output = ()> + Send + 'static {
        self.shutdown.wait()
    }
}
