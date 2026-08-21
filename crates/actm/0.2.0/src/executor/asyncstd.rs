//! Support for `async_std`

use std::{convert::Infallible, future::Future};

use async_std::task::JoinHandle;
use async_trait::async_trait;

use super::{AsyncJoin, Executor, SyncJoin};
use crate::types::Either;

#[async_trait]
impl<T> AsyncJoin for JoinHandle<T>
where
    T: Send,
{
    type Item = T;
    type Error = Infallible;
    async fn async_join(self) -> Result<Self::Item, Self::Error> {
        Ok(self.await)
    }
}

impl<T> SyncJoin for JoinHandle<T>
where
    T: Send + 'static,
{
    type Item = T;

    type Error = Infallible;

    fn sync_join(self) -> Result<Self::Item, Self::Error> {
        Ok(futures::executor::block_on(self))
    }

    fn to_async(self) -> Either<Box<dyn AsyncJoin<Item = Self::Item, Error = Self::Error>>, Self>
    where
        Self: Sized,
    {
        Either::Happy(Box::new(self))
    }
}

/// [`Executor`] implementation for `async_std`
///
/// Implements async task spawning through [`async_std::task::spawn`], and synchronous spawning
/// through [`async_std::task::spawn_blocking`]
pub struct AsyncStd;

impl Executor for AsyncStd {
    type AsyncJoin<T: Send + 'static> = JoinHandle<T>;
    type SyncJoin<T: Send + 'static> = JoinHandle<T>;
    fn spawn_async<T, F>(future: F) -> Self::AsyncJoin<T>
    where
        T: Send + Sync + 'static,
        F: Future<Output = T> + Send + 'static,
    {
        async_std::task::spawn(future)
    }

    fn spawn_sync<T, C>(closure: C) -> Self::SyncJoin<T>
    where
        T: Send + Sync + 'static,
        C: FnOnce() -> T + Send + 'static,
    {
        async_std::task::spawn_blocking(closure)
    }
}
