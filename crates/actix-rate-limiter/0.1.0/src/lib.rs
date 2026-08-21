//! `actix-rate-limiter` is a simple yet powerful per-route rate limiter for
//! [Actix](https://docs.rs/actix-web/latest/actix_web/) with support for regex.
//!
//! ### Available backends
//!
//! Right now, only in-memory storage is supported officially. But you can
//! create your own backend using the `BackendProvider` trait. You can use
//! `MemoryBackendProvider` as an example implementation.
//!
//! We plan to add support for some other backends in the future, such as Redis.
//! If you want to help with their development, please checkout our
//! [GitHub](https://github.com/Pelfox/actix-rate-limiter).
//!
//! ### Examples
//!
//! Check the examples folder of our repository to see the available code samples.
//!

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unsafe_code
)]

use regex::Regex;

pub mod backend;
pub mod middleware;

/// General type for limits inside the rate limiter.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct Limit {
    /// Lifetime of a bucket in seconds.
    pub ttl: i64,
    /// The amount of requests that one specific bucket can accept in the TTL.
    pub amount: i64,
}

/// General type for tne ID of the request. It consists of the requester's
/// identifier and the request's path. Guaranteed format: `{id}:{path}`.
pub type RequestId = String;

/// Represents a mapping between the path and the maximum number of requests
/// per second.
#[derive(Clone, Debug)]
pub struct RateLimiter {
    default: Limit,
    mapping: Vec<(Regex, Limit)>,
}

impl RateLimiter {
    /// Returns the limit for the specified route. Fallbacks to `self.default`
    /// value if the limit aren't specified.
    pub(crate) fn get_limit(&self, route: &str) -> Limit {
        // TODO: rewrite this, looks really messy
        for pair in &self.mapping {
            if pair.0.is_match(route) {
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
    mapping: Vec<(Regex, Limit)>,
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
            default: Limit { ttl: 1, amount: 5 },
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

    /// Adds a new route to the mapping. If needed, you can specify the `route`
    /// parameter as a regex, which will be later validated and checked. A Limit
    /// is set for requests per second (RPC).
    ///
    /// See [Regex](https://docs.rs/regex/latest/regex/) documentation for more.
    pub fn add_route(&mut self, route: &str, limit: Limit) -> &mut Self {
        self.mapping.push((
            // fix for per-line route regex
            Regex::new(format!("^({})$", route).as_str()).unwrap(), // FIXME: this will panic on error
            limit,
        ));
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
