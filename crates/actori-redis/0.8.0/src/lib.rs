//! Redis integration for Actori framework.
//!
//! ## Documentation
//! * [API Documentation (Development)](http://actori.github.io/actori-redis/actori_redis/)
//! * [API Documentation (Releases)](https://docs.rs/actori-redis/)
//! * [Chat on gitter](https://gitter.im/actori/actori)
//! * Cargo package: [actori-redis](https://crates.io/crates/actori-redis)
//! * Minimum supported Rust version: 1.26 or later
//!
#[macro_use]
extern crate log;
#[macro_use]
extern crate redis_async;
#[macro_use]
extern crate derive_more;

mod redis;
pub use redis::{Command, RedisActor};

#[cfg(feature = "web")]
mod session;
#[cfg(feature = "web")]
pub use actori_web::cookie::SameSite;
#[cfg(feature = "web")]
pub use session::RedisSession;

/// General purpose actori redis error
#[derive(Debug, Display, From)]
pub enum Error {
    #[display(fmt = "Redis error {}", _0)]
    Redis(redis_async::error::Error),
    /// Receiving message during reconnecting
    #[display(fmt = "Redis: Not connected")]
    NotConnected,
    /// Cancel all waters when connection get dropped
    #[display(fmt = "Redis: Disconnected")]
    Disconnected,
}

#[cfg(feature = "web")]
impl actori_web::ResponseError for Error {}

// re-export
pub use redis_async::error::Error as RespError;
pub use redis_async::resp::RespValue;
