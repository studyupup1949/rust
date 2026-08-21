//! Representation and implementation of a rate-limiter core logic.

use crate::{limit::Limit, route::Route};

/// Represents a mapping between the path and the maximum number of requests
/// per second.
#[derive(Clone, Debug)]
pub struct RateLimiter {
    default: Limit,
    mapping: Vec<(Route, Limit)>,
}

impl RateLimiter {
    pub(crate) fn get_limit(&self, route: &str, method: &str) -> Limit {
        // TODO: rewrite this, looks really messy
        for pair in &self.mapping {
            if pair.0.is_match(route, method) {
                return pair.1;
            }
        }

        self.default
    }
}

/// Builder for `RateLimiter` struct.
#[derive(Clone, Debug)]
pub struct RateLimiterBuilder {
    default: Limit,
    mapping: Vec<(Route, Limit)>,
}

impl Default for RateLimiterBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RateLimiterBuilder {
    /// Creates a new instance of `RateLimiterBuilder` with some pre-defined values.
    pub fn new() -> Self {
        Self {
            default: Limit::default(),
            mapping: Vec::new(),
        }
    }

    /// Sets a new value for the default amount of allowed requests per second.
    /// This value is used when the path of the incoming request is not mapped.
    /// The default value for TTL is 5, and for amount, it is 1.
    pub fn set_default(&mut self, value: Limit) -> &mut Self {
        self.default = value;
        self
    }

    /// Adds a new route to the mapping. See documentation for `Route` for more
    /// information about `route` paraeter. A Limit is set for requests per
    /// second (RPC).
    ///
    /// See [Regex](https://docs.rs/regex/latest/regex/) documentation for more.
    pub fn add_route(&mut self, route: Route, limit: Limit) -> &mut Self {
        self.mapping.push((route, limit));
        self
    }

    /// Builds the instance of the builder and converts it into a `RateLimiter`
    /// instance with the set values.
    pub fn build(&self) -> RateLimiter {
        RateLimiter {
            default: self.default,
            mapping: self.mapping.clone(),
        }
    }
}
