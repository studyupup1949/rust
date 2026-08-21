//! Basic actor implementation on top of flume queues

use std::{
    future::Future,
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering},
};

use async_trait::async_trait;
use flume::Sender;
use futures::{StreamExt, future::join_all, select};
use once_cell::sync::Lazy;
use snafu::Snafu;
use tracing::{Instrument, error, info, info_span, instrument, warn};

use super::TokenManager;
use crate::{
    executor::Executor,
    traits::{Actor, Event, EventConsumer, EventProducer},
    types::{CompletionToken, DynamicConsumer, DynamicError, Trigger, Waiter},
};

mod newtype_macro;

/// Error connecting to [`AsyncActor`]
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum AsyncActorError {
    /// Background task shutdown
    Shutdown,
}

/// A basic actor that listens to its `Inbox` asynchronously on a background task
pub struct AsyncActor<I: Event, O: Event, X: Executor> {
    /// A channel into our task
    inbox: Sender<I>,
    /// A channel to register consumers over
    consumers: Sender<Box<dyn EventConsumer<O, Error = DynamicError>>>,
    /// A channel to register callbacks over
    #[allow(clippy::type_complexity)] // This type isn't actually that complex
    callbacks: Sender<(CompletionToken, Box<dyn FnOnce(O) + Send + Sync + 'static>)>,
    /// A channel for shutdown triggers
    shutdown: Sender<Trigger>,
    /// A channel for catch up triggers
    catchup: Sender<Trigger>,
    /// Phantom for the executor type
    _executor: PhantomData<X>,
}

// This clone needs to be explicit, as the derive macro for clone adds a requirement for the
// struct's generics to be clone, which we can't do here as not all events are clone
impl<X: Executor, I: Event, O: Event> Clone for AsyncActor<I, O, X> {
    fn clone(&self) -> Self {
        Self {
            inbox: self.inbox.clone(),
            consumers: self.consumers.clone(),
            callbacks: self.callbacks.clone(),
            shutdown: self.shutdown.clone(),
            catchup: self.catchup.clone(),
            _executor: PhantomData,
        }
    }
}

impl<X: Executor, I: Event, O: Event> AsyncActor<I, O, X> {
    /// Initializes this actor from a synchronous closure, spawning off the background task
    ///
    /// The provided closure implements the business logic of the [`Actor`], processing inbound
    /// [`Event`]s, and optionally returning responding outbound [`Event`]s. If the event is a
    /// response to an incoming request, the [`CompletionToken`] must be included in the outbound
    /// [`Event`].
    ///
    /// While the user provided closure is allowed to do actions that take long amounts of time to
    /// complete, you are encouraged to make use of the blocking thread pool.
    ///
    /// The provided `context` will be passed by mutable reference to the user provided closure
    /// every time it is invoked.
    ///
    /// If `limit` is `Some(_)`, then a bounded queue with the specified limit will be created,
    /// otherwise an unbounded queue will be used.
    #[instrument(skip(logic, context))]
    pub fn spawn<F, C>(logic: F, context: C, bound: Option<usize>) -> Self
    where
        F: Fn(C, I) -> (C, Option<O>) + Send + Sync + 'static,
        C: Send + Sync + 'static,
    {
        Self::spawn_async(
            move |a, b| {
                let result = logic(a, b);
                async move { result }
            },
            context,
            bound,
        )
    }

