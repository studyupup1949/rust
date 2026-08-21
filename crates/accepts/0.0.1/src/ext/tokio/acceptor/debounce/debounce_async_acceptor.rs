use core::future::Future;
use std::{
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use tokio::{sync::Mutex, time::sleep};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `AsyncAccepts` implementation that debounces incoming values.
#[must_use = "DebounceAsyncAcceptor must be used to ensure async debounce semantics"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct DebounceAsyncAcceptor<Value, NextAccepts>
where
    Value: Send + 'static,
    NextAccepts: AsyncAccepts<Value> + Clone + Send + 'static,
{
    delay: Duration,
    value: Arc<Mutex<Option<Value>>>,
    counter: Arc<AtomicUsize>,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
}

impl<Value, NextAccepts> DebounceAsyncAcceptor<Value, NextAccepts>
where
    Value: Send + 'static,
    NextAccepts: AsyncAccepts<Value> + Clone + Send + 'static,
{
    /// Creates a new `DebounceAsyncAcceptor`.
    pub fn new(delay: Duration, next_acceptor: NextAccepts) -> Self {
        Self {
            delay,
            value: Arc::new(Mutex::new(None)),
            counter: Arc::new(AtomicUsize::new(0)),
            next_acceptor,
        }
    }
}

#[auto_impl_dyn_internal]
impl<Value, NextAccepts> AsyncAccepts<Value> for DebounceAsyncAcceptor<Value, NextAccepts>
where
    Value: Send + 'static,
    NextAccepts: AsyncAccepts<Value> + Clone + Send + 'static,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async move {
            {
                let mut slot = self.value.lock().await;
                *slot = Some(value);
            }
            let id = self.counter.fetch_add(1, Ordering::SeqCst) + 1;
            sleep(self.delay).await;
            let value_opt = if self.counter.load(Ordering::SeqCst) == id {
                let mut slot = self.value.lock().await;
                slot.take()
            } else {
                None
            };
            if let Some(v) = value_opt {
                self.next_acceptor.accept_async(v).await;
            }
        }
    }
}
