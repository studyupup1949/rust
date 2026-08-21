use accepts::AsyncAccepts;
use core::{future::Future, marker::PhantomData};

/// An `AsyncAccepts<Value>` implementation that forwards values when the predicate resolves to `true`.
#[must_use = "AsyncFilter must be used to apply the async filter predicate"]
#[derive(Debug, Clone)]
pub struct AsyncFilter<Value, Predicate, PredicateFut, NextAccepts> {
    predicate: Predicate,
    next_acceptor: NextAccepts,
    _marker: PhantomData<(Value, PredicateFut)>,
}

impl<Value, Predicate, PredicateFut, NextAccepts>
    AsyncFilter<Value, Predicate, PredicateFut, NextAccepts>
where
    Predicate: Fn(&Value) -> PredicateFut,
    PredicateFut: Future<Output = bool>,
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `AsyncFilter`.
    pub fn new(predicate: Predicate, next_acceptor: NextAccepts) -> Self {
        Self {
            predicate,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, Predicate, PredicateFut, NextAccepts> AsyncAccepts<Value>
    for AsyncFilter<Value, Predicate, PredicateFut, NextAccepts>
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
