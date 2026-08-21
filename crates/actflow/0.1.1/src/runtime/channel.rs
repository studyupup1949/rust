use std::sync::{Arc, RwLock};

use futures::future::BoxFuture;
use tokio::runtime::Runtime;

use crate::{
    ShareLock,
    common::{BroadcastQueue, Shutdown},
    events::{Event, Log, Message},
    runtime::ProcessId,
};

macro_rules! dispatch_event {
    ($handles:expr, $(&$item:ident), +) => {
        let handlers = $handles.read().unwrap();
        for handle in handlers.iter() {
            (handle)($(&$item),+);
        }
    };
}

macro_rules! dispatch_event_async {
    ($handles:expr, $(&$item:ident), +) => {
        let handles = $handles.clone();

        tokio::spawn(async move {
            let handlers = handles.read().unwrap().clone();
            for handle in handlers.iter() {
                (handle)($(&$item),+).await;
            }
        });
    };
}

const EVENT_QUEUE_SIZE: usize = 2048;
const LOG_QUEUE_SIZE: usize = 4096;

pub type WorkflowEventHandle = Arc<dyn Fn(&Event<Message>) + Send + Sync>;
pub type WorkflowLogHandle = Arc<dyn Fn(&Event<Log>) + Send + Sync>;
pub type WorkflowEventHandleAsync = Arc<dyn Fn(&Event<Message>) -> BoxFuture<'static, ()> + Send + Sync>;
pub type WorkflowLogHandleAsync = Arc<dyn Fn(&Event<Log>) -> BoxFuture<'static, ()> + Send + Sync>;

#[derive(Debug, Clone)]
pub struct ChannelOptions {
    /// use the glob pattern to match the process id
    /// eg. pid1*
    pub pid: String,

    /// use the glob pattern to match the node id
    /// eg. nid1*
    pub nid: String,
}

impl Default for ChannelOptions {
    fn default() -> Self {
        Self {
            pid: "*".to_string(),
            nid: "*".to_string(),
        }
    }
}

#[allow(unused)]
impl ChannelOptions {
    pub fn new(
        pid: String,
        nid: String,
    ) -> Self {
        Self {
            pid,
            nid,
        }
    }

    pub fn with_pid(pid: String) -> Self {
        Self {
            pid,
            nid: "*".to_string(),
        }
    }

    pub fn with_nid(nid: String) -> Self {
        Self {
            pid: "*".to_string(),
            nid,
        }
    }
}

#[derive(Clone)]
pub struct Channel {
    event_queue: Arc<BroadcastQueue<Event<Message>>>,
    log_queue: Arc<BroadcastQueue<Event<Log>>>,

    events: ShareLock<Vec<WorkflowEventHandle>>,
    logs: ShareLock<Vec<WorkflowLogHandle>>,
    events_async: ShareLock<Vec<WorkflowEventHandleAsync>>,
    logs_async: ShareLock<Vec<WorkflowLogHandleAsync>>,

    runtime: Arc<Runtime>,
    shutdown: Arc<Shutdown>,
}

impl Channel {
    pub(crate) fn new(runtime: Arc<Runtime>) -> Self {
        Self {
            event_queue: BroadcastQueue::new(EVENT_QUEUE_SIZE),
            log_queue: BroadcastQueue::new(LOG_QUEUE_SIZE),
            events: Arc::new(RwLock::new(Vec::new())),
            logs: Arc::new(RwLock::new(Vec::new())),
            events_async: Arc::new(RwLock::new(Vec::new())),
            logs_async: Arc::new(RwLock::new(Vec::new())),
            runtime,
            shutdown: Arc::new(Shutdown::new()),
        }
    }

    pub(crate) fn log_queue(&self) -> Arc<BroadcastQueue<Event<Log>>> {
        self.log_queue.clone()
    }

    pub(crate) fn event_queue(&self) -> Arc<BroadcastQueue<Event<Message>>> {
        self.event_queue.clone()
    }

