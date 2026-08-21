use core::{fmt::Debug, marker::PhantomData};
use tracing::Level;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<T>` implementation that emits a tracing event before forwarding the value.
#[must_use = "TracingAcceptor must be used to emit tracing events for accepted values"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct TracingAcceptor<Value, LevelFn, NextAccepts>
where
    Value: Debug,
    LevelFn: Fn(&Value) -> Level,
    NextAccepts: Accepts<Value>,
{
    level_fn: LevelFn,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, LevelFn, NextAccepts> TracingAcceptor<Value, LevelFn, NextAccepts>
where
    Value: Debug,
    LevelFn: Fn(&Value) -> Level,
    NextAccepts: Accepts<Value>,
{
    pub fn new(level_fn: LevelFn, next_acceptor: NextAccepts) -> Self {
        Self {
            level_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }

    pub fn with_level(
        level: Level,
        next: NextAccepts,
    ) -> TracingAcceptor<Value, impl Fn(&Value) -> Level, NextAccepts> {
        TracingAcceptor::new(move |_| level, next)
    }
}

impl<Value, LevelFn, NextAccepts> Accepts<Value> for TracingAcceptor<Value, LevelFn, NextAccepts>
where
    Value: Debug,
    LevelFn: Fn(&Value) -> Level,
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        let level = (self.level_fn)(&value);
        match level {
            Level::TRACE => tracing::trace!(?value),
            Level::DEBUG => tracing::debug!(?value),
            Level::INFO => tracing::info!(?value),
            Level::WARN => tracing::warn!(?value),
            Level::ERROR => tracing::error!(?value),
        }
        self.next_acceptor.accept(value);
    }
}
