//! Redis-backed refresh-token store.
//!
//! Mirrors
//! [`store/redis.go`](https://github.com/LdDl/echo-jwt/blob/master/store/redis.go)
//! from the Go implementation.
//!
//! Requires the **`redis-store`** Cargo feature.

use async_trait::async_trait;
use chrono::Utc;
use redis::AsyncCommands;

use crate::core::{RefreshTokenData, TokenStore};
use crate::errors::JwtError;

/// Configuration for connecting to a Redis instance.
///
/// # Examples
///
/// ```no_run
/// use actix_jwt::store::redis::RedisConfig;
///
/// let config = RedisConfig {
///     addr: "redis://10.0.0.1:6379/".to_string(),
///     password: Some("secret".to_string()),
///     db: 2,
///     key_prefix: "myapp:jwt:".to_string(),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis connection URL (e.g. `"redis://127.0.0.1:6379/"`).
    pub addr: String,
    /// Optional password for Redis authentication.
    pub password: Option<String>,
    /// Redis database number.
    pub db: i32,
    /// Logical pool size (informational; `ConnectionManager` is
    /// single-multiplexed).
    pub pool_size: u32,
    /// Key prefix prepended to every token key stored in Redis.
    pub key_prefix: String,
    /// Whether to use TLS (`rediss://` scheme).
    pub tls: bool,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            addr: "redis://127.0.0.1:6379/".to_string(),
            password: None,
            db: 0,
            pool_size: 10,
            key_prefix: "actix-jwt:".to_string(),
            tls: false,
        }
    }
}

/// Redis-backed implementation of [`TokenStore`].
///
/// Uses [`redis::aio::ConnectionManager`] for automatic reconnection and
/// multiplexed access.  Tokens are stored as JSON-serialized
/// [`RefreshTokenData`] with a Redis TTL matching the token expiry.
///
/// # Examples
///
/// ```no_run
/// use actix_jwt::store::redis::{RedisConfig, RedisRefreshTokenStore};
/// use actix_jwt::core::TokenStore;
/// use chrono::{Duration, Utc};
///
/// # #[tokio::main]
/// # async fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let config = RedisConfig::default();
/// let store = RedisRefreshTokenStore::new(&config).await?;
///
/// let expiry = Utc::now() + Duration::hours(1);
/// store.set("tok-1", serde_json::json!({"uid": 1}), expiry).await?;
///
/// let data = store.get("tok-1").await?;
/// assert_eq!(data["uid"], 1);
/// # Ok(())
/// # }
/// ```
pub struct RedisRefreshTokenStore {
    conn: redis::aio::ConnectionManager,
    prefix: String,
}

impl std::fmt::Debug for RedisRefreshTokenStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisRefreshTokenStore")
            .field("prefix", &self.prefix)
            .finish()
    }
}

impl RedisRefreshTokenStore {
    /// Creates a new store and verifies the connection with `PING`.
    ///
    /// # Errors
    ///
    /// Returns [`JwtError::Internal`] if the Redis client cannot be created
    /// or the initial `PING` fails.
    pub async fn new(config: &RedisConfig) -> Result<Self, JwtError> {
        let mut url = config.addr.clone();

        if config.db != 0 && !url.contains('/') {
            url = format!("{}/{}", url.trim_end_matches('/'), config.db);
        }

        if config.tls && url.starts_with("redis://") {
            url = url.replacen("redis://", "rediss://", 1);
        }

        let client = redis::Client::open(url.as_str())
            .map_err(|e| JwtError::Internal(format!("Failed to create Redis client: {}", e)))?;

        let conn = redis::aio::ConnectionManager::new(client)
            .await
            .map_err(|e| JwtError::Internal(format!("Failed to connect to Redis: {}", e)))?;

        let store = Self {
            conn,
            prefix: config.key_prefix.clone(),
        };

        store.ping().await?;

        Ok(store)
    }

    /// Returns the prefixed key for a given token.
    fn key(&self, token: &str) -> String {
        format!("{}{}", self.prefix, token)
    }

    /// Sends a `PING` command to verify the Redis connection is alive.
    pub async fn ping(&self) -> Result<(), JwtError> {
        let mut conn = self.conn.clone();
        let pong: String = redis::cmd("PING")
            .query_async(&mut conn)
            .await
            .map_err(|e| JwtError::Internal(format!("Redis PING failed: {}", e)))?;

        if pong != "PONG" {
            return Err(JwtError::Internal(format!(
                "Unexpected PING response: {}",
                pong
            )));
        }
        Ok(())
    }

    /// Drops the underlying connection manager.
    ///
    /// The store must not be used after calling this method.
    pub async fn close(self) {
        drop(self.conn);
    }

    /// Flushes the current Redis database.  **Intended for testing only.**
    pub async fn flush_db(&self) -> Result<(), JwtError> {
        let mut conn = self.conn.clone();
        redis::cmd("FLUSHDB")
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| JwtError::Internal(format!("Redis FLUSHDB failed: {}", e)))?;
        Ok(())
    }
}

