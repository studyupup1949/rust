use core::{
    future::Future,
    marker::PhantomData,
    sync::atomic::{AtomicBool, Ordering},
};

use accepts::AsyncAccepts;

/// `AsyncAccepts<T>` implementation that forwards only once.
#[must_use = "AsyncOnce must be used to enforce single async acceptance"]
#[derive(Debug)]
pub struct AsyncOnce<Value, NextAccepts> {
    executed: AtomicBool,
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, NextAccepts> AsyncOnce<Value, NextAccepts>
where
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `AsyncOnce`.
    pub fn new(next_acceptor: NextAccepts) -> Self {
        Self {
            executed: AtomicBool::new(false),
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, NextAccepts> AsyncAccepts<Value> for AsyncOnce<Value, NextAccepts>
where
    NextAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async {
            if !self.executed.swap(true, Ordering::SeqCst) {
                self.next_acceptor.accept_async(value).await;
            }
        }
    }
}
