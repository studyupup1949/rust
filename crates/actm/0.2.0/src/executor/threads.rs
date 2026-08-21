//! Thread-per-task support
use std::future::Future;

use async_trait::async_trait;
use flume::Receiver;

use super::{AsyncJoin, Executor, SyncJoin};
use crate::types::Either;

/// [`Executor`] implementation on top of native threads
///
/// Uses the `futures` crate local executor inside a newly spawned thread for handling async tasks
pub struct Threads;

/// Error type indicating that background thread was closed before joining
#[derive(Clone, Copy, Debug)]
pub struct Closed;

impl std::error::Error for Closed {}

impl std::fmt::Display for Closed {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Background task or thread closed before join")
    }
}

/// Join handle for a thread
pub struct ThreadJoin<T>(Receiver<T>);

#[async_trait]
impl<T> AsyncJoin for ThreadJoin<T>
where
    T: Send,
{
    type Item = T;
    type Error = Closed;
    async fn async_join(self) -> Result<Self::Item, Self::Error> {
        self.0.recv_async().await.map_err(|_| Closed)
    }
}

impl<T> SyncJoin for ThreadJoin<T>
where
    T: Send + 'static,
{
    type Item = T;
    type Error = Closed;
    fn sync_join(self) -> Result<Self::Item, Self::Error> {
        self.0.recv().map_err(|_| Closed)
    }

    fn to_async(self) -> Either<Box<dyn AsyncJoin<Item = Self::Item, Error = Self::Error>>, Self>
    where
        Self: Sized,
    {
        Either::Happy(Box::new(self))
    }
}

impl Executor for Threads {
    type AsyncJoin<T: Send + 'static> = ThreadJoin<T>;
    type SyncJoin<T: Send + 'static> = ThreadJoin<T>;
    fn spawn_async<T, F>(future: F) -> Self::AsyncJoin<T>
    where
        T: Send + Sync + 'static,
        F: Future<Output = T> + Send + 'static,
    {
        let (i, o) = flume::bounded(1);
        std::thread::spawn(move || {
            let output = futures::executor::block_on(future);
            let _result = i.send(output);
        });
        ThreadJoin(o)
    }

    fn spawn_sync<T, C>(closure: C) -> Self::SyncJoin<T>
    where
        T: Send + Sync + 'static,
        C: FnOnce() -> T + Send + 'static,
    {
        let (i, o) = flume::bounded(1);
        std::thread::spawn(move || {
            let output = closure();
            let _result = i.send(output);
        });
        ThreadJoin(o)
    }
}
