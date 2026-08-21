use core::{future::Future, marker::PhantomData};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `AsyncAccepts<T>` implementation that calls a closure.
#[must_use = "CallbackAsyncAcceptor must be used to run the async callback"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct CallbackAsyncAcceptor<Value, CallbackFn, CallbackFut, NextAccepts = ()>
where
    CallbackFn: Fn(Value, Option<&NextAccepts>) -> CallbackFut,
    CallbackFut: Future<Output = ()>,
    NextAccepts: AsyncAccepts<Value>,
{
    callback: CallbackFn,
    #[next_acceptor(option_once, ref, mut)]
    next_acceptor: Option<NextAccepts>,
    _marker: PhantomData<(Value, CallbackFut)>,
}

impl<Value, CallbackFn, CallbackFut, NextAccepts>
    CallbackAsyncAcceptor<Value, CallbackFn, CallbackFut, NextAccepts>
where
    CallbackFn: Fn(Value, Option<&NextAccepts>) -> CallbackFut,
    CallbackFut: Future<Output = ()>,
    NextAccepts: AsyncAccepts<Value>,
{
    pub fn with_next(callback: CallbackFn, next_acceptor: Option<NextAccepts>) -> Self {
        Self {
            callback,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, CallbackFut>
    CallbackAsyncAcceptor<Value, fn(Value, Option<&()>) -> CallbackFut, CallbackFut, ()>
where
    CallbackFut: Future<Output = ()>,
    (): AsyncAccepts<Value>,
{
    /// Creates a `CallbackAsyncAcceptor` from a simple async closure.
    pub fn new<G>(
        callback: G,
    ) -> CallbackAsyncAcceptor<Value, impl Fn(Value, Option<&()>) -> CallbackFut, CallbackFut, ()>
    where
        G: Fn(Value) -> CallbackFut,
    {
        CallbackAsyncAcceptor::with_next(move |value, _| callback(value), None)
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, CallbackFn, CallbackFut, NextAccepts> AsyncAccepts<Value>
    for CallbackAsyncAcceptor<Value, CallbackFn, CallbackFut, NextAccepts>
where
    CallbackFn: Fn(Value, Option<&NextAccepts>) -> CallbackFut,
    CallbackFut: Future<Output = ()>,
    NextAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        (self.callback)(value, self.next_acceptor.as_ref())
    }
}
