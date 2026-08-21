use core::{future::Future, marker::PhantomData};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `Accepts<Value>` that runs hooks with shared context before and after delegating.
#[must_use = "AroundAsyncAcceptor must be used so the async before/after hooks run"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct AroundAsyncAcceptor<Value, Context, BeforeFn, BeforeFut, AfterFn, AfterFut, NextAccepts>
where
    BeforeFn: Fn(&Context) -> BeforeFut,
    BeforeFut: Future<Output = ()>,
    AfterFn: Fn(&Context) -> AfterFut,
    AfterFut: Future<Output = ()>,
    NextAccepts: AsyncAccepts<Value>,
{
    context: Context,
    before_fn: BeforeFn,
    after_fn: AfterFn,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, Context, BeforeFn, BeforeFut, AfterFn, AfterFut, NextAccepts>
    AroundAsyncAcceptor<Value, Context, BeforeFn, BeforeFut, AfterFn, AfterFut, NextAccepts>
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

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, Context, BeforeFn, BeforeFut, AfterFn, AfterFut, NextAccepts> AsyncAccepts<Value>
    for AroundAsyncAcceptor<Value, Context, BeforeFn, BeforeFut, AfterFn, AfterFut, NextAccepts>
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
