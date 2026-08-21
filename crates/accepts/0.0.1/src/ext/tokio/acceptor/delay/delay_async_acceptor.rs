use core::{future::Future, marker::PhantomData};
use std::time::Duration;
use tokio::time::sleep;

use crate::{
    core_traits::AsyncAccepts,
    macros::internal::codegen::{NextAcceptorsInternal, auto_impl_dyn_internal},
};

/// `AsyncAccepts` implementation that waits for a duration before forwarding the value.
///
/// The delay duration is configured when constructing this acceptor via `DelayAsyncAcceptor::new`.
/// This type is available through the `accepts::ext::tokio::acceptor` module for Tokio-based
/// integrations.
///
/// # Examples
///
/// ```ignore
/// use accepts::ext::tokio::acceptor::DelayAsyncAcceptor;
/// use std::time::Duration;
///
/// # struct Print;
/// # impl accepts::core_traits::AsyncAccepts<i32> for Print {
/// #     fn accept_async<'a>(&'a self, value: i32) -> impl Future<Output = ()> + 'a {
/// #         async move { println!("{}", value); }
/// #     }
/// # }
/// let next = Print;
/// // delay for 100 milliseconds before forwarding the value
/// let _ = DelayAsyncAcceptor::new(Duration::from_millis(100), next);
/// ```
#[must_use = "DelayAsyncAcceptor must be used to apply the configured delay"]
#[derive(Debug, NextAcceptorsInternal)]
pub struct DelayAsyncAcceptor<Value, NextAccepts>
where
    NextAccepts: AsyncAccepts<Value>,
{
    delay: Duration,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    _marker: PhantomData<Value>,
}

impl<Value, NextAccepts> DelayAsyncAcceptor<Value, NextAccepts>
where
    NextAccepts: AsyncAccepts<Value>,
{
    /// Creates a new `DelayAsyncAcceptor` with the given delay duration.
    pub fn new(delay: Duration, next_acceptor: NextAccepts) -> Self {
        Self {
            delay,
            next_acceptor,
            _marker: PhantomData,
        }
    }
}

#[auto_impl_dyn_internal]
impl<Value, NextAccepts> AsyncAccepts<Value> for DelayAsyncAcceptor<Value, NextAccepts>
where
    NextAccepts: AsyncAccepts<Value>,
{
    fn accept_async<'a>(&'a self, value: Value) -> impl Future<Output = ()> + 'a
    where
        Value: 'a,
    {
        async move {
            sleep(self.delay).await;
            self.next_acceptor.accept_async(value).await;
        }
    }
}
