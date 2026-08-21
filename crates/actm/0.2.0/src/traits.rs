//! Traits used by the [`Event`] Module
use std::{convert::Infallible, error::Error, fmt::Debug, hash::Hash};

use async_trait::async_trait;
use flume::{Receiver, Sender};
use futures::channel::oneshot;
use snafu::ResultExt;
use tracing::{error, instrument, warn};

use super::types::CompletionToken;
use crate::{
    executor::Executor,
    types::Waiter,
    util::either_error::{EitherError, LeftSnafu, RightSnafu},
};

/// The basic, individual message that are passed between components in platform, these are intended
/// to be fairly 'dumb' objects, mostly being data only.
///
/// The `Flags` associated type and `flags` method is intended to allow `Event`s to provide, in a
/// generic way, a method for pre-computing some commonly used flags for passing to potentially many
/// filters.
pub trait Event: Eq + Debug + Ord + Hash + Send + Sync + 'static {
    /// A collection of precomputed values that filters can use to avoid heavy computation.
    type Flags: Clone + Eq + Debug + Ord + Hash + Send + Sync + 'static;
    /// Precompute or retrieve from cache values commonly checked by filters, to allow the
    /// potentially many filters from needing to individually repeat those computations itself.
    ///
    /// This method is allowed to block, and should always be run on the blocking thread pool.
    fn flags(&self) -> Self::Flags;
    /// This method returns the associated [`CompletionToken`], if the event is associated with one.
    /// In order to comply with the [`CompletionToken`] contract, this method **must** move the
    /// existing [`CompletionToken`] out of the `Event`, and **must not** copy it.
    ///
    /// If the `Event` does not have an associated token, this method should not alter the event in
    /// any way.
    fn token(&mut self) -> Option<CompletionToken> {
        None
    }
    /// Sets a [`CompletionToken`] on this [`Event`], returning the other half of the new completion
    /// token if the call was successful
    fn tokenize(&mut self) -> Option<CompletionToken> {
        None
    }

    /// Makes a copy of this `Event` without copying over any non-cloneable state, such as a
    /// [`CompletionToken`]
    ///
    /// This method is used in place of the [`Clone`] trait, as we would like for `Event`'s to be
    /// able to contain non-cloneable data, especially [`CompletionToken`]s, that don't preserve
    /// meaning across different executions of the program, and for which a sensible clone of the
    /// `Event`'s data can be made lacking the non-cloneable portion.'
    #[must_use]
    fn stateless_clone(&self) -> Self;
}

/// Stub implementation of [`Event`] for `()` to make some things more ergonomic to write
impl Event for () {
    type Flags = ();

    fn flags(&self) -> Self::Flags {}

    fn stateless_clone(&self) -> Self {}
}

/// The inbox side of an [`Actor`]
#[async_trait]
pub trait EventConsumer<E: Event>: Send + Sync + 'static {
    /// The error type for this event consumer
    type Error: Error + Send + Sync + Sized + 'static;
    /// Accepts an event
    ///
    /// This method can run long running computation, but is strongly encouraged to properly utilize
    /// the blocking thread pool.
    ///
    /// # Errors
    ///
    /// Will return an error if communication with the actor fails
    async fn accept(&self, event: E) -> Result<(), Self::Error>;

    /// Accepts an event, blocking if needed
    /// # Errors
    ///
    /// Will return an error if communication with the actor fails
    fn accept_sync(&self, event: E) -> Result<(), Self::Error>;
}

#[async_trait]
impl<E: Event, F, H> EventConsumer<E> for (F, H)
where
    F: Fn(&E::Flags, &E) -> bool + Send + Sync + 'static,
    H: Fn(E) + Send + Sync + 'static,
{
    type Error = Infallible;
    async fn accept(&self, event: E) -> Result<(), Self::Error> {
        self.1(event);
        Ok(())
    }

    fn accept_sync(&self, event: E) -> Result<(), Self::Error> {
        self.1(event);
        Ok(())
    }
}

#[async_trait]
impl<E: Event> EventConsumer<E> for Sender<E> {
    type Error = flume::SendError<E>;
    async fn accept(&self, event: E) -> Result<(), Self::Error> {
        self.send_async(event).await
    }
    fn accept_sync(&self, event: E) -> Result<(), Self::Error> {
        self.send(event)
    }
}

