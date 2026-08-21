use std::{
    cell::RefCell,
    marker::PhantomData,
    thread::sleep,
    time::{Duration, Instant},
};

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

use super::RateLimitStrategy;

/// `Accepts` implementation that limits the rate of forwarded values.
#[must_use = "RateLimitAcceptor must be used to enforce the configured rate limits"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct RateLimitAcceptor<Value, NextAccepts>
where
    NextAccepts: Accepts<Value>,
{
    interval: Duration,
    strategy: RateLimitStrategy,
    last: RefCell<Option<Instant>>,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, NextAccepts> RateLimitAcceptor<Value, NextAccepts>
where
    NextAccepts: Accepts<Value>,
{
    /// Creates a new `RateLimitAcceptor`.
    pub fn new(
        interval: Duration,
        strategy: RateLimitStrategy,
        next_acceptor: NextAccepts,
    ) -> Self {
        Self {
            interval,
            strategy,
            last: RefCell::new(None),
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

impl<Value, NextAccepts> Accepts<Value> for RateLimitAcceptor<Value, NextAccepts>
where
    NextAccepts: Accepts<Value>,
{
    fn accept(&self, value: Value) {
        let mut last = self.last.borrow_mut();
        let now = Instant::now();
        if let Some(prev) = *last {
            let elapsed = now - prev;
            if elapsed < self.interval {
                match self.strategy {
                    RateLimitStrategy::Drop => return,
                    RateLimitStrategy::Wait => {
                        sleep(self.interval - elapsed);
                        *last = Some(Instant::now());
                        drop(last);
                        self.next_acceptor.accept(value);
                        return;
                    }
                }
            }
        }
        *last = Some(now);
        drop(last);
        self.next_acceptor.accept(value);
    }
}