    /// Initializes this actor from a closure returning a future (an "async closure"), spawning
    /// off the background task.
    ///
    /// Otherwise behaves identically to [`AsyncActor::spawn`].
    #[instrument(skip(logic, context))]
    pub fn spawn_async<R, F, C>(logic: F, mut context: C, bound: Option<usize>) -> Self
    where
        R: Future<Output = (C, Option<O>)> + Send,
        F: Fn(C, I) -> R + Send + Sync + 'static,
        C: Send + Sync + 'static,
    {
        /// Counter for generating logging ids
        static COUNTER: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
        // Create our streams
        let (inbox_tx, inbox_rx): (Sender<I>, _) = match bound {
            Some(x) => flume::bounded(x),
            None => flume::unbounded(),
        };
        let (consumers_tx, consumers_rx): (
            Sender<Box<dyn EventConsumer<O, Error = DynamicError>>>,
            _,
        ) = match bound {
            Some(x) => flume::bounded(x),
            None => flume::unbounded(),
        };
        #[allow(clippy::type_complexity)]
        let (callbacks_tx, callbacks_rx): (
            Sender<(CompletionToken, Box<dyn FnOnce(O) + Send + Sync + 'static>)>,
            _,
        ) = match bound {
            Some(x) => flume::bounded(x),
            None => flume::unbounded(),
        };
        let (shutdown_tx, shutdown_rx) = flume::bounded::<Trigger>(1);
        let (catchup_tx, catchup_rx) = flume::bounded::<Trigger>(1);
        // Get a task id
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        info!(?id, "Spawning BasicActor task");
        // Spawn the background processing task that implements the actor
        X::spawn_async(
            async move {
                // Create the token manager
                // We will use this intercept all outbound
                let mut token_manager: TokenManager<O> = TokenManager::new();
                // Convert channels into streams
                let mut inbox_stream = inbox_rx.clone().into_stream();
                let mut consumers_inbox_stream = consumers_rx.clone().into_stream();
                let mut callbacks_stream = callbacks_rx.clone().into_stream();
                let mut shutdown_stream = shutdown_rx.into_stream();
                let mut catchup_stream = catchup_rx.into_stream();
                // Store our consumers
                let mut consumers = Vec::<Box<dyn EventConsumer<O, Error = DynamicError>>>::new();
                // Enter our event handling loop
                loop {
                    // Check for a shutdown signal
                    // Select over our three inboxes. In each branch, we will log an error and break
                    // out of the loop, implicitly closing down the task, if the other side of the
                    // stream has been closed
                    //
                    // Because `select!` chooses between futures ready at the same time in a
                    // ""semi-random"" way, this should give roughly equal attention to all the
                    // inputs under load
                    select! {
                        x = inbox_stream.next() => {
                            if let Some(x) = x {
                                // Process the event with the caller provided closure and context,
                                // and then perform bookkeeping if the closure returns an outbound
                                // event
                                let (new_context, output) = logic(context, x).await;
                                context = new_context;
                                if let Some(output) = output {
                                    // First process the event through the token manager, so any
                                    // callbacks can be called back
                                    let output = token_manager.process(output);
                                    // Then distribute it to all of our consumers
                                    let results = join_all(
                                        consumers
                                            .iter_mut()
                                            .map(|x| async { x.accept(output.stateless_clone()).await})
                                    ).await;
                                    // Log all of our errors
                                    for result in results {
                                        if let Err(e) = result {
                                            warn!(?e, "Error occurred feeding event into consumer");
                                        }
                                    }
                                }
                            } else {
                                error!("Inbox channel unexpectedly shutdown");
                                break;
                            }
                        }
                        x = consumers_inbox_stream.next() => {
                            if let Some(x) = x {
                                // Add the new consumer to the list
                                consumers.push(x);
                            } else {
                                error!("Consumers channel unexpectedly shutdown");
                                break;
                            }
                        }
                        x = callbacks_stream.next() => {
                            if let Some(x) = x {
                                // Register the callback with the token manager
                                token_manager.register_callback(x.1, x.0);
                            } else {
                                error!("Callbacks channel unexpectedly shutdown");
                                break;
                            }
                        }
                        shutdown = shutdown_stream.next() => {
                            if let Some(shutdown) = shutdown {
                                // Drain the queues
                                let inbox = inbox_rx.drain().collect::<Vec<_>>();
                                let consumers_inbox = consumers_rx.drain().collect::<Vec<_>>();
                                let callbacks = callbacks_rx.drain().collect::<Vec<_>>();
                                // Handle connecting the remaining callbacks and consumers first
                                for callback in callbacks {
                                    token_manager.register_callback(callback.1, callback.0);
                                }
                                for consumer in consumers_inbox {
                                    consumers.push(consumer);
                                }
                                // Then handle any remaining events
                                for event in inbox {
                                    let (new_context, output) = logic(context, event).await;
                                    context = new_context;
                                    if let Some(output) = output {
                                        // First process the event through the token manager, so any
                                        // callbacks can be called back
                                        let output = token_manager.process(output);
                                        // Then distribute it to all of our consumers
                                        let results = join_all(
                                            consumers
                                                .iter_mut()
                                                .map(|x| async { x.accept(output.stateless_clone()).await})
                                        ).await;
                                        // Log all of our errors
                                        for result in results {
                                            if let Err(e) = result {
                                                warn!(?e, "Error occurred feeding event into consumer");
                                            }
                                        }
                                    }

                                }
                                // Tell the caller we are done
                                shutdown.trigger();
                                break;
                            }
                        }
                        catchup = catchup_stream.next() => {
                            if let Some(catchup) = catchup {
                                // Drain the queues
                                let inbox = inbox_rx.drain().collect::<Vec<_>>();
                                let consumers_inbox = consumers_rx.drain().collect::<Vec<_>>();
                                let callbacks = callbacks_rx.drain().collect::<Vec<_>>();
                                // Handle connecting the remaining callbacks and consumers first
                                for callback in callbacks {
                                    token_manager.register_callback(callback.1, callback.0);
                                }
                                for consumer in consumers_inbox {
                                    consumers.push(consumer);
                                }
                                // Then handle any remaining events
                                for event in inbox {
                                    let (new_context, output) = logic(context, event).await;
                                    context = new_context;
                                    if let Some(output) = output {
                                        // First process the event through the token manager, so any
                                        // callbacks can be called back
                                        let output = token_manager.process(output);
                                        // Then distribute it to all of our consumers
                                        let results = join_all(
                                            consumers
                                                .iter_mut()
                                                .map(|x| async { x.accept(output.stateless_clone()).await})
                                        ).await;
                                        // Log all of our errors
                                        for result in results {
                                            if let Err(e) = result {
                                                warn!(?e, "Error occurred feeding event into consumer");
                                            }
                                        }
                                    }

                                }
                                // Tell the caller we are done
                                catchup.trigger();
                            }
                        }
                        complete => {
                            break;
                        }
                    }
                }
            }
            .instrument(info_span!(target: "AsyncActor","AsyncActor Inner Task", id = id)),
        );

        Self {
            inbox: inbox_tx,
            consumers: consumers_tx,
            callbacks: callbacks_tx,
            shutdown: shutdown_tx,
            catchup: catchup_tx,
            _executor: PhantomData,
        }
    }
}

#[async_trait]
impl<X: Executor, I: Event, O: Event> EventConsumer<I> for AsyncActor<I, O, X> {
    type Error = AsyncActorError;

