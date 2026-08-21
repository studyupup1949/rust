//! Types used by the event interface

use std::{
    error::Error,
    fmt::Debug,
    hash::Hash,
    marker::PhantomData,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
};

use async_trait::async_trait;
use once_cell::sync::Lazy;

use super::traits::{Event, EventConsumer};

mod waiter;

pub use waiter::{Trigger, Waiter};

/// Utility type for generalized errors
pub enum Either<A, B> {
    /// The happy path or preferred option
    Happy(A),
    /// The sad path or error
    Sad(B),
}

/// Token for listening for responses to requests
///
/// These tokens are always guaranteed, to have at most one equal counterpart, and they are
/// deliberately not [`Clone`]able or [`Copy`]able to help enforce that constraint.
///
/// Internally, the token contains a [`u64`] sourced from an incrementing atomic, no guarantees are
/// made about iteration order, and the tokens have no meaning across executions.
#[derive(PartialEq, Eq, Ord, PartialOrd, Debug, Hash)]
pub struct CompletionToken(Arc<u64>);

impl CompletionToken {
    /// Creates a pair of equal `CompletionTokens`
    ///
    /// It is guaranteed that no previous or future pair of `CompletionTokens`, at least within a
    /// single execution of the program, will be equal to these tokens
    pub fn new() -> (CompletionToken, CompletionToken) {
        /// Counter used for generating tokens
        static COUNTER: Lazy<AtomicU64> = Lazy::new(|| AtomicU64::new(0));
        let value = Arc::new(COUNTER.fetch_add(1, Ordering::SeqCst));
        (CompletionToken(value.clone()), CompletionToken(value))
    }

    /// Returns the number of copies of this [`CompletionToken`] currently alive
    ///
    /// This is used for doing "Garbage Collection" of token's whose partners have been dropped.
    pub fn count(&self) -> usize {
        let count = Arc::strong_count(&self.0);
        // We use a `debug_assert` here instead of an `assert` or returning an error for the
        // following reasons:
        //   - This will be used in places where there's no good place to put in error handling
        //   - There's no safe way for an application to recover from this that wouldn't be hugely
        //     onerous to implement
        //   - While this would be a bug, its consequences are likely limited to a memory leak, and
        //     since it should actually be quite hard to cause this such instances are expected to
        //     be rare, so it's not really worth it to outright crash a production application over
        //     this
        //   - It is still, however, worth being noisy to developers about, so that we can get bug
        //     reports filed and the bug fixed
        debug_assert!(
            count <= 2,
            "A CompletionToken has been illegally duplicated, copies: {} maximum: 2. \
             This is a bug, please file a bug report.",
            count
        );
        count
    }
}

/// Dynamic wrapper around an [`Error`] to make some type magic work
pub struct DynamicError(Box<dyn Error + Send + Sync>);

impl std::fmt::Display for DynamicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, f)
    }
}

impl std::fmt::Debug for DynamicError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[allow(deprecated)]
impl Error for DynamicError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        self.0.source()
    }

    fn description(&self) -> &str {
        self.0.description()
    }

    fn cause(&self) -> Option<&dyn Error> {
        self.0.cause()
    }
}

/// Wrapper type around an [`EventConsumer`] that boxes the resulting error
pub struct DynamicConsumer<E: Event, C: EventConsumer<E>> {
    /// Inner Value
    inner: C,
    /// Phantom for the event type, since we need it
    _event: PhantomData<E>,
}

impl<E: Event, C: EventConsumer<E>> From<C> for DynamicConsumer<E, C> {
    fn from(inner: C) -> Self {
        Self {
            inner,
            _event: PhantomData,
        }
    }
}

#[async_trait]
impl<E: Event, C: EventConsumer<E>> EventConsumer<E> for DynamicConsumer<E, C> {
    type Error = DynamicError;
    async fn accept(&self, event: E) -> Result<(), Self::Error> {
        match self.inner.accept(event).await {
            Ok(_) => Ok(()),
            Err(e) => Err(DynamicError(Box::new(e))),
        }
    }

    fn accept_sync(&self, event: E) -> Result<(), Self::Error> {
        match self.inner.accept_sync(event) {
            Ok(_) => Ok(()),
            Err(e) => Err(DynamicError(Box::new(e))),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    // Smoke test `CompletionToken`'s reference counting
    #[test]
    fn completion_token_ref_counting() {
        let (token_1, token_2) = CompletionToken::new();
        assert_eq!(token_1.count(), 2);
        assert_eq!(token_2.count(), 2);
        // Drop a token
        std::mem::drop(token_2);
        // Check the count
        assert_eq!(token_1.count(), 1);
    }
    // Make sure tokens are equal to themsleves
    #[test]
    fn token_self_equality() {
        let (token_a, token_b) = CompletionToken::new();
        assert_eq!(token_a, token_b);
    }
    // Make sure successive tokens are non-equal
    #[test]
    fn token_non_equal() {
        let (token_a, _) = CompletionToken::new();
        let (token_b, _) = CompletionToken::new();
        assert_ne!(token_a, token_b);
    }
}