    pub(crate) fn listen(&self) {
        let mut event_queue = self.event_queue.subscribe();
        let mut log_queue = self.log_queue.subscribe();
        let events = self.events.clone();
        let logs = self.logs.clone();
        let events_async = self.events_async.clone();
        let logs_async = self.logs_async.clone();

        let shutdown = self.shutdown.clone();
        self.runtime.spawn(async move {
            loop {
                tokio::select! {
                    _ = shutdown.wait() => break,
                    Ok(e) = event_queue.recv() => {
                        let evt = e.clone();
                        dispatch_event!(events, &evt);
                        dispatch_event_async!(events_async, &e);
                    }
                    Ok(log) = log_queue.recv() => {
                        let l = log.clone();
                        dispatch_event!(logs, &l);
                        dispatch_event_async!(logs_async, &log);
                    }
                }
            }
        });
    }

    pub(crate) fn shutdown(&self) {
        self.shutdown.shutdown();
    }
}

#[derive(Clone)]
pub struct ChannelEvent {
    channel: Arc<Channel>,

    glob: (globset::GlobMatcher, globset::GlobMatcher),
}

#[allow(unused)]
impl ChannelEvent {
    pub fn channel(
        channel: Arc<Channel>,
        options: ChannelOptions,
    ) -> Self {
        Self {
            channel,
            glob: (
                globset::Glob::new(&options.pid).unwrap().compile_matcher(),
                globset::Glob::new(&options.nid).unwrap().compile_matcher(),
            ),
        }
    }

    pub fn on_complete(
        &self,
        f: impl Fn(ProcessId) + Send + Sync + 'static,
    ) {
        let glob = self.glob.clone();

        self.channel.events.write().unwrap().push(Arc::new(move |e| {
            if e.event.is_complete() && is_match(&glob, e) {
                f(e.pid.clone());
            }
        }));
    }

    pub fn on_error(
        &self,
        f: impl Fn(&Event<Message>) + Send + Sync + 'static,
    ) {
        let glob = self.glob.clone();

        self.channel.events.write().unwrap().push(Arc::new(move |e| {
            if e.event.is_error() && is_match(&glob, e) {
                f(e);
            }
        }));
    }

    pub fn on_event(
        &self,
        f: impl Fn(&Event<Message>) + Send + Sync + 'static,
    ) {
        let glob = self.glob.clone();

        self.channel.events.write().unwrap().push(Arc::new(move |e| {
            if is_match(&glob, e) {
                f(e);
            }
        }));
    }

    pub fn on_log(
        &self,
        f: impl Fn(&Event<Log>) + Send + Sync + 'static,
    ) {
        let glob = self.glob.clone();

        self.channel.logs.write().unwrap().push(Arc::new(move |e| {
            if is_match_log(&glob, e) {
                f(e);
            }
        }));
    }

    pub fn on_event_async<F>(
        &self,
        f: F,
    ) where
        F: Fn(&Event<Message>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let glob = self.glob.clone();

        self.channel.events_async.write().unwrap().push(Arc::new(move |e| {
            if is_match(&glob, e) {
                f(e)
            } else {
                Box::pin(async {})
            }
        }));
    }

    pub fn on_log_async<F>(
        &self,
        f: F,
    ) where
        F: Fn(&Event<Log>) -> BoxFuture<'static, ()> + Send + Sync + 'static,
    {
        let glob = self.glob.clone();

        self.channel.logs_async.write().unwrap().push(Arc::new(move |e| {
            if is_match_log(&glob, e) {
                f(e)
            } else {
                Box::pin(async {})
            }
        }));
    }
}

fn is_match(
    glob: &(globset::GlobMatcher, globset::GlobMatcher),
    e: &Event<Message>,
) -> bool {
    let (pat_pid, pat_nid) = glob;
    pat_pid.is_match(&e.pid) && pat_nid.is_match(&e.nid)
}

fn is_match_log(
    glob: &(globset::GlobMatcher, globset::GlobMatcher),
    e: &Event<Log>,
) -> bool {
    let (pat_pid, pat_nid) = glob;
    pat_pid.is_match(&e.pid) && pat_nid.is_match(&e.nid)
}