    async fn accept(&self, event: I) -> Result<(), Self::Error> {
        self.inbox
            .send_async(event)
            .await
            .map_err(|_| AsyncActorError::Shutdown)
    }

    fn accept_sync(&self, event: I) -> Result<(), Self::Error> {
        self.inbox
            .send(event)
            .map_err(|_| AsyncActorError::Shutdown)
    }
}

#[async_trait]
impl<X: Executor, I: Event, O: Event> EventProducer<O> for AsyncActor<I, O, X> {
    type Error = AsyncActorError;

    async fn register_consumer<C>(&self, consumer: C) -> Result<(), Self::Error>
    where
        C: EventConsumer<O> + Send + Sync + 'static,
    {
        self.consumers
            .send_async(Box::new(DynamicConsumer::from(consumer)))
            .await
            .map_err(|_| AsyncActorError::Shutdown)
    }

    async fn register_callback<F>(
        &self,
        callback: F,
        token: CompletionToken,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(O) + Send + Sync + 'static,
    {
        self.callbacks
            .send_async((token, Box::new(callback)))
            .await
            .map_err(|_| AsyncActorError::Shutdown)
    }

    fn register_consumer_sync<C>(&self, consumer: C) -> Result<(), Self::Error>
    where
        C: EventConsumer<O> + Send + Sync + 'static,
    {
        self.consumers
            .send(Box::new(DynamicConsumer::from(consumer)))
            .map_err(|_| AsyncActorError::Shutdown)
    }

    fn register_callback_sync<F>(
        &self,
        callback: F,
        token: CompletionToken,
    ) -> Result<(), Self::Error>
    where
        F: FnOnce(O) + Send + Sync + 'static,
    {
        self.callbacks
            .send((token, Box::new(callback)))
            .map_err(|_| AsyncActorError::Shutdown)
    }
}

impl<X: Executor, I: Event, O: Event> Actor<I, O, X> for AsyncActor<I, O, X> {
    type Inbox = Self;
    type Outbox = Self;

    fn inbox(&self) -> &Self::Inbox {
        self
    }

    fn outbox(&self) -> &Self::Outbox {
        self
    }

