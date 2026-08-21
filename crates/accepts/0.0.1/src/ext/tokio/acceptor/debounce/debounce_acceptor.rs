use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use tokio::{spawn, sync::Mutex, time::sleep};

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts` implementation that debounces incoming values.
///
/// Calling [`accept`](Accepts::accept) requires an active Tokio runtime. Without
/// a running runtime, the internal [`tokio::spawn`] call will panic.
///
/// # Examples
/// ```no_run
/// use std::time::Duration;
///
/// use accepts::core_traits::Accepts;
/// use accepts::ext::tokio::acceptor::DebounceAcceptor;
/// use tokio::runtime::Runtime;
///
/// #[derive(Clone)]
/// struct Sink;
///
/// impl Accepts<u32> for Sink {
///     fn accept(&self, value: u32) {
///         println!("Received {value}");
///     }
/// }
///
/// let debounce = DebounceAcceptor::new(Duration::from_millis(10), Sink);
/// let runtime = Runtime::new().expect("Tokio runtime");
/// runtime.block_on(async move {
///     debounce.accept(42);
///     tokio::time::sleep(Duration::from_millis(20)).await;
/// });
/// ```
#[must_use = "DebounceAcceptor must be used to ensure debounce semantics"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct DebounceAcceptor<Value, NextAccepts>
where
    Value: Send + 'static,
    NextAccepts: Accepts<Value> + Clone + Send + 'static,
{
    delay: Duration,
    value: Arc<Mutex<Option<Value>>>,
    counter: Arc<AtomicUsize>,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
}

impl<Value, NextAccepts> DebounceAcceptor<Value, NextAccepts>
where
    Value: Send + 'static,
    NextAccepts: Accepts<Value> + Clone + Send + 'static,
{
    /// Creates a new `DebounceAcceptor`.
    pub fn new(delay: Duration, next_acceptor: NextAccepts) -> Self {
        Self {
            delay,
            value: Arc::new(Mutex::new(None)),
            counter: Arc::new(AtomicUsize::new(0)),
            next_acceptor,
        }
    }
}

impl<Value, NextAccepts> Accepts<Value> for DebounceAcceptor<Value, NextAccepts>
where
    Value: Send + 'static,
    NextAccepts: Accepts<Value> + Clone + Send + 'static,
{
    fn accept(&self, value: Value) {
        {
            let mut slot = self.value.blocking_lock();
            *slot = Some(value);
        }
        let id = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
        let delay = self.delay;
        let value = Arc::clone(&self.value);
        let counter = Arc::clone(&self.counter);
        let next = self.next_acceptor.clone();
        spawn(async move {
            sleep(delay).await;
            if counter.load(Ordering::SeqCst) == id {
                if let Some(v) = value.lock().await.take() {
                    next.accept(v);
                }
            }
        });
    }
}
