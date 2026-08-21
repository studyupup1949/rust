//! Wrapper type returned by AAML lookup methods.

use std::fmt::Display;
use std::ops::Deref;

/// The result of a successful key lookup in an [`AAML`](crate::aaml::AAML) map.
///
/// `FoundValue` wraps the string value associated with a key and provides
/// helper methods for common transformations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FoundValue {
    inner: String,
}

impl FoundValue {
    /// Creates a new `FoundValue` from a string slice.
    pub fn new(value: &str) -> FoundValue {
        FoundValue {
            inner: value.to_string(),
        }
    }

    /// Removes all occurrences of `target` from the inner string in-place.
    ///
    /// Returns `&mut Self` for chaining.
    pub fn remove(&mut self, target: &str) -> &mut Self {
        self.inner = self.inner.replace(target, "");
        self
    }

    /// Returns the inner value as a string slice.
    pub fn as_str(&self) -> &str {
        &self.inner
    }
}

impl From<String> for FoundValue {
    fn from(value: String) -> Self {
        FoundValue { inner: value }
    }
}

impl PartialEq<&str> for FoundValue {
    fn eq(&self, other: &&str) -> bool {
        self.inner == *other
    }
}

impl Display for FoundValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.inner.clone())
    }
}

impl Deref for FoundValue {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}