    fn shutdown(&self) -> Waiter {
        let (waiter, trigger) = Waiter::new();
        let _res = self.shutdown.send(trigger);
        waiter
    }
    fn catchup(&self) -> Waiter {
        let (waiter, trigger) = Waiter::new();
        let _res = self.catchup.send(trigger);
        waiter
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        executor::Threads,
        testing_util::{Add, Math, MathEvent, MathEventType, Output, OutputEvent},
        traits::ActorExt,
    };
    // Basic smoke test, increment the counter by one a few times
    async fn smoke<X: Executor>() {
        // Create our actor
        let actor: AsyncActor<MathEvent, OutputEvent, X> = AsyncActor::spawn(
            |mut value: i64, mut math: MathEvent| {
                // Pull out the completion token, if there is any
                let token = math.token();
                // Perform the operation
                let old_value = value;
                let math = math.into_inner();
                value = math.operate(value);
                // Check to see if there was a completion token, if so, send back an Output
                if let Some(token) = token {
                    // Make our output
                    let output = Output {
                        before: old_value,
                        after: value,
                        input: math,
                    };
                    // Wrap it up
                    let mut output = OutputEvent::from(output);
                    // Attach the token
                    output.set_completion_token(token);
                    // Send it up
                    (value, Some(output))
                } else {
                    (value, None)
                }
            },
            0,
            None,
        );
        // Create a channel to collect our outputs
        let (tx, rx) = flume::unbounded();
        // Create 100 individual tasks that each add 1 to the actor
        let events = join_all(
            (0..100)
                // Create 100 individual tasks
                .map(|_| {
                    let event_type = MathEventType::Add(Add(1));
                    let mut event = MathEvent::from(event_type);
                    let token = event.tokenize().unwrap();
                    (event, token)
                })
                // Register the callbacks
                .map(|(event, token)| {
                    // Make a copy of our stream input
                    let tx = tx.clone();
                    // Clone the actor, cheating a bit, a real application wouldn't be spawning tasks
                    // like this
                    let actor = actor.clone();
                    async move {
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
                        // Return the event
                        event
                    }
                }),
        )
        .await;
        // Hook up our collector
        let collector_out = actor.stream(None).await.unwrap();
        // Spawn up some tasks to fill our actor's inbox
        let _tasks = join_all(events.into_iter().map(|x| {
            // Same cheeky cloning of the actor
            let actor = actor.clone();
            async move {
                actor.inbox().accept(x).await.unwrap();
            }
        }))
        .await;
        // Pull out of our collector and sort the results
        let mut collector_out: Vec<_> = collector_out.into_stream().take(100).collect().await;
        collector_out.sort();
        // Pull out of our callback stream and sort the results
        let mut callbacks: Vec<_> = rx.into_stream().take(100).collect().await;
        callbacks.sort();
        // The should be equal
        assert_eq!(collector_out, callbacks);
        // Generate the expected list
        let expected: Vec<OutputEvent> = (0..100)
            .map(|x| {
                OutputEvent::from(Output {
                    before: x,
                    after: x + 1,
                    input: MathEventType::Add(Add(1)),
                })
            })
            .collect();
        // Double check equality
        assert_eq!(collector_out, expected);
    }

    #[cfg(feature = "async-std")]
    #[async_std::test]
    async fn smoke_async_std() {
        smoke::<crate::executor::AsyncStd>().await;
    }

    #[async_std::test]
    async fn smoke_threads_async() {
        smoke::<Threads>().await;
    }

    #[test]
    fn smoke_threads_sync() {
        // Create our actor
        let actor: AsyncActor<MathEvent, OutputEvent, Threads> = AsyncActor::spawn(
            |mut value: i64, mut math: MathEvent| {
                // Pull out the completion token, if there is any
                let token = math.token();
                // Perform the operation
                let old_value = value;
                let math = math.into_inner();
                value = math.operate(value);
                // Check to see if there was a completion token, if so, send back an Output
                if let Some(token) = token {
                    // Make our output
                    let output = Output {
                        before: old_value,
                        after: value,
                        input: math,
                    };
                    // Wrap it up
                    let mut output = OutputEvent::from(output);
                    // Attach the token
                    output.set_completion_token(token);
                    // Send it up
                    (value, Some(output))
                } else {
                    (value, None)
                }
            },
            0,
            None,
        );
        // Create a channel to collect our outputs
        let (tx, rx) = flume::unbounded();
        // Create 100 individual tasks that each add 1 to the actor
        let mut events = vec![];
        for _ in 0..100 {
            // Create 100 individual tasks
            let event_type = MathEventType::Add(Add(1));
            let mut event = MathEvent::from(event_type);
            let token = event.tokenize().unwrap();
            let tx = tx.clone();
            // Register the callback
            actor
                .outbox()
                .register_callback_sync(
                    move |event| {
                        tx.send(event).unwrap();
                    },
                    token,
                )
                .unwrap();
            // Return the event
            events.push(event);
        }
        // Hook up our collector
        let collector_out = actor.stream_sync(None).unwrap();
        // Fill our actor's inbox with some threads
        for x in events {
            // Some cheeky cloning of the actor
            let actor = actor.clone();
            std::thread::spawn(move || actor.inbox().accept_sync(x).unwrap());
        }
        // Pull out of our collector and sort the results
        let mut collector_out: Vec<_> = collector_out.into_iter().take(100).collect();
        collector_out.sort();
        // Pull out of our callback stream and sort the results
        let mut callbacks: Vec<_> = rx.into_iter().take(100).collect();
        callbacks.sort();
        // The should be equal
        assert_eq!(collector_out, callbacks);
        // Generate the expected list
        let expected: Vec<OutputEvent> = (0..100)
            .map(|x| {
                OutputEvent::from(Output {
                    before: x,
                    after: x + 1,
                    input: MathEventType::Add(Add(1)),
                })
            })
            .collect();
        // Double check equality
        assert_eq!(collector_out, expected);
    }
}
