use accepts::AsyncAccepts;
use core::{future::Future, marker::PhantomData};

/// `AsyncAccepts<I>` implementation that forwards each item to the next acceptor.
#[must_use = "AsyncForEach must be used to forward iterator items asynchronously"]
#[derive(Debug, Clone)]
pub struct AsyncForEach<Iter, NextAccepts> {
    next_acceptor: NextAccepts,
    _marker: PhantomData<Iter>,
}

impl<Iter, NextAccepts> AsyncForEach<Iter, NextAccepts>
where
    Iter: IntoIterator,
    NextAccepts: AsyncAccepts<Iter::Item>,
{
    /// Creates a new `AsyncForEach`.
    pub fn new(next_acceptor: NextAccepts) -> Self {
        Self {
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Iter, NextAccepts> AsyncAccepts<Iter> for AsyncForEach<Iter, NextAccepts>
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
