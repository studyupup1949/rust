use core::{cell::Cell, marker::PhantomData, time::Duration};
use std::time::Instant;

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// `Accepts<Result<ResultValue, ErrValue>>` implementation that stops forwarding values
/// after a number of consecutive `Err` values. After a cooldown the breaker resets.
#[must_use = "CircuitBreakerAcceptor must be used to enforce circuit breaker semantics"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct CircuitBreakerAcceptor<OkValue, ErrValue, NextAccepts>
where
    NextAccepts: Accepts<OkValue>,
{
    max_errors: usize,
    cooldown: Duration,
    error_count: Cell<usize>,
    open_until: Cell<Option<Instant>>,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<(OkValue, ErrValue)>,
}

impl<OkValue, ErrValue, NextAccepts> CircuitBreakerAcceptor<OkValue, ErrValue, NextAccepts>
where
    NextAccepts: Accepts<OkValue>,
{
    /// Creates a new `CircuitBreakerAcceptor`.
    pub fn new(max_errors: usize, cooldown: Duration, next_acceptor: NextAccepts) -> Self {
        Self {
            max_errors,
            cooldown,
            error_count: Cell::new(0),
            open_until: Cell::new(None),
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<OkValue, ErrValue, NextAccepts> Accepts<Result<OkValue, ErrValue>>
    for CircuitBreakerAcceptor<OkValue, ErrValue, NextAccepts>
where
    NextAccepts: Accepts<OkValue>,
{
    fn accept(&self, value: Result<OkValue, ErrValue>) {
        let now = Instant::now();
        if let Some(until) = self.open_until.get() {
            if now < until {
                return;
            }
            self.open_until.set(None);
            self.error_count.set(0);
        }

        match value {
            Ok(v) => {
                self.error_count.set(0);
                self.next_acceptor.accept(v);
            }
            Err(_) => {
                let cnt = self.error_count.get() + 1;
                self.error_count.set(cnt);
                if cnt >= self.max_errors {
                    self.open_until.set(Some(now + self.cooldown));
                }
            }
        }
    }
}
