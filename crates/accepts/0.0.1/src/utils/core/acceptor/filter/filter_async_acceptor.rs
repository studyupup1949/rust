use core::{future::Future, marker::PhantomData};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// An `AsyncAccepts<Value>` implementation that forwards values when the predicate resolves to `true`.
#[must_use = "FilterAsyncAcceptor must be used to apply the async filter predicate"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct FilterAsyncAcceptor<Value, Predicate, PredicateFut, NextAccepts>
where
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
    NextAccepts: AsyncAccepts<Value>,
{
    predicate: Predicate,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<(Value, PredicateFut)>,
}

impl<Value, Predicate, PredicateFut, NextAccepts>
    FilterAsyncAcceptor<Value, Predicate, PredicateFut, NextAccepts>
where
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `FilterAsyncAcceptor`.
    pub fn new(predicate: Predicate, next_acceptor: NextAccepts) -> Self {
        Self {
            predicate,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, Predicate, PredicateFut, NextAccepts> AsyncAccepts<Value>
    for FilterAsyncAcceptor<Value, Predicate, PredicateFut, NextAccepts>
where
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
    NextAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async {
            if (self.predicate)(&value).await {
                self.next_acceptor.accept_async(value).await;
            }
        }
    }
}