/// Defines the outline of an [`Event`] producer, the outbox side of an [`Actor`]
///
/// An implementation is also provided for a tuple of a filtering function and a processing
/// function, however, it is advised to not run long running computations inside the processing
/// function with this implementation
#[async_trait]
pub trait EventProducer<E: Event>: Send + Sync + 'static {
    /// The error type for this event producer
    type Error: Error + Send + Sync + 'static;
    /// Registers an event consumer with the producer
    ///
    /// # Errors
    ///
    /// Can return an implementation defined error if registering the consumer fails
    async fn register_consumer<C>(&self, consumer: C) -> Result<(), Self::Error>
    where
        C: EventConsumer<E> + Send + Sync + 'static;
    /// Registers a callback to be called on the completion of the computation associated with a
    /// [`CompletionToken`]
    /// # Errors
    ///
    /// Can return an implementation defined error if registering the consumer fails
    async fn register_callback<F>(
        &self,
        callback: F,
        token: CompletionToken,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(E) + Send + Sync + 'static;

    /// Registers an event consumer with the producer
    ///
    /// Blocks if required
    ///
    /// # Errors
    ///
    /// Can return an implementation defined error if registering the consumer fails
    fn register_consumer_sync<C>(&self, consumer: C) -> Result<(), Self::Error>
    where
        C: EventConsumer<E> + Send + Sync + 'static;
    /// Registers a callback to be called on the completion of the computation associated with a
    /// [`CompletionToken`]
    ///
    /// Blocks if required
    ///
    /// # Errors
    ///
    /// Can return an implementation defined error if registering the consumer fails
    fn register_callback_sync<F>(
        &self,
        callback: F,
        token: CompletionToken,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(E) + Send + Sync + 'static;
}

/// Defines what an `Actor` is
///
/// An `Actor` is defined by having a shareable `Inbox` connection to send messages too, as well as
/// being able to hook into the `Actor` to process its outbound messages
///
/// The `I` type argument is the type of [`Event`] this `Actor` accepts through its inbox, and the `O` type argument
/// is the type of [`Event`] this `Actor` emits through its outbox.
pub trait Actor<I: Event, O: Event, E: Executor>: Send + Sync + 'static {
    /// The type that this `Actor` uses for its inbox
    type Inbox: EventConsumer<I> + 'static;
    /// The type that this `Actor` uses for subscribing to outbound messages
    type Outbox: EventProducer<O> + 'static;

    /// Returns a new handle to the associated `Inbox` for this `Actor`, this should be owned so it
    /// can be easily passed around between threads
    fn inbox(&self) -> &Self::Inbox;

    /// Returns a new handle to the associated `Outbox` for this `Actor`, this should be owned so it
    /// can be easily passed around between threads
    fn outbox(&self) -> &Self::Outbox;

    /// Shutdown the `Actor`, waiting for all currently in flight events to be processed, and return
    /// a [`Waiter`] that will be signaled when the `Actor` is shutdown
    fn shutdown(&self) -> Waiter;

    /// Process all currently in flight events in a batch, and return a [`Waiter`] that will be
    /// signaled when the `Actor` is caught up
    fn catchup(&self) -> Waiter;
}

/// Extension trait providing utility methods for [`Actor`]
#[async_trait]
pub trait ActorExt<I: Event, O: Event, E: Executor>: Actor<I, O, E> {
    /// Takes the given event, tokenizes it, sends it into the [`Actor`], and setups a callback to
    /// retrieve the result value.
    ///
    /// Will return `None` in the event that the attempt to tokenize the event fails
    ///
    /// This method is provided for ergonomically writing tests, top-level access to [`Actor`]s and
    /// other simple/quick use-cases, it is not advisable to use this method from inside the inner
    /// task of another [`Actor`].
    ///
    /// This variant of the method asynchronously subscribes to the [`Actor`].
    #[instrument(skip(self))]
    async fn call(
        &self,
        mut event: I,
    ) -> Result<
        Option<O>,
        EitherError<
            <Self::Inbox as EventConsumer<I>>::Error,
            <Self::Outbox as EventProducer<O>>::Error,
        >,
    > {
        // Get our tokens
        if let Some(token) = event.tokenize() {
            // Get a queue to communicate over
            let (tx, rx) = oneshot::channel();
            // Register the callback
            self.outbox()
                .register_callback(
                    move |event| {
                        // Explicitly ignore error
                        let _res = tx.send(event);
                    },
                    token,
                )
                .await
                .context(RightSnafu)?;
            // Send the event
            self.inbox().accept(event).await.context(LeftSnafu)?;
            // Await the result
            match rx.await {
                Ok(x) => Ok(Some(x)),
                Err(e) => {
                    error!(?e, "Failed to await returned event");
                    Ok(None)
                }
            }
        } else {
            // Event was not tokenizeable
            warn!("Event was not tokenizeable");
            Ok(None)
        }
    }

