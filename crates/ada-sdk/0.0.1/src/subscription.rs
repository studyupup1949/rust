use std::collections::{BTreeMap, HashMap};
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use tokio_util::sync::CancellationToken;
use tonic::Code;

use crate::AdaError;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StreamKind {
    Events,
    Signals,
    Jobs,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectingInfo {
    pub stream: StreamKind,
    pub attempt: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConnectedInfo {
    pub stream: StreamKind,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ReconnectInfo {
    pub stream: StreamKind,
    pub attempt: usize,
    pub delay: Duration,
    pub error: AdaError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DisconnectedInfo {
    pub stream: StreamKind,
    pub error: AdaError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CursorInfo {
    pub stream: StreamKind,
    pub event_id: String,
    pub principal_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ProtocolErrorInfo {
    pub stream: StreamKind,
    pub error: AdaError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ListenerErrorInfo {
    pub stream: StreamKind,
    pub event: String,
    pub principal_id: String,
    pub error: AdaError,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TerminalInfo {
    pub stream: StreamKind,
    pub error: AdaError,
}

#[derive(Clone, Debug)]
pub struct SubscriptionOptions {
    pub after_event_id: String,
    pub replay_limit: i32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub max_reconnect_attempts: Option<usize>,
}

impl Default for SubscriptionOptions {
    fn default() -> Self {
        Self {
            after_event_id: String::new(),
            replay_limit: 0,
            initial_delay: Duration::from_millis(250),
            max_delay: Duration::from_secs(10),
            max_reconnect_attempts: None,
        }
    }
}

impl SubscriptionOptions {
    pub(crate) fn delay(&self, attempt: usize) -> Duration {
        let exponent = attempt.saturating_sub(1).min(31) as u32;
        self.initial_delay
            .saturating_mul(2_u32.saturating_pow(exponent))
            .min(self.max_delay)
    }

    pub(crate) fn exhausted(&self, attempt: usize) -> bool {
        self.max_reconnect_attempts
            .is_some_and(|maximum| attempt > maximum)
    }
}

#[derive(Clone, Debug, Default)]
pub struct StreamConfig {
    pub events: SubscriptionOptions,
    pub signals: SubscriptionOptions,
    pub jobs: SubscriptionOptions,
}

#[derive(Clone)]
pub struct Unsubscribe {
    action: Arc<dyn Fn() + Send + Sync>,
}

impl Unsubscribe {
    pub(crate) fn new(action: impl Fn() + Send + Sync + 'static) -> Self {
        Self {
            action: Arc::new(action),
        }
    }

    pub fn unsubscribe(&self) {
        (self.action)();
    }
}

struct Listener<T> {
    active: AtomicBool,
    handler: Arc<dyn Fn(T) + Send + Sync>,
}

pub(crate) struct ListenerSet<T> {
    next_id: AtomicU64,
    listeners: Arc<RwLock<BTreeMap<u64, Arc<Listener<T>>>>>,
}

impl<T> Default for ListenerSet<T> {
    fn default() -> Self {
        Self {
            next_id: AtomicU64::new(0),
            listeners: Arc::new(RwLock::new(BTreeMap::new())),
        }
    }
}

impl<T> ListenerSet<T>
where
    T: Clone + Send + 'static,
{
    pub(crate) fn on(&self, handler: impl Fn(T) + Send + Sync + 'static) -> Unsubscribe {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let listener = Arc::new(Listener {
            active: AtomicBool::new(true),
            handler: Arc::new(handler),
        });
        self.listeners
            .write()
            .expect("listener write lock poisoned")
            .insert(id, listener.clone());
        let listeners = self.listeners.clone();
        Unsubscribe::new(move || {
            if listener.active.swap(false, Ordering::AcqRel) {
                listeners
                    .write()
                    .expect("listener write lock poisoned")
                    .remove(&id);
            }
        })
    }

    pub(crate) fn emit(&self, value: T) {
        self.emit_with(value, |_| {});
    }

    pub(crate) fn emit_with(&self, value: T, on_panic: impl Fn(AdaError)) {
        let listeners = self
            .listeners
            .read()
            .expect("listener read lock poisoned")
            .values()
            .cloned()
            .collect::<Vec<_>>();
        for listener in listeners {
            if !listener.active.load(Ordering::Acquire) {
                continue;
            }
            let result = catch_unwind(AssertUnwindSafe(|| {
                (listener.handler)(value.clone());
            }));
            if result.is_err() {
                on_panic(AdaError {
                    code: Code::Internal,
                    message: "listener panicked".to_owned(),
                });
            }
        }
    }
}

pub(crate) struct PrincipalListenerSet<T> {
    listeners: RwLock<HashMap<String, Arc<ListenerSet<T>>>>,
}

impl<T> Default for PrincipalListenerSet<T> {
    fn default() -> Self {
        Self {
            listeners: RwLock::new(HashMap::new()),
        }
    }
}

impl<T> PrincipalListenerSet<T>
where
    T: Clone + Send + 'static,
{
    pub(crate) fn on(
        &self,
        principal_id: &str,
        handler: impl Fn(T) + Send + Sync + 'static,
    ) -> Unsubscribe {
        let listeners = {
            let mut all = self
                .listeners
                .write()
                .expect("principal listener write lock poisoned");
            all.entry(principal_id.to_owned())
                .or_insert_with(|| Arc::new(ListenerSet::default()))
                .clone()
        };
        listeners.on(handler)
    }

    pub(crate) fn emit_with(&self, principal_id: &str, value: T, on_panic: impl Fn(AdaError)) {
        let listeners = self
            .listeners
            .read()
            .expect("principal listener read lock poisoned")
            .get(principal_id)
            .cloned();
        if let Some(listeners) = listeners {
            listeners.emit_with(value, on_panic);
        }
    }
}

#[derive(Default)]
pub struct Lifecycle {
    connecting: ListenerSet<ConnectingInfo>,
    connected: ListenerSet<ConnectedInfo>,
    reconnecting: ListenerSet<ReconnectInfo>,
    disconnected: ListenerSet<DisconnectedInfo>,
    cursor: ListenerSet<CursorInfo>,
    protocol_error: ListenerSet<ProtocolErrorInfo>,
    listener_error: ListenerSet<ListenerErrorInfo>,
    terminal: ListenerSet<TerminalInfo>,
    closing: ListenerSet<()>,
    closed: ListenerSet<()>,
}

impl Lifecycle {
    pub fn on_connecting(
        &self,
        handler: impl Fn(ConnectingInfo) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.connecting.on(handler)
    }

    pub fn on_connected(
        &self,
        handler: impl Fn(ConnectedInfo) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.connected.on(handler)
    }

    pub fn on_reconnecting(
        &self,
        handler: impl Fn(ReconnectInfo) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.reconnecting.on(handler)
    }

    pub fn on_disconnected(
        &self,
        handler: impl Fn(DisconnectedInfo) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.disconnected.on(handler)
    }

    pub fn on_cursor(&self, handler: impl Fn(CursorInfo) + Send + Sync + 'static) -> Unsubscribe {
        self.cursor.on(handler)
    }

    pub fn on_protocol_error(
        &self,
        handler: impl Fn(ProtocolErrorInfo) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.protocol_error.on(handler)
    }

    pub fn on_listener_error(
        &self,
        handler: impl Fn(ListenerErrorInfo) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.listener_error.on(handler)
    }

    pub fn on_terminal(
        &self,
        handler: impl Fn(TerminalInfo) + Send + Sync + 'static,
    ) -> Unsubscribe {
        self.terminal.on(handler)
    }

    pub fn on_closing(&self, handler: impl Fn() + Send + Sync + 'static) -> Unsubscribe {
        self.closing.on(move |()| handler())
    }

    pub fn on_closed(&self, handler: impl Fn() + Send + Sync + 'static) -> Unsubscribe {
        self.closed.on(move |()| handler())
    }

    pub(crate) fn connecting(&self, stream: StreamKind, attempt: usize) {
        self.connecting.emit(ConnectingInfo { stream, attempt });
    }

    pub(crate) fn connected(&self, stream: StreamKind) {
        self.connected.emit(ConnectedInfo { stream });
    }

    pub(crate) fn disconnected(&self, stream: StreamKind, error: AdaError) {
        self.disconnected.emit(DisconnectedInfo { stream, error });
    }

    pub(crate) fn cursor(&self, stream: StreamKind, event_id: String, principal_id: String) {
        if !event_id.is_empty() {
            self.cursor.emit(CursorInfo {
                stream,
                event_id,
                principal_id,
            });
        }
    }

    pub(crate) fn protocol_error(&self, stream: StreamKind, message: &str) {
        self.protocol_error.emit(ProtocolErrorInfo {
            stream,
            error: AdaError {
                code: Code::Internal,
                message: message.to_owned(),
            },
        });
    }

    pub(crate) fn listener_error(
        &self,
        stream: StreamKind,
        event: &str,
        principal_id: &str,
        error: AdaError,
    ) {
        self.listener_error.emit(ListenerErrorInfo {
            stream,
            event: event.to_owned(),
            principal_id: principal_id.to_owned(),
            error,
        });
    }

    pub(crate) fn terminal(&self, stream: StreamKind, error: AdaError) {
        self.terminal.emit(TerminalInfo { stream, error });
    }

    pub(crate) fn closing(&self) {
        self.closing.emit(());
    }

    pub(crate) fn closed(&self) {
        self.closed.emit(());
    }

    pub(crate) fn reconnecting(
        &self,
        stream: StreamKind,
        attempt: usize,
        delay: Duration,
        error: AdaError,
    ) {
        self.reconnecting.emit(ReconnectInfo {
            stream,
            attempt,
            delay,
            error,
        });
    }
}

pub(crate) struct HubControl {
    pub cancellation: CancellationToken,
    pub started: AtomicBool,
    pub options: SubscriptionOptions,
    pub lifecycle: Arc<Lifecycle>,
    pub kind: StreamKind,
}

impl HubControl {
    pub(crate) fn new(
        cancellation: CancellationToken,
        lifecycle: Arc<Lifecycle>,
        kind: StreamKind,
        options: SubscriptionOptions,
    ) -> Self {
        Self {
            cancellation,
            started: AtomicBool::new(false),
            options,
            lifecycle,
            kind,
        }
    }

    pub(crate) async fn failure(&self, attempt: usize, error: AdaError) -> bool {
        self.lifecycle.disconnected(self.kind, error.clone());
        if !crate::errors::retryable(&error) || self.options.exhausted(attempt) {
            self.lifecycle.terminal(self.kind, error);
            return false;
        }
        let delay = self.options.delay(attempt);
        self.lifecycle
            .reconnecting(self.kind, attempt, delay, error);
        tokio::select! {
            () = self.cancellation.cancelled() => false,
            () = tokio::time::sleep(delay) => true,
        }
    }

    pub(crate) fn ensure_open(&self) {
        assert!(
            !self.cancellation.is_cancelled(),
            "listener registration rejected because the client is closed"
        );
    }
}

pub(crate) fn local_principal_id(principal_id: &str) -> Result<&str, AdaError> {
    let normalized = principal_id.trim();
    let local = normalized.rsplit(':').next().unwrap_or_default();
    if local.is_empty() {
        return Err(AdaError {
            code: Code::Internal,
            message: "stream event is missing principal_id".to_owned(),
        });
    }
    Ok(local)
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;
    use std::sync::atomic::{AtomicUsize, Ordering};

    use super::*;

    #[test]
    fn unsubscribe_is_independent_and_idempotent() {
        let listeners = ListenerSet::<usize>::default();
        let first = Arc::new(AtomicUsize::new(0));
        let second = Arc::new(AtomicUsize::new(0));
        let first_calls = first.clone();
        let unsubscribe = listeners.on(move |_| {
            first_calls.fetch_add(1, Ordering::AcqRel);
        });
        let second_calls = second.clone();
        listeners.on(move |_| {
            second_calls.fetch_add(1, Ordering::AcqRel);
        });
        unsubscribe.unsubscribe();
        unsubscribe.unsubscribe();
        listeners.emit(1);
        assert_eq!(first.load(Ordering::Acquire), 0);
        assert_eq!(second.load(Ordering::Acquire), 1);
    }

    #[test]
    fn listener_panic_is_isolated() {
        let listeners = ListenerSet::<usize>::default();
        let failures = Arc::new(AtomicUsize::new(0));
        let delivered = Arc::new(AtomicUsize::new(0));
        listeners.on(|_| panic!("broken listener"));
        let delivered_calls = delivered.clone();
        listeners.on(move |_| {
            delivered_calls.fetch_add(1, Ordering::AcqRel);
        });
        let failure_calls = failures.clone();
        listeners.emit_with(1, move |_| {
            failure_calls.fetch_add(1, Ordering::AcqRel);
        });
        assert_eq!(failures.load(Ordering::Acquire), 1);
        assert_eq!(delivered.load(Ordering::Acquire), 1);
    }

    #[test]
    fn qualified_principal_is_routed_to_bare_id() {
        assert_eq!(local_principal_id("namespace:alice").unwrap(), "alice");
        assert!(local_principal_id("").is_err());
    }

    #[test]
    fn cursor_metadata_identifies_stream_and_principal() {
        let lifecycle = Lifecycle::default();
        let observed = Arc::new(Mutex::new(None));
        let captured = observed.clone();
        lifecycle.on_cursor(move |info| {
            *captured.lock().expect("cursor lock poisoned") = Some(info);
        });
        lifecycle.cursor(
            StreamKind::Events,
            "cursor-1".to_owned(),
            "alice".to_owned(),
        );
        let info = observed
            .lock()
            .expect("cursor lock poisoned")
            .clone()
            .expect("cursor event missing");
        assert_eq!(info.stream, StreamKind::Events);
        assert_eq!(info.event_id, "cursor-1");
        assert_eq!(info.principal_id, "alice");
    }

    #[test]
    #[should_panic(expected = "listener registration rejected because the client is closed")]
    fn listener_registration_after_close_is_rejected() {
        let cancellation = CancellationToken::new();
        cancellation.cancel();
        let control = HubControl::new(
            cancellation,
            Arc::new(Lifecycle::default()),
            StreamKind::Events,
            SubscriptionOptions::default(),
        );
        control.ensure_open();
    }
}
