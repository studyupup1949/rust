//! Wrapper type returned by AAML lookup methods.

use std::fmt::Display;
use std::ops::Deref;
use std::collections::HashMap;
use crate::aaml::parsing;
use crate::types::list::ListType;

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

    /// Parses the value as a list literal `[item, item, ...]` and returns
    /// the items as a `Vec<String>`.
    ///
    /// Returns `None` if the value is not in `[...]` form.
    ///
    /// # Example
    /// ```
    /// use aam_rs::found_value::FoundValue;
    /// let v = FoundValue::new("[rust, aam, config]");
    /// assert_eq!(v.as_list().unwrap(), vec!["rust", "aam", "config"]);
    /// ```
    pub fn as_list(&self) -> Option<Vec<String>> {
        ListType::parse_items(&self.inner)
            .map(|items| items.iter().map(|s| s.to_string()).collect())
    }

    /// Parses the value as an inline object `{ k = v, ... }` and returns a
    /// `HashMap<String, String>` of its fields.
    ///
    /// Returns `None` if the value is not in `{...}` form or cannot be parsed.
    ///
    /// # Example
    /// ```
    /// use aam_rs::found_value::FoundValue;
    /// let v = FoundValue::new("{ x = 1.0, y = 2.0 }");
    /// let map = v.as_object().unwrap();
    /// assert_eq!(map["x"], "1.0");
    /// ```
    pub fn as_object(&self) -> Option<HashMap<String, String>> {
        if !parsing::is_inline_object(&self.inner) {
            return None;
        }
        parsing::parse_inline_object(&self.inner)
            .ok()
            .map(|pairs| pairs.into_iter().collect())
    }

    /// Returns `true` when this value is a list literal `[...]`.
    pub fn is_list(&self) -> bool {
        let s = self.inner.trim();
        s.starts_with('[') && s.ends_with(']')
    }

    /// Returns `true` when this value is an inline object literal `{...}`.
    pub fn is_object(&self) -> bool {
        parsing::is_inline_object(&self.inner)
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