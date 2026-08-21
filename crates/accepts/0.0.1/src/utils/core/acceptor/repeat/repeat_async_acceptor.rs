use core::{
    future::{Future, Ready, ready},
    marker::PhantomData,
};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

#[must_use = "RepeatAsyncAcceptor must be used to apply the async repeat count when forwarding values"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct RepeatAsyncAcceptor<Value, RepeatCountFn, RepeatCountFut, NextAccepts>
where
    Value: Clone,
    RepeatCountFn: Fn(&Value) -> RepeatCountFut,
    RepeatCountFut: Future<Output = usize>,
    NextAccepts: AsyncAccepts<Value>,
{
    repeat_count_fn: RepeatCountFn,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, RepeatCountFn, RepeatCountFut, NextAccepts>
    RepeatAsyncAcceptor<Value, RepeatCountFn, RepeatCountFut, NextAccepts>
where
    Value: Clone,
    RepeatCountFn: Fn(&Value) -> RepeatCountFut,
    RepeatCountFut: Future<Output = usize>,
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `RepeatAsyncAcceptor` with a repeat count function.
    pub fn with_fn(repeat_count_fn: RepeatCountFn, next_acceptor: NextAccepts) -> Self {
        Self {
            repeat_count_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }

    /// Creates a new `RepeatAsyncAcceptor` with a fixed repeat count.
    pub fn new(
        repeat_count: usize,
        next_acceptor: NextAccepts,
    ) -> RepeatAsyncAcceptor<Value, impl Fn(&Value) -> Ready<usize>, Ready<usize>, NextAccepts>
    {
        RepeatAsyncAcceptor {
            repeat_count_fn: move |_| ready(repeat_count),
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, RepeatCountFn, RepeatCountFut, NextAccepts> AsyncAccepts<Value>
    for RepeatAsyncAcceptor<Value, RepeatCountFn, RepeatCountFut, NextAccepts>
where
    Value: Clone,
    RepeatCountFn: Fn(&Value) -> RepeatCountFut,
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
