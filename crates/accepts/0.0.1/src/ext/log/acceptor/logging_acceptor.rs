use core::{fmt::Debug, marker::PhantomData};

use log::{Level, log};

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<T>` implementation that logs the value before forwarding it.
#[must_use = "LoggingAcceptor must be used to emit log entries for accepted values"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct LoggingAcceptor<Value, LevelFn, NextAccepts>
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

impl<Value, LevelFn, NextAccepts> LoggingAcceptor<Value, LevelFn, NextAccepts>
where
    Value: Debug,
    LevelFn: Fn(&Value) -> Level,
    NextAccepts: Accepts<Value>,
{
    pub fn new(
        level: Level,
        next: NextAccepts,
    ) -> LoggingAcceptor<Value, impl Fn(&Value) -> Level, NextAccepts> {
        LoggingAcceptor::with_fn(move |_| level, next)
    }

    pub fn with_fn(level_fn: LevelFn, next_acceptor: NextAccepts) -> Self {
        Self {
            level_fn,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, LevelFn, NextAccepts> Accepts<Value> for LoggingAcceptor<Value, LevelFn, NextAccepts>
where
    Value: Debug,
    LevelFn: Fn(&Value) -> Level,
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        let level = (self.level_fn)(&value);
        log!(level, "{:?}", value);
        self.next_acceptor.accept(value);
    }
}
