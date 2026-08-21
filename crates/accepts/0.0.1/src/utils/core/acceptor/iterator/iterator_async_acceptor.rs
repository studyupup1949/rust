use core::{future::Future, marker::PhantomData};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `Accepts<I>` implementation that forwards each item to the next acceptor.
#[must_use = "IteratorAsyncAcceptor must be used to forward iterator items asynchronously"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct IteratorAsyncAcceptor<Iter, NextAccepts>
where
    Iter: IntoIterator,
    NextAccepts: AsyncAccepts<Iter::Item>,
{
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Iter>,
}

impl<Iter, NextAccepts> IteratorAsyncAcceptor<Iter, NextAccepts>
where
    Iter: IntoIterator,
    NextAccepts: AsyncAccepts<Iter::Item>,
{
    /// Creates a new `IteratorAcceptor`.
    pub fn new(next: NextAccepts) -> Self {
        Self {
            next_acceptor: next,
            _marker: PhantomData,
        }
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Iter, NextAccepts> AsyncAccepts<Iter> for IteratorAsyncAcceptor<Iter, NextAccepts>
where
    Iter: IntoIterator,
    NextAccepts: AsyncAccepts<Iter::Item>,
{
    fn accept_async<'a>(&'a self, iter: Iter) -> impl Future<Output = ()> + 'a
    where
        Iter: 'a,
    {
        async move {
            for item in iter {
                self.next_acceptor.accept_async(item).await;
            }
        }
    }
}
