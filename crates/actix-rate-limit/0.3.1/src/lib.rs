//! Rate-Limit middleware for `actix-web`
//!
//! Response headers:
//! - X-RateLimit-Limit: 60
//! - X-RateLimit-Remaining: 59
//! - X-RateLimit-Reset: 1350085394

mod limiter;
mod types;
mod util;

#[cfg(feature = "redis")]
mod redis_backend;

#[cfg(feature = "redis")]
mod redis_limiter;

#[macro_use]
extern crate log;

pub use limiter::RateLimit;