#[async_trait]
impl TokenStore for RedisRefreshTokenStore {
    /// Stores a refresh token in Redis using `SETEX`.
    ///
    /// The token data is JSON-serialized and stored with a TTL derived from
    /// the expiry time.
    ///
    /// # Errors
    ///
    /// * [`JwtError::TokenEmpty`] - empty token string.
    /// * [`JwtError::ExpiryInPast`] - computed TTL < 1 second.
    async fn set(
        &self,
        token: &str,
        user_data: serde_json::Value,
        expiry: chrono::DateTime<Utc>,
    ) -> Result<(), JwtError> {
        if token.is_empty() {
            return Err(JwtError::TokenEmpty);
        }

        let ttl_secs = (expiry - Utc::now()).num_seconds();
        if ttl_secs < 1 {
            return Err(JwtError::ExpiryInPast);
        }

        let data = RefreshTokenData {
            user_data,
            expiry,
            created: Utc::now(),
        };

        let serialized = serde_json::to_string(&data)
            .map_err(|e| JwtError::Internal(format!("Failed to serialize token data: {}", e)))?;

        let mut conn = self.conn.clone();
        conn.set_ex::<_, _, ()>(self.key(token), serialized, ttl_secs as u64)
            .await
            .map_err(|e| JwtError::Internal(format!("Redis SETEX failed: {}", e)))?;

        Ok(())
    }

    /// Retrieves user data for a refresh token from Redis.
    ///
    /// If the token data is found but expired (edge case due to clock skew),
    /// the key is deleted and [`JwtError::RefreshTokenNotFound`] is returned.
    ///
    /// # Errors
    ///
    /// * [`JwtError::TokenEmpty`] - empty token string.
    /// * [`JwtError::RefreshTokenNotFound`] - key does not exist or is
    ///   expired.
    async fn get(&self, token: &str) -> Result<serde_json::Value, JwtError> {
        if token.is_empty() {
            return Err(JwtError::TokenEmpty);
        }

        let mut conn = self.conn.clone();
        let result: Option<String> = conn
            .get(self.key(token))
            .await
            .map_err(|e| JwtError::Internal(format!("Redis GET failed: {}", e)))?;

        match result {
            Some(serialized) => {
                let data: RefreshTokenData = serde_json::from_str(&serialized).map_err(|e| {
                    JwtError::Internal(format!("Failed to deserialize token data: {}", e))
                })?;

                if data.is_expired() {
                    let mut del_conn = self.conn.clone();
                    let _ = del_conn.del::<_, ()>(self.key(token)).await;
                    return Err(JwtError::RefreshTokenNotFound);
                }

                Ok(data.user_data)
            }
            None => Err(JwtError::RefreshTokenNotFound),
        }
    }

    /// Removes a refresh token from Redis.
    ///
    /// Silently succeeds if the token is empty or the key does not exist.
    async fn delete(&self, token: &str) -> Result<(), JwtError> {
        if token.is_empty() {
            return Ok(());
        }

        let mut conn = self.conn.clone();
        conn.del::<_, ()>(self.key(token))
            .await
            .map_err(|e| JwtError::Internal(format!("Redis DEL failed: {}", e)))?;

        Ok(())
    }

    /// Scans all keys with the configured prefix and removes expired tokens.
    ///
    /// Uses `SCAN` to iterate without blocking Redis.  Returns the number of
    /// removed entries.
    async fn cleanup(&self) -> Result<usize, JwtError> {
        let pattern = format!("{}*", self.prefix);
        let mut conn = self.conn.clone();
        let mut removed = 0usize;
        let mut cursor: u64 = 0;

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .map_err(|e| JwtError::Internal(format!("Redis SCAN failed: {}", e)))?;

            for key in &keys {
                let value: Option<String> = conn
                    .get(key)
                    .await
                    .map_err(|e| JwtError::Internal(format!("Redis GET failed: {}", e)))?;

                if let Some(serialized) = value {
                    if let Ok(data) = serde_json::from_str::<RefreshTokenData>(&serialized) {
                        if data.is_expired() {
                            conn.del::<_, ()>(key).await.map_err(|e| {
                                JwtError::Internal(format!("Redis DEL failed: {}", e))
                            })?;
                            removed += 1;
                        }
                    }
                }
            }

            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(removed)
    }

    /// Counts all keys matching the configured prefix using `SCAN`.
    async fn count(&self) -> Result<usize, JwtError> {
        let pattern = format!("{}*", self.prefix);
        let mut conn = self.conn.clone();
        let mut total = 0usize;
        let mut cursor: u64 = 0;

        loop {
            let (next_cursor, keys): (u64, Vec<String>) = redis::cmd("SCAN")
                .arg(cursor)
                .arg("MATCH")
                .arg(&pattern)
                .arg("COUNT")
                .arg(100)
                .query_async(&mut conn)
                .await
                .map_err(|e| JwtError::Internal(format!("Redis SCAN failed: {}", e)))?;

            total += keys.len();

            cursor = next_cursor;
            if cursor == 0 {
                break;
            }
        }

        Ok(total)
    }
}
