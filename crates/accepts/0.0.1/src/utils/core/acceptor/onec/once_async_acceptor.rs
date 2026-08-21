use core::{
    future::Future,
    marker::PhantomData,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `Accepts<T>` implementation that forwards only once.
#[must_use = "OnceAsyncAcceptor must be used to enforce single async acceptance"]
#[derive(Debug, NextAcceptorsInternal)]
pub struct OnceAsyncAcceptor<Value, NextAccepts>
where
    NextAccepts: AsyncAccepts<Value>,
{
    executed: AtomicBool,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, NextAccepts> OnceAsyncAcceptor<Value, NextAccepts>
where
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `OnceAsyncAcceptor`.
    pub fn new(next_acceptor: NextAccepts) -> Self {
        Self {
            executed: AtomicBool::new(false),
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, NextAccepts> AsyncAccepts<Value> for OnceAsyncAcceptor<Value, NextAccepts>
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
