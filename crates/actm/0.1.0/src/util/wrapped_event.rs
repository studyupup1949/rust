//! Wrapper type to turn an arbitrary type into an [`Event`]
use std::{fmt::Debug, hash::Hash};

use crate::{traits::Event, types::CompletionToken};

/// Wrapper around compatible types that implements [`Event`] by keeping just enough manually
/// controlled state variables to implement the [`Event`] api, with no business logic aside from the
/// provided setters.
#[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Hash, Default)]
pub struct WrappedEvent<T>(T, Option<CompletionToken>);

impl<T> WrappedEvent<T> {
    /// Converts back to the contained type
    pub fn into_inner(self) -> T {
        self.0
    }
    /// Sets the [`CompletionToken`]
    pub fn set_completion_token(&mut self, token: CompletionToken) {
        self.1 = Some(token);
    }
}

impl<T> From<T> for WrappedEvent<T> {
    fn from(item: T) -> Self {
        Self(item, None)
    }
}

impl<T> Event for WrappedEvent<T>
where
    T: Clone + Eq + Debug + Ord + Hash + Send + Sync + 'static,
{
    type Flags = ();

    fn flags(&self) -> Self::Flags {}

    fn token(&mut self) -> Option<CompletionToken> {
        self.1.take()
    }

    fn tokenize(&mut self) -> Option<CompletionToken> {
        let (token_1, token_2) = CompletionToken::new();
        self.set_completion_token(token_1);
        Some(token_2)
    }

    fn stateless_clone(&self) -> Self {
        Self(self.0.clone(), None)
    }
}

/// Automatically generate an [`Event`] implementing wrapper for a type
///
/// This macro takes two arguments:
///   * The desired name of the wrapper type
///   * The name of the type to be wrapped
#[macro_export]
macro_rules! wrapped_event {
    ($wrapper_name: ident, $event_type: ident) => {
        #[derive(PartialEq, Eq, Debug, PartialOrd, Ord, Hash)]
        pub struct $wrapper_name(WrappedEvent<$event_type>);
        impl $wrapper_name {
            pub fn into_inner(self) -> $event_type {
                self.0.into_inner()
            }
            pub fn set_completion_token(&mut self, token: CompletionToken) {
                self.0.set_completion_token(token);
            }
        }
        impl From<$event_type> for $wrapper_name {
            fn from(item: $event_type) -> Self {
                Self(WrappedEvent::from(item))
            }
        }
        impl Event for $wrapper_name {
            type Flags = ();
            fn flags(&self) -> Self::Flags {}
            fn token(&mut self) -> Option<CompletionToken> {
                self.0.token()
            }
            fn tokenize(&mut self) -> Option<CompletionToken> {
                self.0.tokenize()
            }
            fn stateless_clone(&self) -> Self {
                Self(self.0.stateless_clone())
            }
        }
    };
}
