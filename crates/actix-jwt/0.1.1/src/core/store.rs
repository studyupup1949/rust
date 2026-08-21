//! Refresh-token storage trait.
//!
//! Mirrors the
//! [`core.TokenStore`](https://github.com/LdDl/echo-jwt/blob/master/core/store.go)
//! interface from the Go implementation. Implementations live in the
//! [`crate::store`] module.

use async_trait::async_trait;

use crate::errors::JwtError;

/// Async trait defining the contract for refresh-token storage backends.
///
/// All methods are object-safe so that `Arc<dyn TokenStore>` can be used at
/// runtime to swap implementations (in-memory, Redis, etc.).
///
/// # Provided implementations
///
/// * [`crate::store::InMemoryRefreshTokenStore`] - thread-safe `HashMap`
///   behind a [`tokio::sync::RwLock`].
/// * `RedisRefreshTokenStore` - Redis-backed store
///   (requires the `redis-store` feature).
///
/// # Examples
///
/// ```
/// use actix_jwt::core::TokenStore;
/// use actix_jwt::store::InMemoryRefreshTokenStore;
///
/// # #[tokio::main]
/// # async fn main() {
/// let store: Box<dyn TokenStore> = Box::new(InMemoryRefreshTokenStore::new());
/// let count = store.count().await.unwrap();
/// assert_eq!(count, 0);
/// # }
/// ```
#[async_trait]
pub trait TokenStore: Send + Sync {
    /// Stores a refresh token with associated user data and expiration.
    async fn set(
        &self,
        token: &str,
        user_data: serde_json::Value,
        expiry: chrono::DateTime<chrono::Utc>,
    ) -> Result<(), JwtError>;

    /// Retrieves user data associated with a refresh token.
    async fn get(&self, token: &str) -> Result<serde_json::Value, JwtError>;

    /// Removes a refresh token from storage.
    async fn delete(&self, token: &str) -> Result<(), JwtError>;

    /// Removes expired tokens. Returns the count of cleaned entries.
    async fn cleanup(&self) -> Result<usize, JwtError>;

    /// Returns the total number of active refresh tokens.
    async fn count(&self) -> Result<usize, JwtError>;
}