    /// Takes the given event, tokenizes it, sends it into the [`Actor`], and setups a callback to
    /// retrieve the result value.
    ///
    /// Will return `None` in the event that the attempt to tokenize the event fails
    ///
    /// This method is provided for ergonomically writing tests, top-level access to [`Actor`]s and
    /// other simple/quick use-cases, it is not advisable to use this method from inside the inner
    /// task of another [`Actor`].
    ///
    /// This variant of the method synchronously subscribes to the [`Actor`].
    #[instrument(skip(self))]
    #[allow(clippy::type_complexity)]
    fn call_sync(
        &self,
        mut event: I,
    ) -> Result<
        Option<O>,
        EitherError<
            <Self::Inbox as EventConsumer<I>>::Error,
            <Self::Outbox as EventProducer<O>>::Error,
        >,
    > {
        // Get our tokens
        if let Some(token) = event.tokenize() {
            // Get a queue to communicate over
            let (tx, rx) = flume::bounded(1);
            // Register the callback
            self.outbox()
                .register_callback_sync(
                    move |event| {
                        // Explicitly ignore error
                        let _res = tx.send(event);
                    },
                    token,
                )
                .context(RightSnafu)?;
            // Send the event
            self.inbox().accept_sync(event).context(LeftSnafu)?;
            // Await the result
            match rx.recv() {
                Ok(x) => Ok(Some(x)),
                Err(e) => {
                    error!(?e, "Failed to await returned event");
                    Ok(None)
                }
            }
        } else {
            // Event was not tokenizeable
            warn!("Event was not tokenizeable");
            Ok(None)
        }
    }

    /// Provides a [`Receiver`] that forwards the outbox of this [`Actor`].
    ///
    /// The bounding behavior of this stream can be specified with the provided argument, with
    /// `None` sepecifying an unbounded queue. Be advised that setting a bound here will provide
    /// backpressure to the actor.
    ///
    /// This variant of the method asynchronously subscribes to the [`Actor`]
    #[instrument(skip(self))]
    async fn stream(
        &self,
        bound: Option<usize>,
    ) -> Result<Receiver<O>, <Self::Outbox as EventProducer<O>>::Error> {
        // Create the channel
        let (i, o) = match bound {
            Some(bound) => flume::bounded(bound),
            None => flume::unbounded(),
        };
        // Register the consuming side
        self.outbox().register_consumer(i).await?;
        // Return the receiving side
        Ok(o)
    }

    /// Provides a [`Receiver`] that forwards the outbox of this [`Actor`].
    ///
    /// The bounding behavior of this stream can be specified with the provided argument, with
    /// `None` sepecifying an unbounded queue. Be advised that setting a bound here will provide
    /// backpressure to the actor.
    ///
    /// This variant of the method synchronously subscribes to the [`Actor`]
    #[instrument(skip(self))]
    fn stream_sync(
        &self,
        bound: Option<usize>,
    ) -> Result<Receiver<O>, <Self::Outbox as EventProducer<O>>::Error> {
        // Create the channel
        let (i, o) = match bound {
            Some(bound) => flume::bounded(bound),
            None => flume::unbounded(),
        };
        // Register the consuming side
        self.outbox().register_consumer_sync(i)?;
        // Return the receiving side
        Ok(o)
    }
}

impl<I: Event, O: Event, E: Executor, T: Actor<I, O, E>> ActorExt<I, O, E> for T {}
