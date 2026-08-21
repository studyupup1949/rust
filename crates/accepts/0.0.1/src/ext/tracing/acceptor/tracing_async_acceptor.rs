use core::{fmt::Debug, future::Future, marker::PhantomData};
use tracing::Level;

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `AsyncAccepts<T>` implementation that emits a tracing event before forwarding the value.
#[must_use = "TracingAsyncAcceptor must be used to emit async tracing events for accepted values"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct TracingAsyncAcceptor<Value, LevelFn, LevelFut, NextAccepts>
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
    TracingAsyncAcceptor<Value, LevelFn, LevelFut, NextAccepts>
where
    Value: Debug,
    LevelFn: Fn(&Value) -> LevelFut,
    LevelFut: Future<Output = Level>,
    NextAccepts: AsyncAccepts<Value>,
{
    pub fn new(level_fn: LevelFn, next_acceptor: NextAccepts) -> Self {
        Self {
            level_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<Value, LevelFn, LevelFut, NextAccepts> AsyncAccepts<Value>
    for TracingAsyncAcceptor<Value, LevelFn, LevelFut, NextAccepts>
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
        async move {
            let level = (self.level_fn)(&value).await;
            match level {
                Level::TRACE => tracing::trace!(?value),
                Level::DEBUG => tracing::debug!(?value),
                Level::INFO => tracing::info!(?value),
                Level::WARN => tracing::warn!(?value),
                Level::ERROR => tracing::error!(?value),
            }
            self.next_acceptor.accept_async(value).await;
        }
    }
}
