//! Abstraction over various executors that actors can run on
use std::{convert::Infallible, future::Future};

use async_trait::async_trait;
use futures::future::BoxFuture;

#[cfg(feature = "async-std")]
pub mod asyncstd;
pub mod threads;

#[cfg(feature = "async-std")]
pub use asyncstd::AsyncStd;
pub use threads::Threads;

use crate::types::Either;

/// Asynchronous join handle
///
/// Provides an executor-agnostic abstraction over joining an asynchronous task
#[async_trait]
pub trait AsyncJoin {
    /// Item being joined
    type Item;
    /// Error type for failed joins
    type Error: std::error::Error + 'static;
    /// Asynchronously join this task
    ///
    /// # Errors
    ///
    /// Returns an error if joining the task fails
    async fn async_join(self) -> Result<Self::Item, Self::Error>;
}

#[async_trait]
impl<T> AsyncJoin for BoxFuture<'static, T>
where
    T: Send,
{
    type Item = T;
    type Error = Infallible;
    async fn async_join(self) -> Result<Self::Item, Self::Error> {
        Ok(self.await)
    }
}

/// Synchronous join handle
///
/// Provides an executor-agnostic abstraction over joining synchronous/blocking tasks or threads
pub trait SyncJoin {
    /// Item being joined
    type Item;
    /// Error type for failed joins
    type Error: std::error::Error + 'static;
    /// Synchronously join this thread
    ///
    /// # Errors
    ///
    /// Returns an error if joining the task fails
    fn sync_join(self) -> Result<Self::Item, Self::Error>;
    /// Attempts to convert this handle to an asynchronous one
    ///
    /// Will return `Either::Happy` if the conversion is possible, `Either::Sad` otherwise
    fn to_async(self) -> Either<Box<dyn AsyncJoin<Item = Self::Item, Error = Self::Error>>, Self>
    where
        Self: Sized,
    {
        Either::Sad(self)
    }
}

/// Abstract executor for actors
pub trait Executor: Send + Sync + 'static {
    /// Async join handle
    type AsyncJoin<T: Send + 'static>: AsyncJoin<Item = T>;
    /// Sync join handle
    type SyncJoin<T: Send + 'static>: SyncJoin<Item = T>;
    /// Spawns a future onto the executor as a task, returning a handle that can be `.await`ed for
    /// its result
    ///
    /// On async executors, this corresponds to an asynchronous task.
    /// On the [`Threads`] executor, this corresponds to a new thread with a single-threaded futures
    /// executor running on it.
    fn spawn_async<T, F>(future: F) -> Self::AsyncJoin<T>
    where
        T: Send + Sync + 'static,
        F: Future<Output = T> + Send + 'static;
    /// Spawns a closure onto its own system thread or blocking task, returning a handle that can be
    /// blocked on for its result
    ///
    /// On async executors this corresponds to a task on the blocking thread pool, or a new thread
    /// if the executor does not provide a blocking thread pool
    /// On the [`Threads`] executor, this corresponds to a new thread.
    fn spawn_sync<T, C>(closure: C) -> Self::SyncJoin<T>
    where
        T: Send + Sync + 'static,
        C: FnOnce() -> T + Send + 'static;
}
