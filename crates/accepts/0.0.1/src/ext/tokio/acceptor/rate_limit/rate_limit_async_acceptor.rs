use std::{marker::PhantomData, time::Duration};
use tokio::{
    sync::Mutex,
    time::{Instant, sleep_until},
};

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
    utils::std::acceptor::RateLimitStrategy,
};

/// `AsyncAccepts` implementation that limits the rate of forwarded values.
#[must_use = "RateLimitAsyncAcceptor must be used to enforce the configured async rate limits"]
#[derive(Debug, NextAcceptorsInternal)]
pub struct RateLimitAsyncAcceptor<Value, NextAccepts>
where
    NextAccepts: AsyncAccepts<Value>,
{
    interval: Duration,
    strategy: RateLimitStrategy,
    last: Mutex<Option<Instant>>,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, NextAccepts> RateLimitAsyncAcceptor<Value, NextAccepts>
where
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `RateLimitAsyncAcceptor`.
    pub fn new(
        interval: Duration,
        strategy: RateLimitStrategy,
        next_acceptor: NextAccepts,
    ) -> Self {
        Self {
            interval,
            strategy,
            last: Mutex::new(None),
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

#[auto_impl_dyn_internal]
impl<Value, NextAccepts> AsyncAccepts<Value> for RateLimitAsyncAcceptor<Value, NextAccepts>
where
    NextAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl core::future::Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async move {
            let mut last = self.last.lock().await;
            let now = Instant::now();
            if let Some(prev) = *last {
                let deadline = prev + self.interval;
                if now < deadline {
                    match self.strategy {
                        RateLimitStrategy::Drop => return,
                        RateLimitStrategy::Wait => {
                            sleep_until(deadline).await;
                            *last = Some(Instant::now());
                            drop(last);
                            self.next_acceptor.accept_async(value).await;
                            return;
                        }
                    }
                }
            }
            *last = Some(now);
            drop(last);
            self.next_acceptor.accept_async(value).await;
        }
    }
}
