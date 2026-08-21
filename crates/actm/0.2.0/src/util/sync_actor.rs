//! Basic actor implementation on top of flume queues

use std::{
    cell::RefCell,
    marker::PhantomData,
    sync::atomic::{AtomicU64, Ordering},
};

use async_trait::async_trait;
use flume::{Receiver, Sender};
use futures::future::join_all;
use once_cell::sync::Lazy;
use snafu::Snafu;
use tracing::{error, info, info_span, instrument, warn};

use super::TokenManager;
use crate::{
    executor::Executor,
    traits::{Actor, Event, EventConsumer, EventProducer},
    types::{CompletionToken, DynamicConsumer, DynamicError, Trigger, Waiter},
};

mod newtype_macro;

/// Error connecting to [`SyncActor`]
#[derive(Debug, Snafu)]
#[non_exhaustive]
pub enum SyncActorError {
    /// Background task shutdown
    Shutdown,
}

/// A basic actor that listens to its `Inbox` asynchronously on a background task
pub struct SyncActor<I: Event, O: Event, X: Executor> {
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
impl<X: Executor, I: Event, O: Event> Clone for SyncActor<I, O, X> {
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

impl<X: Executor, I: Event, O: Event> SyncActor<I, O, X> {
    /// Initializes this actor from a synchronous closure, spawning off the background thread or
    /// blocking task.
    ///
    /// The provided closure implements the business logic of the [`Actor`], processing inbound
    /// [`Event`]s, and optionally returning responding outbound [`Event`]s. If the event is a
    /// response to an incoming request, the [`CompletionToken`] must be included in the outbound
    /// [`Event`].
    ///
    /// This [`Actor`] runs either inside its own dedicated thread, or on the blocking thread pool,
    /// allowing long-running blocking operations to be performed.
    ///
    /// The provided `context` will be passed by mutable reference to the user provided closure
    /// every time it is invoked.
    ///
    /// If `limit` is `Some(_)`, then a bounded queue with the specified limit will be created,
    /// otherwise an unbounded queue will be used.
    #[instrument(skip(logic, context))]
    pub fn spawn<F, C>(logic: F, context: C, bound: Option<usize>) -> Self
    where
        F: FnMut(&mut C, I) -> Option<O> + Send + 'static,
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
        let (shutdown_tx, shutdown_rx) = flume::bounded(1);
        let (catchup_tx, catchup_rx) = flume::bounded(1);
        // Get a task id
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        info!(?id, "Spawning BasicActor task");
        // Spawn the background processing task that implements the actor
        X::spawn_sync(move || {
            info_span!(target: "SyncActor","SyncActor Inner Task", id = id);
            // Create the token manager
            // We will use this intercept all outbound
            let token_manager: RefCell<TokenManager<O>> = RefCell::new(TokenManager::new());
            // Convert channels into streams
            let inbox = inbox_rx;
            let consumers_inbox = consumers_rx;
            let callbacks = callbacks_rx;
            let shutdown_channel: Receiver<Trigger> = shutdown_rx;
            let catchup: Receiver<Trigger> = catchup_rx;
            // Store our consumers
            let consumers =
                RefCell::new(Vec::<Box<dyn EventConsumer<O, Error = DynamicError>>>::new());
            // Wrap the logic and context in a cell so we can borrow it multiple places
            let logic = RefCell::new(logic);
            let context = RefCell::new(context);
            // Enter our event handling loop
            let shutdown = RefCell::new(false);
            loop {
                if *shutdown.borrow() {
                    break;
                }
                // Select over our three inboxes. In each branch, we will log an error and break
                // out of the loop, implicitly closing down the task, if the other side of the
                // stream has been closed
                //
                // Because `select!` chooses between futures ready at the same time in a
                // ""semi-random"" way, this should give roughly equal attention to all the
                // inputs under load
                flume::Selector::new()
                    .recv(&inbox, |i| match i {
                        Ok(i) => {
                            if let Some(output) = logic.borrow_mut()(&mut context.borrow_mut(), i) {
                                // First process the event through the token manager, so any
                                // callbacks can be called back
                                let output = token_manager.borrow_mut().process(output);
                                // Then distribute it to all of our consumers, using some cheeky
                                // async here to send out to all the consumers concurrently
                                let results = futures::executor::block_on(join_all(
                                    consumers.borrow_mut().iter_mut().map(|x| async {
                                        x.accept(output.stateless_clone()).await
                                    }),
                                ));
                                // Log all of our errors
                                for result in results {
                                    if let Err(e) = result {
                                        warn!(?e, "Error occurred feeding event into consumer");
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            error!(?e, "Error receiving inbox message");
                            *shutdown.borrow_mut() = true;
                        }
                    })
                    .recv(&consumers_inbox, |c| match c {
                        Ok(c) => consumers.borrow_mut().push(c),
                        Err(e) => {
                            error!(?e, "Error receiving new consumer message");
                            *shutdown.borrow_mut() = true;
                        }
                    })
                    .recv(&callbacks, |c| match c {
                        Ok(c) => token_manager.borrow_mut().register_callback(c.1, c.0),
                        Err(e) => {
                            error!(?e, "Error receiving callback");
                            *shutdown.borrow_mut() = true;
                        }
                    })
                    .recv(&shutdown_channel, |c| {
                        if let Ok(c) = c {
                            let inbox = inbox.drain().collect::<Vec<_>>();
                            let consumers_inbox = consumers_inbox.drain().collect::<Vec<_>>();
                            let callbacks = callbacks.drain().collect::<Vec<_>>();
                            // Handle connecting the remaining callbacks and consumers first
                            for callback in callbacks {
                                token_manager
                                    .borrow_mut()
                                    .register_callback(callback.1, callback.0);
                            }
                            for consumer in consumers_inbox {
                                consumers.borrow_mut().push(consumer);
                            }
                            // Then handle any remaining events
                            for event in inbox {
                                if let Some(output) =
                                    logic.borrow_mut()(&mut context.borrow_mut(), event)
                                {
                                    // First process the event through the token manager, so any
                                    // callbacks can be called back
                                    let output = token_manager.borrow_mut().process(output);
                                    // Then distribute it to all of our consumers, using some cheeky
                                    // async here to send out to all the consumers concurrently
                                    let results = futures::executor::block_on(join_all(
                                        consumers.borrow_mut().iter_mut().map(|x| async {
                                            x.accept(output.stateless_clone()).await
                                        }),
                                    ));
                                    // Log all of our errors
                                    for result in results {
                                        if let Err(e) = result {
                                            warn!(?e, "Error occurred feeding event into consumer");
                                        }
                                    }
                                }
                            }
                            // Now we can shutdown the executor and signal completion
                            *shutdown.borrow_mut() = true;
                            c.trigger();
                        }
                    })
                    .recv(&catchup, |c| {
                        if let Ok(c) = c {
                            let inbox = inbox.drain().collect::<Vec<_>>();
                            let consumers_inbox = consumers_inbox.drain().collect::<Vec<_>>();
                            let callbacks = callbacks.drain().collect::<Vec<_>>();
                            // Handle connecting the remaining callbacks and consumers first
                            for callback in callbacks {
                                token_manager
                                    .borrow_mut()
                                    .register_callback(callback.1, callback.0);
                            }
                            for consumer in consumers_inbox {
                                consumers.borrow_mut().push(consumer);
                            }
                            // Then handle any remaining events
                            for event in inbox {
                                if let Some(output) =
                                    logic.borrow_mut()(&mut context.borrow_mut(), event)
                                {
                                    // First process the event through the token manager, so any
                                    // callbacks can be called back
                                    let output = token_manager.borrow_mut().process(output);
                                    // Then distribute it to all of our consumers, using some cheeky
                                    // async here to send out to all the consumers concurrently
                                    let results = futures::executor::block_on(join_all(
                                        consumers.borrow_mut().iter_mut().map(|x| async {
                                            x.accept(output.stateless_clone()).await
                                        }),
                                    ));
                                    // Log all of our errors
                                    for result in results {
                                        if let Err(e) = result {
                                            warn!(?e, "Error occurred feeding event into consumer");
                                        }
                                    }
                                }
                            }
                            // Now we can signal completion
                            c.trigger();
                        }
                    })
                    .wait();
            }
        });
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
impl<X: Executor, I: Event, O: Event> EventConsumer<I> for SyncActor<I, O, X> {
    type Error = SyncActorError;

    async fn accept(&self, event: I) -> Result<(), Self::Error> {
        self.inbox
            .send_async(event)
            .await
            .map_err(|_| SyncActorError::Shutdown)
    }
    fn accept_sync(&self, event: I) -> Result<(), Self::Error> {
        self.inbox.send(event).map_err(|_| SyncActorError::Shutdown)
    }
}

#[async_trait]
impl<X: Executor, I: Event, O: Event> EventProducer<O> for SyncActor<I, O, X> {
    type Error = SyncActorError;

    async fn register_consumer<C>(&self, consumer: C) -> Result<(), Self::Error>
    where
        C: EventConsumer<O> + Send + Sync + 'static,
    {
        self.consumers
            .send_async(Box::new(DynamicConsumer::from(consumer)))
            .await
            .map_err(|_| SyncActorError::Shutdown)
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
            .map_err(|_| SyncActorError::Shutdown)
    }
    fn register_consumer_sync<C>(&self, consumer: C) -> Result<(), Self::Error>
    where
        C: EventConsumer<O> + Send + Sync + 'static,
    {
        self.consumers
            .send(Box::new(DynamicConsumer::from(consumer)))
            .map_err(|_| SyncActorError::Shutdown)
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
            .map_err(|_| SyncActorError::Shutdown)
    }
}

impl<X: Executor, I: Event, O: Event> Actor<I, O, X> for SyncActor<I, O, X> {
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
    fn smoke<X: Executor>() {
        // Create our actor
        let actor: SyncActor<MathEvent, OutputEvent, X> = SyncActor::spawn(
            |value: &mut i64, mut math: MathEvent| {
                // Pull out the completion token, if there is any
                let token = math.token();
                // Perform the operation
                let old_value = *value;
                let math = math.into_inner();
                *value = math.operate(*value);
                // Check to see if there was a completion token, if so, send back an Output
                if let Some(token) = token {
                    // Make our output
                    let output = Output {
                        before: old_value,
                        after: *value,
                        input: math,
                    };
                    // Wrap it up
                    let mut output = OutputEvent::from(output);
                    // Attach the token
                    output.set_completion_token(token);
                    // Send it up
                    Some(output)
                } else {
                    None
                }
            },
            0,
            None,
        );
        println!("Created actor");
        // Hook up our collector
        let collector_out = actor.stream_sync(None).unwrap();
        println!("Hooked up collector");
        assert!(actor.catchup().wait_sync());
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
        println!("Pushed callbacks into actor");
        // Fill our actor's inbox with some threads
        for x in events {
            // Some cheeky cloning of the actor
            let actor = actor.clone();
            X::spawn_sync(move || actor.inbox().accept_sync(x).unwrap());
        }
        println!("Spawned actor filling threads");
        // Pull out of our collector and sort the results
        let mut collector_out: Vec<_> = collector_out.iter().take(100).collect();
        collector_out.sort();
        println!("Pulled out of collector");
        // Pull out of our callback stream and sort the results
        let mut callbacks: Vec<_> = rx.iter().take(100).collect();
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
    #[test]
    fn smoke_async_std() {
        smoke::<crate::executor::AsyncStd>();
    }

    #[test]
    fn smoke_threads() {
        smoke::<Threads>();
    }
}
