use core::{
    fmt::Debug,
    future::{Future, Ready},
    marker::PhantomData,
};

use log::{Level, log};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `Accepts<T>` implementation that logs the value before forwarding it.
#[must_use = "LoggingAsyncAcceptor must be used to emit async log entries for accepted values"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct LoggingAsyncAcceptor<Value, LevelFn, LevelFut, NextAccepts>
where
    Value: Debug,
    LevelFn: Fn(&Value) -> LevelFut,
    LevelFut: Future<Output = Level>,
    NextAccepts: AsyncAccepts<Value>,
{
    level_fn: LevelFn,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, LevelFn, LevelFut, NextAccepts>
    LoggingAsyncAcceptor<Value, LevelFn, LevelFut, NextAccepts>
where
    Value: Debug,
    LevelFn: Fn(&Value) -> LevelFut,
    LevelFut: Future<Output = Level>,
    NextAccepts: AsyncAccepts<Value>,
{
    pub fn new(
        level: Level,
        next: NextAccepts,
    ) -> LoggingAsyncAcceptor<Value, impl Fn(&Value) -> Ready<Level>, Ready<Level>, NextAccepts>
    {
        LoggingAsyncAcceptor::with_fn(move |_| core::future::ready(level), next)
    }

    pub fn with_fn(level_fn: LevelFn, next_acceptor: NextAccepts) -> Self {
        Self {
            level_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, LevelFn, LevelFut, NextAccepts> AsyncAccepts<Value>
    for LoggingAsyncAcceptor<Value, LevelFn, LevelFut, NextAccepts>
where
    Value: Debug,
    LevelFn: Fn(&Value) -> LevelFut,
    LevelFut: Future<Output = Level>,
    NextAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async {
            let level = (self.level_fn)(&value).await;
            log!(level, "{:?}", value);
            self.next_acceptor.accept_async(value).await;
        }
    }
}
