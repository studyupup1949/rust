//! Utility type for managing tokens when used for callback registration

use std::{
    collections::HashMap,
    sync::atomic::{AtomicU64, Ordering},
};

use itertools::{Either, Itertools};
use once_cell::sync::Lazy;
use tracing::{debug, error, instrument, trace, warn};

use crate::{traits::Event, types::CompletionToken};

/// Utility type for managing [`CompletionToken`]s used for callbacks when implementing an
/// [`Actor`](crate::traits::Actor)
///
/// It is best to try and avoid using this type directly, only using it if you need to implement an
/// [`Actor`](crate::traits::Actor) with custom event handling logic
pub struct TokenManager<T: Event> {
    /// Mapping of the completion token to the associated [`Event`]
    event_map: HashMap<CompletionToken, T>,
    /// Currently waiting callbacks
    callbacks: HashMap<CompletionToken, Box<dyn FnOnce(T) + Send + Sync>>,
    /// Unique id used for distinguishing between `TokenManager`s in logs, this value has no other
    /// use
    id: u64,
}

impl<T: Event> TokenManager<T> {
    /// Create a new `TokenManager` with an empty state
    #[instrument]
    pub fn new() -> Self {
        /// Counter to ensure unique ids for logging
        static COUNTER: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        debug!(?id, "Starting TokenManager");
        Self {
            id,
            event_map: HashMap::new(),
            callbacks: HashMap::new(),
        }
    }

    /// Process an [`Event`]
    ///
    /// If the [`Event`] has an associated [`CompletionToken`], it will be stripped off and the
    /// event will be copied, and the resulting pair will them either be shipped off to a registered
    /// callback, or placed into storage awaiting a matching callback's registration.
    ///
    /// If the [`Event`] has no associated [`CompletionToken`], it will be returned unmodified.
    #[instrument(skip(self),fields(id=self.id))]
    pub fn process(&mut self, mut event: T) -> T {
        match event.token() {
            Some(token) => {
                // Check to see if we already have a callback waiting for this event
                if let Some(callback) = self.callbacks.remove(&token) {
                    trace!(?token, "Calling callback");
                    callback(event.stateless_clone());
                } else {
                    trace!(?token, "Registering event");
                    self.event_map.insert(token, event.stateless_clone());
                }
                // We can just return the event, as the `token` method removes the existing token
                event
            }
            None => event,
        }
    }

    /// Register a callback
    ///
    /// This callback is expected to consume an [`Event`], and it will be called either immediately, if
    /// the associated [`Event`] is already in storage, or when that [`Event`] arrives.
    ///
    /// This callback is not guaranteed to be called, it may be garbage collected if the matching
    /// [`CompletionToken`] is dropped before the associated [`Event`] arrives.
    ///
    /// The callback is expected not to panic
    #[instrument(skip(self,callback), fields(id=self.id))]
    pub fn register_callback(
        &mut self,
        callback: impl FnOnce(T) + Send + Sync + 'static,
        token: CompletionToken,
    ) {
        // First check to see if the associated data is already registered
        if let Some(event) = self.event_map.remove(&token) {
            // In this case we can simply pass the event into the callback now, its already been
            // removed from the map
            trace!("Calling callback");
            callback(event);
        } else {
            // We don't already have the event, so go ahead and put this in the map where it can
            // await its friend
            trace!("Registering callback");
            self.callbacks.insert(token, Box::new(callback));
        }
    }

