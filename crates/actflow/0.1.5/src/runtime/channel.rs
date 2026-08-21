//! Event channel for broadcasting workflow events and logs.
//!
//! The channel provides a pub/sub mechanism for workflow events, supporting
//! both synchronous and asynchronous event handlers with glob-based filtering.

use std::sync::{Arc, RwLock};

use futures::future::BoxFuture;
use tokio::runtime::Runtime;

use crate::{
    ShareLock,
    common::{BroadcastQueue, Shutdown},
    events::{Event, Log, Message},
    runtime::ProcessId,
};

/// Dispatches events to all registered synchronous handlers.
macro_rules! dispatch_event {
    ($handles:expr, $(&$item:ident), +) => {
        let handlers = $handles.read().unwrap();
        for handle in handlers.iter() {
            (handle)($(&$item),+);
        }
    };
}

/// Dispatches events to all registered asynchronous handlers.
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

/// Maximum number of events in the event queue.
const EVENT_QUEUE_SIZE: usize = 2048;
/// Maximum number of logs in the log queue.
const LOG_QUEUE_SIZE: usize = 4096;

/// Synchronous event handler type.
pub type WorkflowEventHandle = Arc<dyn Fn(&Event<Message>) + Send + Sync>;
/// Synchronous log handler type.
pub type WorkflowLogHandle = Arc<dyn Fn(&Event<Log>) + Send + Sync>;
/// Asynchronous event handler type.
pub type WorkflowEventHandleAsync = Arc<dyn Fn(&Event<Message>) -> BoxFuture<'static, ()> + Send + Sync>;
/// Asynchronous log handler type.
pub type WorkflowLogHandleAsync = Arc<dyn Fn(&Event<Log>) -> BoxFuture<'static, ()> + Send + Sync>;

/// Options for filtering events by process ID and node ID.
///
/// Supports glob patterns for flexible matching:
/// - `*` matches any string
/// - `pid1*` matches process IDs starting with "pid1"
///
/// # Example
///
/// ```rust
/// use actflow::ChannelOptions;
///
/// // Match specific process
/// let opts = ChannelOptions::with_pid("process123".to_string());
///
/// // Match all events from any process
/// let opts = ChannelOptions::default();
/// ```
#[derive(Debug, Clone)]
pub struct ChannelOptions {
    /// Glob pattern to match process IDs (e.g., "pid1*", "*").
    pub pid: String,
    /// Glob pattern to match node IDs (e.g., "nid1*", "*").
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
    /// Creates new options with specific process and node ID patterns.
    pub fn new(
        pid: String,
        nid: String,
    ) -> Self {
        Self {
            pid,
            nid,
        }
    }

    /// Creates options filtering by process ID only.
    pub fn with_pid(pid: String) -> Self {
        Self {
            pid,
            nid: "*".to_string(),
        }
    }

    /// Creates options filtering by node ID only.
    pub fn with_nid(nid: String) -> Self {
        Self {
            pid: "*".to_string(),
            nid,
        }
    }
}

/// Central event bus for broadcasting workflow events and logs.
///
/// The channel uses broadcast queues to distribute events to all registered
/// handlers. It supports both synchronous and asynchronous handlers.
#[derive(Clone)]
pub struct Channel {
    /// Queue for workflow/node events.
    event_queue: Arc<BroadcastQueue<Event<Message>>>,
    /// Queue for log messages.
    log_queue: Arc<BroadcastQueue<Event<Log>>>,
    /// Registered synchronous event handlers.
    events: ShareLock<Vec<WorkflowEventHandle>>,
    /// Registered synchronous log handlers.
    logs: ShareLock<Vec<WorkflowLogHandle>>,
    /// Registered asynchronous event handlers.
    events_async: ShareLock<Vec<WorkflowEventHandleAsync>>,
    /// Registered asynchronous log handlers.
    logs_async: ShareLock<Vec<WorkflowLogHandleAsync>>,
    /// Tokio runtime for spawning async tasks.
    runtime: Arc<Runtime>,
    /// Shutdown coordinator.
    shutdown: Arc<Shutdown>,
}

impl Channel {
    /// Creates a new event channel.
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

    /// Returns the log queue for sending log events.
    pub(crate) fn log_queue(&self) -> Arc<BroadcastQueue<Event<Log>>> {
        self.log_queue.clone()
    }

    /// Returns the event queue for sending workflow events.
    pub(crate) fn event_queue(&self) -> Arc<BroadcastQueue<Event<Message>>> {
        self.event_queue.clone()
    }

    /// Starts listening for events and dispatching to handlers.
    ///
    /// This spawns an async task that listens to both event and log queues,
    /// dispatching to registered handlers until shutdown is signaled.
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

    /// Signals the channel to stop listening.
    pub(crate) fn shutdown(&self) {
        self.shutdown.shutdown();
    }
}

/// Builder for registering event handlers with filtering.
///
/// Use this to subscribe to specific events from the channel:
///
/// ```rust,ignore
/// ChannelEvent::channel(channel, ChannelOptions::with_pid(pid))
///     .on_complete(|pid| println!("Process {} completed", pid))
///     .on_error(|e| println!("Error: {:?}", e))
///     .on_log(|log| println!("Log: {}", log.content));
/// ```
#[derive(Clone)]
pub struct ChannelEvent {
    /// Reference to the underlying channel.
    channel: Arc<Channel>,
    /// Compiled glob matchers for (pid, nid) filtering.
    glob: (globset::GlobMatcher, globset::GlobMatcher),
}

#[allow(unused)]
impl ChannelEvent {
    /// Creates a new event subscriber with the given options.
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

    /// Registers a handler for workflow completion events.
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

    /// Registers a handler for workflow error events.
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

    /// Registers a handler for all matching events.
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

    /// Registers a handler for log events.
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

    /// Registers an async handler for all matching events.
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

    /// Registers an async handler for log events.
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

/// Checks if an event matches the glob patterns.
fn is_match(
    glob: &(globset::GlobMatcher, globset::GlobMatcher),
    e: &Event<Message>,
) -> bool {
    let (pat_pid, pat_nid) = glob;
    pat_pid.is_match(&e.pid) && pat_nid.is_match(&e.nid)
}

/// Checks if a log event matches the glob patterns.
fn is_match_log(
    glob: &(globset::GlobMatcher, globset::GlobMatcher),
    e: &Event<Log>,
) -> bool {
    let (pat_pid, pat_nid) = glob;
    pat_pid.is_match(&e.pid) && pat_nid.is_match(&e.nid)
}
