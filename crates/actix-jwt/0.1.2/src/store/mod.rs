//! Refresh-token storage backends and factory utilities.
//!
//! Mirrors the
//! [`store/`](https://github.com/LdDl/echo-jwt/tree/master/store) package
//! from the Go implementation.
//!
//! # Available backends
//!
//! | Backend | Feature flag | Type |
//! |---------|-------------|------|
//! | In-memory (`HashMap` + `RwLock`) | *(always)* | [`InMemoryRefreshTokenStore`] |
//! | Redis (`redis` crate) | `redis-store` | `RedisRefreshTokenStore` |
//!
//! # Factory
//!
//! Use [`Factory`], [`new_store`], [`new_memory_store`] or [`default_store`]
//! to create stores without importing concrete types.
//!
//! ```
//! use actix_jwt::store::{default_store, new_memory_store};
//!
//! let store = default_store();
//! let another = new_memory_store();
//! ```

pub mod factory;
pub mod memory;
#[cfg(feature = "redis-store")]
pub mod redis;

pub use factory::*;
pub use memory::*;
#[cfg(feature = "redis-store")]
pub use redis::*;

use crate::core::TokenStore;

/// Creates a default in-memory token store.
///
/// This is a shorthand for `Box::new(InMemoryRefreshTokenStore::new())`.
///
/// # Examples
///
/// ```
/// # #[tokio::main]
/// # async fn main() {
/// let store = actix_jwt::store::default_store();
/// assert_eq!(store.count().await.unwrap(), 0);
/// # }
/// ```
pub fn default_store() -> Box<dyn TokenStore> {
    Box::new(InMemoryRefreshTokenStore::new())
}
