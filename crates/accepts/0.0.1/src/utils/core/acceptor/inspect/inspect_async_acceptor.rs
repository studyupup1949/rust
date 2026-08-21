use core::{future::Future, marker::PhantomData};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `AsyncAccepts<Value>` implementation that inspects the value before passing it on.
#[must_use = "InspectAsyncAcceptor must be used to run the async inspection before forwarding"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct InspectAsyncAcceptor<Value, InspectFn, InspectFut, NextAccepts>
where
    InspectFn: Fn(&Value) -> InspectFut,
    InspectFut: Future<Output = ()>,
    NextAccepts: AsyncAccepts<Value>,
{
    inspect_fn: InspectFn,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, InspectFn, InspectFut, NextAccepts>
    InspectAsyncAcceptor<Value, InspectFn, InspectFut, NextAccepts>
where
    InspectFn: Fn(&Value) -> InspectFut,
    InspectFut: Future<Output = ()>,
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `InspectAsyncAcceptor`.
    pub fn new(inspect_fn: InspectFn, next_acceptor: NextAccepts) -> Self {
        Self {
            inspect_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, InspectFn, InspectFut, NextAccepts> AsyncAccepts<Value>
    for InspectAsyncAcceptor<Value, InspectFn, InspectFut, NextAccepts>
where
    InspectFn: Fn(&Value) -> InspectFut,
    InspectFut: Future<Output = ()>,
    NextAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async move {
            (self.inspect_fn)(&value).await;
            self.next_acceptor.accept_async(value).await;
        }
    }
}
