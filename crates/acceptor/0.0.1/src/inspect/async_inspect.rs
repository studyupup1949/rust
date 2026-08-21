use accepts::AsyncAccepts;
use core::{future::Future, marker::PhantomData};

/// `AsyncAccepts<Value>` implementation that inspects the value before passing it on.
#[must_use = "AsyncInspect must be used to run the async inspection before forwarding"]
#[derive(Debug, Clone)]
pub struct AsyncInspect<Value, InspectFn, InspectFut, NextAccepts> {
    inspect_fn: InspectFn,
    next_acceptor: NextAccepts,
    _marker: PhantomData<(Value, InspectFut)>,
}

impl<Value, InspectFn, InspectFut, NextAccepts>
    AsyncInspect<Value, InspectFn, InspectFut, NextAccepts>
where
    InspectFn: Fn(&Value) -> InspectFut,
    InspectFut: Future<Output = ()>,
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `AsyncInspect`.
    pub fn new(inspect_fn: InspectFn, next_acceptor: NextAccepts) -> Self {
        Self {
            inspect_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, InspectFn, InspectFut, NextAccepts> AsyncAccepts<Value>
    for AsyncInspect<Value, InspectFn, InspectFut, NextAccepts>
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
