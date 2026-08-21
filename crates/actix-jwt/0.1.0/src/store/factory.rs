//! Store factory for creating token stores from configuration.
//!
//! Mirrors
//! [`store/factory.go`](https://github.com/LdDl/echo-jwt/blob/master/store/factory.go)
//! from the Go implementation.

use crate::core::TokenStore;
use crate::errors::JwtError;

use super::memory::InMemoryRefreshTokenStore;

/// Selects which storage backend to create.
///
/// # Examples
///
/// ```
/// use actix_jwt::store::factory::StoreType;
///
/// let st = StoreType::Memory;
/// assert_eq!(st, StoreType::Memory);
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum StoreType {
    /// In-memory store (no external dependencies).
    Memory,
    /// Redis-backed store (requires the `redis-store` feature).
    Redis,
}

/// Configuration passed to [`Factory::create_store`].
///
/// # Examples
///
/// ```
/// use actix_jwt::store::factory::{StoreConfig, StoreType};
///
/// let config = StoreConfig::default();
/// assert_eq!(config.store_type, StoreType::Memory);
/// ```
pub struct StoreConfig {
    /// Which backend to instantiate.
    pub store_type: StoreType,
    /// Redis-specific configuration (only used when `store_type` is
    /// [`StoreType::Redis`] **and** the `redis-store` feature is enabled).
    #[cfg(feature = "redis-store")]
    pub redis: Option<super::redis::RedisConfig>,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            store_type: StoreType::Memory,
            #[cfg(feature = "redis-store")]
            redis: None,
        }
    }
}

/// Factory for creating [`TokenStore`] instances from a [`StoreConfig`].
///
/// # Examples
///
/// ```
/// use actix_jwt::store::factory::{Factory, StoreConfig};
///
/// # #[tokio::main]
/// # async fn main() {
/// let factory = Factory::new();
/// let store = factory.create_store(&StoreConfig::default()).await.unwrap();
/// assert_eq!(store.count().await.unwrap(), 0);
/// # }
/// ```
pub struct Factory;

impl Factory {
    /// Creates a new factory instance.
    pub fn new() -> Self {
        Factory
    }

    /// Creates a [`TokenStore`] based on the provided configuration.
    ///
    /// # Errors
    ///
    /// * [`JwtError::Internal`] - when `StoreType::Redis` is requested but
    ///   the `redis-store` feature is not enabled, or when the Redis
    ///   connection fails.
    pub async fn create_store(
        &self,
        config: &StoreConfig,
    ) -> Result<Box<dyn TokenStore>, JwtError> {
        match config.store_type {
            StoreType::Memory => Ok(Box::new(InMemoryRefreshTokenStore::new())),
            StoreType::Redis => {
                #[cfg(feature = "redis-store")]
                {
                    let redis_config = config.redis.clone().unwrap_or_default();
                    let store = super::redis::RedisRefreshTokenStore::new(&redis_config).await?;
                    Ok(Box::new(store))
                }
                #[cfg(not(feature = "redis-store"))]
                {
                    Err(JwtError::Internal(
                        "Redis store feature not enabled. Enable the 'redis-store' feature in Cargo.toml".into(),
                    ))
                }
            }
        }
    }
}

impl Default for Factory {
    fn default() -> Self {
        Self::new()
    }
}

/// Creates an in-memory token store.
///
/// Shorthand for `Box::new(InMemoryRefreshTokenStore::new())`.
///
/// # Examples
///
/// ```
/// # #[tokio::main]
/// # async fn main() {
/// let store = actix_jwt::store::new_memory_store();
/// assert_eq!(store.count().await.unwrap(), 0);
/// # }
/// ```
pub fn new_memory_store() -> Box<dyn TokenStore> {
    Box::new(InMemoryRefreshTokenStore::new())
}

/// Same as [`new_memory_store`] but mirrors
/// [`MustNewMemoryStore`](https://github.com/LdDl/echo-jwt/blob/master/store/factory.go)
/// from the Go implementation, which panics on error.  Since the in-memory store is infallible this
/// function never panics, but the name is kept for API parity.
///
/// # Examples
///
/// ```
/// # #[tokio::main]
/// # async fn main() {
/// let store = actix_jwt::store::factory::must_new_memory_store();
/// assert_eq!(store.count().await.unwrap(), 0);
/// # }
/// ```
pub fn must_new_memory_store() -> Box<dyn TokenStore> {
    new_memory_store()
}

/// Creates a token store from the given configuration using the default
/// [`Factory`].
///
/// # Examples
///
/// ```
/// use actix_jwt::store::factory::{new_store, StoreConfig};
///
/// # #[tokio::main]
/// # async fn main() {
/// let store = new_store(&StoreConfig::default()).await.unwrap();
/// assert_eq!(store.count().await.unwrap(), 0);
/// # }
/// ```
pub async fn new_store(config: &StoreConfig) -> Result<Box<dyn TokenStore>, JwtError> {
    Factory::new().create_store(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_factory_create_store_memory() {
        let factory = Factory::new();
        let config = StoreConfig {
            store_type: StoreType::Memory,
            #[cfg(feature = "redis-store")]
            redis: None,
        };

        let store = factory.create_store(&config).await;
        assert!(
            store.is_ok(),
            "Factory should create memory store successfully"
        );

        // Verify the store works
        let store = store.unwrap();
        let count = store.count().await.unwrap();
        assert_eq!(count, 0, "New memory store should be empty");
    }

    #[tokio::test]
    async fn test_factory_default_config() {
        let factory = Factory::new();
        let config = StoreConfig::default();

        assert_eq!(
            config.store_type,
            StoreType::Memory,
            "Default config should use Memory type"
        );

        let store = factory.create_store(&config).await;
        assert!(
            store.is_ok(),
            "Factory should create store from default config"
        );
    }

    #[tokio::test]
    async fn test_new_store_memory() {
        let config = StoreConfig {
            store_type: StoreType::Memory,
            #[cfg(feature = "redis-store")]
            redis: None,
        };

        let store = new_store(&config).await;
        assert!(
            store.is_ok(),
            "new_store() should create memory store successfully"
        );

        let store = store.unwrap();
        let count = store.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_new_memory_store() {
        let store = new_memory_store();
        let count = store.count().await.unwrap();
        assert_eq!(count, 0, "new_memory_store() should return an empty store");
    }

    #[tokio::test]
    async fn test_default_store() {
        let store = crate::store::default_store();
        let count = store.count().await.unwrap();
        assert_eq!(count, 0, "default_store() should return an empty store");
    }

    #[tokio::test]
    async fn test_must_new_memory_store() {
        let store = must_new_memory_store();
        let count = store.count().await.unwrap();
        assert_eq!(
            count, 0,
            "must_new_memory_store() should return an empty store"
        );
    }

    #[cfg(not(feature = "redis-store"))]
    #[tokio::test]
    async fn test_factory_create_store_redis_without_feature() {
        let factory = Factory::new();
        let config = StoreConfig {
            store_type: StoreType::Redis,
        };

        let result = factory.create_store(&config).await;
        assert!(
            result.is_err(),
            "Creating Redis store without feature should fail"
        );

        let err_msg = match result {
            Err(e) => e.to_string(),
            Ok(_) => panic!("Expected error but got Ok"),
        };
        assert!(
            err_msg.contains("Redis store feature not enabled"),
            "Error should mention feature not enabled, got: {}",
            err_msg
        );
    }
}
