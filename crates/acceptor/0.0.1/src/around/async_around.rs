use accepts::AsyncAccepts;
use core::{future::Future, marker::PhantomData};

/// `AsyncAccepts<Value>` that runs async hooks with shared context before and after delegating.
#[must_use = "AsyncAround must be used so the async before/after hooks run"]
#[derive(Debug, Clone)]
pub struct AsyncAround<Value, Context, BeforeFn, BeforeFut, AfterFn, AfterFut, NextAccepts> {
    context: Context,
    before_fn: BeforeFn,
    after_fn: AfterFn,
    next_acceptor: NextAccepts,
    _marker: PhantomData<(Value, BeforeFut, AfterFut)>,
}

impl<Value, Context, BeforeFn, BeforeFut, AfterFn, AfterFut, NextAccepts>
    AsyncAround<Value, Context, BeforeFn, BeforeFut, AfterFn, AfterFut, NextAccepts>
where
    BeforeFn: Fn(&Context) -> BeforeFut,
    BeforeFut: Future<Output = ()>,
    AfterFn: Fn(&Context) -> AfterFut,
    AfterFut: Future<Output = ()>,
    NextAccepts: AsyncAccepts<Value>,
{
    pub fn new(
        context: Context,
        before_fn: BeforeFn,
        after_fn: AfterFn,
        next_acceptor: NextAccepts,
    ) -> Self {
        Self {
            context,
            before_fn,
            after_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, Context, BeforeFn, BeforeFut, AfterFn, AfterFut, NextAccepts> AsyncAccepts<Value>
    for AsyncAround<Value, Context, BeforeFn, BeforeFut, AfterFn, AfterFut, NextAccepts>
where
    BeforeFn: Fn(&Context) -> BeforeFut,
    BeforeFut: Future<Output = ()>,
    AfterFn: Fn(&Context) -> AfterFut,
    AfterFut: Future<Output = ()>,
    NextAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async {
            (self.before_fn)(&self.context).await;
            self.next_acceptor.accept_async(value).await;
            (self.after_fn)(&self.context).await;
        }
    }
}
