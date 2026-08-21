//! Handle rate-limit
//!
//! Response headers:
//! - X-RateLimit-Limit: 60
//! - X-RateLimit-Remaining: 59
//! - X-RateLimit-Reset: 1350085394

mod limiter;
mod redis_backend;
mod types;
mod util;

#[macro_use]
extern crate log;

pub use limiter::RateLimit;