    /// Garbage collect the internal storage
    ///
    /// This will remove any pairs indexed by a [`CompletionToken`] whose partner has already been
    /// dropped, and finalize processing for any tokens where both pairs are already in the
    /// `TokenManager.
    ///
    /// Returns `true` if any items were garbage collected, false otherwise.
    ///
    /// This method is not automatically called, and must be called by the implementation when
    /// needed. It is possible to avoid needing to call this method through careful use of the API,
    /// but this is provided as a "backup solution" to rectify memory leaks for when such an issue
    /// would be too onerous to fix.
    #[instrument(skip(self),fields(id=self.id))]
    pub fn garbage_collect(&mut self) -> bool {
        let mut collected = 0_usize;
        // First, garbage collect the event_map
        let (garbage, rest): (Vec<_>, Vec<_>) =
            self.event_map.drain().partition(|(x, _)| x.count() < 2);
        // Drop the garbage
        for g in garbage {
            trace!(?g, "Garbage collecting event");
            collected += 1;
            std::mem::drop(g);
        }
        // Now partition out any events that are also present in the callbacks map
        let (stragglers, rest): (Vec<_>, HashMap<_, _>) =
            rest.into_iter().partition_map(|(x, y)| {
                if self.callbacks.contains_key(&x) {
                    Either::Left((x, y))
                } else {
                    Either::Right((x, y))
                }
            });
        // Now that we have the stragglers out, we can zip up the rest back into the map
        self.event_map = rest;
        // Now lets go ahead and remove the stragglers from the the call backs map and process them
        for (token, event) in stragglers {
            if let Some(callback) = self.callbacks.remove(&token) {
                // Error because this situation is indicative of a bug, but not a _serious_ one
                error!(?token, ?event, "Processing event in garbage collection.");
                // Call the callback!
                callback(event);
                // Increment the counter, these both count
                collected += 2;
            }
        }
        // Now time to garbage collect the callbacks map
        let (garbage, rest): (Vec<_>, HashMap<_, _>) =
            self.callbacks.drain().partition_map(|(x, y)| {
                if x.count() < 2 {
                    Either::Left((x, y))
                } else {
                    Either::Right((x, y))
                }
            });
        // Drop the garbage
        for (x, y) in garbage {
            trace!(?x, "Garbage collecting callback");
            collected += 1;
            std::mem::drop(x);
            std::mem::drop(y);
        }
        // Shove the rest back in there
        self.callbacks = rest;
        // Be noisy if we actually collected garbage
        if collected > 0 {
            warn!(
                ?collected,
                "Garbage was collected, a memory leak is happening!"
            );
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use proptest::prelude::*;
    use proptest_derive::Arbitrary;

    use super::*;
    use crate::util::WrappedEvent;

    // Make sure `TokenManager`'s id's are actually non equal
    #[test]
    fn token_manager_ids_nonequal() {
        let tkm_1: TokenManager<WrappedEvent<()>> = TokenManager::new();
        let tkm_2: TokenManager<WrappedEvent<()>> = TokenManager::new();
        assert_ne!(tkm_1.id, tkm_2.id);
    }

    // Make sure the TokenManager's processing works correctly
    #[test]
    fn token_manager_processing() {
        let mut tkm: TokenManager<WrappedEvent<usize>> = TokenManager::new();
        let (token_1, token_2) = CompletionToken::new();
        let mut event: WrappedEvent<usize> = 10.into();
        event.set_completion_token(token_1);
        // Setup a backchannel
        let cell_1: Arc<Mutex<Option<usize>>> = Arc::new(Mutex::new(None));
        let cell_2 = cell_1.clone();
        // Register the callback
        tkm.register_callback(
            move |event| {
                let mut value = cell_2.lock().unwrap();
                *value = Some(event.into_inner());
            },
            token_2,
        );
        // Make sure our value is still none
        assert!(cell_1.lock().unwrap().is_none());
        // Send some tokenless events
        for i in 0..10 {
            let x = tkm.process(i.into());
            std::mem::drop(x);
        }

        // Make sure our value is still none
        assert!(cell_1.lock().unwrap().is_none());
        // Send the event with the right token
        let _event = tkm.process(event);
        // Make sure our value is now Some
        assert!(cell_1.lock().unwrap().is_some());
    }

    // An arbitrary event type
    #[derive(Arbitrary, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum ArbEventType {
        Event1 { number: u64 },
        Event2 { letters: String },
    }

    // An arbitrary event, wrapped
    #[derive(Arbitrary, Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
    enum ArbEventWrapped {
        Tracked(ArbEventType),
        TrackedDropped(ArbEventType),
        Untracked(ArbEventType),
    }

    impl ArbEventWrapped {
        // Turn into an arb event
        fn into_event(self) -> Either<ArbEvent, (ArbEvent, CompletionToken)> {
            match self {
                ArbEventWrapped::Tracked(event) => {
                    let (token_1, token_2) = CompletionToken::new();
                    Either::Right((
                        ArbEvent {
                            event,
                            token: Some(token_1),
                            should_drop: false,
                        },
                        token_2,
                    ))
                }
                ArbEventWrapped::TrackedDropped(event) => {
                    let (token_1, token_2) = CompletionToken::new();
                    Either::Right((
                        ArbEvent {
                            event,
                            token: Some(token_1),
                            should_drop: true,
                        },
                        token_2,
                    ))
                }
                ArbEventWrapped::Untracked(event) => Either::Left(ArbEvent {
                    event,
                    token: None,
                    should_drop: false,
                }),
            }
        }
    }

    // An arbitrary event
    #[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
    struct ArbEvent {
        event: ArbEventType,
        token: Option<CompletionToken>,
        should_drop: bool,
    }

    impl Event for ArbEvent {
        type Flags = ();

        fn flags(&self) -> Self::Flags {}

        fn stateless_clone(&self) -> Self {
            Self {
                event: self.event.clone(),
                token: None,
                should_drop: self.should_drop,
            }
        }

        fn token(&mut self) -> Option<CompletionToken> {
            self.token.take()
        }
    }

    // Make sure garbage collection is working at least somewhat properly
    fn token_manager_garbage_inner(events: Vec<ArbEventWrapped>) {
        // Setup our token manager
        let mut tkm: TokenManager<ArbEvent> = TokenManager::new();
        // Setup our storage
        let mut kept = Vec::new();
        let mut dropped = Vec::new();
        let mut untracked = Vec::new();
        // Process our events
        for event in events {
            match event.into_event() {
                Either::Left(event) => untracked.push(tkm.process(event)),
                Either::Right((event, token)) => {
                    if event.should_drop {
                        dropped.push((tkm.process(event), token));
                    } else {
                        kept.push((tkm.process(event), token));
                    }
                }
            }
        }
        let dropped_len = dropped.len();
        let empty_events = dropped.is_empty();
        // the events map should contain as many things as there are dropped + kept entries
        assert!(tkm.event_map.len() == dropped_len + kept.len());
        // Drop the dropped events, this shouldn't affect the count yet
        std::mem::drop(dropped);
        assert!(tkm.event_map.len() == dropped_len + kept.len());
        // Do garbage collection, this should return true, since we dropped things
        let collected = tkm.garbage_collect();
        // This will always be false if there are no events
        assert!(collected || empty_events);
        // The count should equal the number of kept items now
        assert!(tkm.event_map.len() == kept.len());
        // Go through and make sure all of our kept items have call backs
        let kept_len = kept.len();
        let count: Arc<Mutex<usize>> = Arc::new(Mutex::new(0));
        for (event_1, token) in kept {
            let count = count.clone();
            tkm.register_callback(
                move |event_2| {
                    assert_eq!(event_1, event_2);
                    let mut count_ref = count.lock().unwrap();
                    *count_ref += 1;
                },
                token,
            );
        }
        // Make sure the correct thing happened
        assert_eq!(*count.lock().unwrap(), kept_len);
        assert!(tkm.event_map.is_empty());
    }

    proptest! {
        #[test]
        fn token_manager_garbage(
            events in any::<Vec<ArbEventWrapped>>()
        ) {
            token_manager_garbage_inner(events);
        }
    }
}
