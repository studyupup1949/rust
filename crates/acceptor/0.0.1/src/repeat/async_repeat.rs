use accepts::AsyncAccepts;
use core::{
    future::{Future, Ready, ready},
    marker::PhantomData,
};

#[must_use = "AsyncRepeat must be used to apply the async repeat count when forwarding values"]
#[derive(Debug, Clone)]
pub struct AsyncRepeat<Value, RepeatCountFn, RepeatCountFut, NextAccepts> {
    repeat_count_fn: RepeatCountFn,
    next_acceptor: NextAccepts,
    _marker: PhantomData<(Value, RepeatCountFut)>,
}

impl<Value, RepeatCountFn, RepeatCountFut, NextAccepts>
    AsyncRepeat<Value, RepeatCountFn, RepeatCountFut, NextAccepts>
where
    Value: Clone,
    RepeatCountFn: for<'a> Fn(&'a Value) -> RepeatCountFut,
    RepeatCountFut: Future<Output = usize>,
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `AsyncRepeat` with a repeat count function.
    pub fn with_fn(repeat_count_fn: RepeatCountFn, next_acceptor: NextAccepts) -> Self {
        Self {
            repeat_count_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, NextAccepts> AsyncRepeat<Value, fn(&Value) -> Ready<usize>, Ready<usize>, NextAccepts>
where
    Value: Clone,
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `AsyncRepeat` with a fixed repeat count.
    pub fn new(
        repeat_count: usize,
        next_acceptor: NextAccepts,
    ) -> AsyncRepeat<Value, impl for<'a> Fn(&'a Value) -> Ready<usize>, Ready<usize>, NextAccepts>
    {
        let repeat_count_fn = move |_: &Value| ready(repeat_count);
        AsyncRepeat::with_fn(repeat_count_fn, next_acceptor)
    }
}

impl<Value, RepeatCountFn, RepeatCountFut, NextAccepts> AsyncAccepts<Value>
    for AsyncRepeat<Value, RepeatCountFn, RepeatCountFut, NextAccepts>
where
    Value: Clone,
    RepeatCountFn: for<'a> Fn(&'a Value) -> RepeatCountFut,
    RepeatCountFut: Future<Output = usize>,
    NextAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async {
            let n: usize = (self.repeat_count_fn)(&value).await;
            if n == 0 {
                return;
            }
            for _ in 0..n - 1 {
                self.next_acceptor.accept_async(value.clone()).await;
            }
            self.next_acceptor.accept_async(value).await;
        }
    }
}
