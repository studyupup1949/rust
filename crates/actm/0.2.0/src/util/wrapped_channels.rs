//! Wrapper types for adapting flume receivers for connection to [`Actor`](crate::traits::Actor)s

use std::marker::PhantomData;

use async_mutex::Mutex;
use async_trait::async_trait;
use flume::{Receiver, Sender};
use snafu::Snafu;
use tracing::{Instrument, error, info_span};

use crate::{
    executor::Executor,
    traits::{Event, EventConsumer, EventProducer},
    types::CompletionToken,
};

/// Error type for `WrappedReceiver`
#[derive(Debug, Snafu)]
pub enum WrappedReceiverError {
    /// Attempted to register an already consumed receiver
    AlreadyRegistered,
    /// Attempted to register a callback, which is not supported by `WrappedReceiver`
    Callback,
}

/// Wrapper around a flume channel receiver that adapts it into an [`EventProducer`]
///
/// Only one [`EventConsumer`] can ever be registered to a given `WrappedReceiver`
///
/// This version asynchronously listens to its `Inbox` on a background task.
pub struct AsyncWrappedReceiver<X: Executor, T> {
    /// Underlying [`Receiver`]
    inner: Mutex<Option<Receiver<T>>>,
    /// Phantom for the executor type
    _executor: PhantomData<X>,
}

impl<X: Executor, T> AsyncWrappedReceiver<X, T> {
    /// Create a new channel, wrapping the receiving end in a `WrappedReceiver`
    ///
    /// The provided `bound` argument will be used to set the buffer size limit for the underlying
    /// `flume` queue, an unbounded queue will be used if present.
    pub fn new(bound: Option<usize>) -> (Sender<T>, Self) {
        let (tx, rx) = match bound {
            Some(bound) => flume::bounded(bound),
            None => flume::unbounded(),
        };
        (tx, Self::from(rx))
    }
}

impl<X: Executor, T> From<Receiver<T>> for AsyncWrappedReceiver<X, T> {
    fn from(inner: Receiver<T>) -> Self {
        Self {
            inner: Mutex::new(Some(inner)),
            _executor: PhantomData,
        }
    }
}

#[async_trait]
impl<X: Executor, T: Event> EventProducer<T> for AsyncWrappedReceiver<X, T> {
    type Error = WrappedReceiverError;

    async fn register_consumer<C>(&self, consumer: C) -> Result<(), Self::Error>
    where
        C: EventConsumer<T> + Send + Sync + 'static,
    {
        if let Some(channel) = self.inner.lock().await.take() {
            // Spin up a task to call into the consumer
            X::spawn_async(
                async move {
                    while let Ok(msg) = channel.recv_async().await {
                        // Send the message to the consumer, logging any errors
                        if let Err(e) = consumer.accept(msg).await {
                            error!(?e, "Error pushing message into consumer");
                        }
                    }
                }
                .instrument(info_span!("WrappedReceiver internal task")),
            );
            Ok(())
        } else {
            AlreadyRegisteredSnafu.fail()
        }
    }

    async fn register_callback<F>(
        &self,
        _callback: F,
        _token: CompletionToken,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(T) + Send + Sync + 'static,
    {
        CallbackSnafu.fail()
    }

    fn register_consumer_sync<C>(&self, consumer: C) -> Result<(), Self::Error>
    where
        C: EventConsumer<T> + Send + Sync + 'static,
    {
        if let Some(channel) = futures::executor::block_on(self.inner.lock()).take() {
            // Spin up a task to call into the consumer
            X::spawn_sync(move || {
                {
                    while let Ok(msg) = channel.recv() {
                        // Send the message to the consumer, logging any errors
                        if let Err(e) = consumer.accept_sync(msg) {
                            error!(?e, "Error pushing message into consumer");
                        }
                    }
                }
                .instrument(info_span!("WrappedReceiver internal task"))
            });
            Ok(())
        } else {
            AlreadyRegisteredSnafu.fail()
        }
    }

    fn register_callback_sync<F>(
        &self,
        _callback: F,
        _token: CompletionToken,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(T) + Send + Sync + 'static,
    {
        CallbackSnafu.fail()
    }
}

#[cfg(test)]
mod tests {
    use futures::channel::oneshot;

    use super::*;
    use crate::{
        executor::{Executor, Threads},
        traits::Actor,
        util::{AsyncActor, WrappedEvent},
    };

    // Dummy events for testing
    #[derive(Clone, PartialEq, Eq, Debug, PartialOrd, Ord, Hash, Default)]
    pub struct Output {
        val: usize,
    }

    // Basic functionality test for [`WrappedReceiver`]
    async fn smoke<X: Executor>() {
        // Create our channel
        let (tx, rx) = AsyncWrappedReceiver::<X, WrappedEvent<usize>>::new(Some(1));
        // Spawn up an actor
        let actor: AsyncActor<WrappedEvent<usize>, WrappedEvent<Output>, X> = AsyncActor::spawn(
            |mut value: usize, mut add: WrappedEvent<usize>| {
                // Grab the token, if there is one
                let token = add.token();
                let add = add.into_inner();
                // Do our addition
                value += add;
                // See if we need to return
                if let Some(token) = token {
                    let mut event: WrappedEvent<Output> = Output { val: value }.into();
                    event.set_completion_token(token);
                    (value, Some(event))
                } else {
                    (value, None)
                }
            },
            0,
            Some(1),
        );
        // Hookup the actor to the queue
        rx.register_consumer(actor.inbox().clone()).await.unwrap();

        // Feed in some numbers
        for _ in 0..100 {
            tx.send_async(1_usize.into()).await.unwrap();
        }
        // Send in a 0 to get the current value
        let (tx, rx) = oneshot::channel();
        let mut event: WrappedEvent<usize> = 0_usize.into();
        let token = event.tokenize().unwrap();
        // Register the callback
        actor
            .outbox()
            .register_callback(
                move |event| {
                    tx.send(event).unwrap();
                },
                token,
            )
            .await
            .unwrap();
        // Make the actor catchup
        actor.catchup().wait().await;
        // Send the event
        actor.inbox().accept(event).await.unwrap();
        // Await the result
        let res = rx.await.unwrap().into_inner();
        // With how this is written, using extreme backpressure, this should realistically always be
        // 100, but 99 is technically allowable too, the last two events, and only the last two
        // events, are allow to mix order here with both the bounds set to 1
        println!("res.val: {}", res.val);
        assert!(res.val == 98 || res.val == 99 || res.val == 100);
    }

    #[cfg(feature = "async-std")]
    #[async_std::test]
    async fn smoke_async_std() {
        smoke::<crate::executor::AsyncStd>().await;
    }

    #[async_std::test]
    async fn smoke_threads() {
        smoke::<Threads>().await;
    }
}
