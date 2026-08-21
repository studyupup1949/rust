//! Thread-safe, in-memory refresh-token store.
//!
//! Mirrors
//! [`store/memory.go`](https://github.com/LdDl/echo-jwt/blob/master/store/memory.go)
//! from the Go implementation.

use std::collections::HashMap;

use async_trait::async_trait;
use chrono::Utc;
use tokio::sync::RwLock;

use crate::core::{RefreshTokenData, TokenStore};
use crate::errors::JwtError;

/// Thread-safe, in-memory implementation of [`TokenStore`].
///
/// Internally keeps a `HashMap<String, RefreshTokenData>` behind a
/// [`tokio::sync::RwLock`], allowing concurrent reads while serializing
/// writes.  Suitable for development, testing and single-instance
/// deployments.
///
/// # Examples
///
/// ```
/// use chrono::{Duration, Utc};
/// use actix_jwt::core::TokenStore;
/// use actix_jwt::store::InMemoryRefreshTokenStore;
///
/// # #[tokio::main]
/// # async fn main() {
/// let store = InMemoryRefreshTokenStore::new();
///
/// let expiry = Utc::now() + Duration::hours(1);
/// store.set("tok-1", serde_json::json!({"uid": 1}), expiry).await.unwrap();
///
/// let data = store.get("tok-1").await.unwrap();
/// assert_eq!(data["uid"], 1);
///
/// store.delete("tok-1").await.unwrap();
/// assert!(store.get("tok-1").await.is_err());
/// # }
/// ```
pub struct InMemoryRefreshTokenStore {
    tokens: RwLock<HashMap<String, RefreshTokenData>>,
}

impl InMemoryRefreshTokenStore {
    /// Creates a new empty store.
    pub fn new() -> Self {
        Self {
            tokens: RwLock::new(HashMap::new()),
        }
    }

    /// Returns a clone of all **non-expired** tokens in the store.
    ///
    /// Expired tokens are filtered out but **not** removed from the
    /// underlying map.  Call [`TokenStore::cleanup`] to actually evict them.
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{Duration, Utc};
    /// use actix_jwt::store::InMemoryRefreshTokenStore;
    /// use actix_jwt::core::TokenStore;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let store = InMemoryRefreshTokenStore::new();
    /// let expiry = Utc::now() + Duration::hours(1);
    /// store.set("a", serde_json::json!(1), expiry).await.unwrap();
    ///
    /// let all = store.get_all().await;
    /// assert_eq!(all.len(), 1);
    /// # }
    /// ```
    pub async fn get_all(&self) -> HashMap<String, RefreshTokenData> {
        let tokens = self.tokens.read().await;
        let now = Utc::now();
        tokens
            .iter()
            .filter(|(_, data)| data.expiry > now)
            .map(|(k, v)| (k.clone(), v.clone()))
            .collect()
    }

    /// Removes **all** tokens from the store (including non-expired ones).
    ///
    /// # Examples
    ///
    /// ```
    /// use chrono::{Duration, Utc};
    /// use actix_jwt::store::InMemoryRefreshTokenStore;
    /// use actix_jwt::core::TokenStore;
    ///
    /// # #[tokio::main]
    /// # async fn main() {
    /// let store = InMemoryRefreshTokenStore::new();
    /// store.set("x", serde_json::json!(1), Utc::now() + Duration::hours(1)).await.unwrap();
    /// store.clear().await;
    /// assert_eq!(store.count().await.unwrap(), 0);
    /// # }
    /// ```
    pub async fn clear(&self) {
        let mut tokens = self.tokens.write().await;
        tokens.clear();
    }
}

impl Default for InMemoryRefreshTokenStore {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TokenStore for InMemoryRefreshTokenStore {
    /// Stores a refresh token with associated user data and expiration.
    ///
    /// # Errors
    ///
    /// Returns [`JwtError::TokenEmpty`] if `token` is an empty string.
    async fn set(
        &self,
        token: &str,
        user_data: serde_json::Value,
        expiry: chrono::DateTime<Utc>,
    ) -> Result<(), JwtError> {
        if token.is_empty() {
            return Err(JwtError::TokenEmpty);
        }

        let data = RefreshTokenData {
            user_data,
            expiry,
            created: Utc::now(),
        };

        let mut tokens = self.tokens.write().await;
        tokens.insert(token.to_string(), data);
        Ok(())
    }

    /// Retrieves user data for the given refresh token.
    ///
    /// Performs lazy cleanup: if the token exists but is expired it is removed
    /// from the store and [`JwtError::RefreshTokenNotFound`] is returned.
    ///
    /// # Errors
    ///
    /// * [`JwtError::TokenEmpty`] - empty token string.
    /// * [`JwtError::RefreshTokenNotFound`] - token absent or expired.
    async fn get(&self, token: &str) -> Result<serde_json::Value, JwtError> {
        if token.is_empty() {
            return Err(JwtError::TokenEmpty);
        }

        let mut tokens = self.tokens.write().await;
        match tokens.get(token) {
            Some(data) => {
                if data.is_expired() {
                    tokens.remove(token);
                    Err(JwtError::RefreshTokenNotFound)
                } else {
                    Ok(data.user_data.clone())
                }
            }
            None => Err(JwtError::RefreshTokenNotFound),
        }
    }

    /// Removes a refresh token from storage.
    ///
    /// Silently succeeds if the token is empty or does not exist.
    async fn delete(&self, token: &str) -> Result<(), JwtError> {
        if token.is_empty() {
            return Ok(());
        }

        let mut tokens = self.tokens.write().await;
        tokens.remove(token);
        Ok(())
    }

    /// Removes all expired tokens from the store.
    ///
    /// Returns the number of entries that were evicted.
    async fn cleanup(&self) -> Result<usize, JwtError> {
        let mut tokens = self.tokens.write().await;
        let now = Utc::now();
        let before = tokens.len();
        tokens.retain(|_, data| data.expiry > now);
        let after = tokens.len();
        Ok(before - after)
    }

    /// Returns the total number of tokens in the store **including expired
    /// ones**.
    ///
    /// To get only valid tokens use [`get_all`](Self::get_all).
    async fn count(&self) -> Result<usize, JwtError> {
        let tokens = self.tokens.read().await;
        Ok(tokens.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[tokio::test]
    async fn test_set() {
        let store = InMemoryRefreshTokenStore::new();
        let user_data =
            serde_json::json!({"id": "123", "username": "testuser", "email": "test@example.com"});
        let expiry = Utc::now() + Duration::hours(1);

        store.set("token123", user_data, expiry).await.unwrap();

        let count = store.count().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_get() {
        let store = InMemoryRefreshTokenStore::new();
        let user_data =
            serde_json::json!({"id": "123", "username": "testuser", "email": "test@example.com"});
        let expiry = Utc::now() + Duration::hours(1);

        store
            .set("token123", user_data.clone(), expiry)
            .await
            .unwrap();

        let result = store.get("token123").await.unwrap();
        assert_eq!(result["id"], "123");
        assert_eq!(result["username"], "testuser");
        assert_eq!(result["email"], "test@example.com");
    }

    #[tokio::test]
    async fn test_set_empty_token() {
        let store = InMemoryRefreshTokenStore::new();
        let expiry = Utc::now() + Duration::hours(1);

        let result = store.set("", serde_json::json!({}), expiry).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_empty_token() {
        let store = InMemoryRefreshTokenStore::new();

        let result = store.get("").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_nonexistent() {
        let store = InMemoryRefreshTokenStore::new();

        let result = store.get("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_get_expired_auto_cleanup() {
        let store = InMemoryRefreshTokenStore::new();
        let expiry = Utc::now() - Duration::seconds(1);

        // Insert an already-expired token
        {
            let mut tokens = store.tokens.write().await;
            tokens.insert(
                "expired".to_string(),
                RefreshTokenData {
                    user_data: serde_json::json!({"user_id": "123"}),
                    expiry,
                    created: Utc::now() - Duration::hours(1),
                },
            );
        }

        let result = store.get("expired").await;
        assert!(result.is_err());

        // Token should have been removed
        let count = store.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_delete() {
        let store = InMemoryRefreshTokenStore::new();
        let expiry = Utc::now() + Duration::hours(1);

        store
            .set("token1", serde_json::json!({}), expiry)
            .await
            .unwrap();

        store.delete("token1").await.unwrap();

        let result = store.get("token1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_delete_empty_token() {
        let store = InMemoryRefreshTokenStore::new();
        // Should not error
        store.delete("").await.unwrap();
    }

    #[tokio::test]
    async fn test_cleanup() {
        let store = InMemoryRefreshTokenStore::new();
        let valid_expiry = Utc::now() + Duration::hours(1);
        let expired_expiry = Utc::now() - Duration::seconds(1);

        store
            .set("valid", serde_json::json!({}), valid_expiry)
            .await
            .unwrap();

        // Insert an expired token directly
        {
            let mut tokens = store.tokens.write().await;
            tokens.insert(
                "expired".to_string(),
                RefreshTokenData {
                    user_data: serde_json::json!({}),
                    expiry: expired_expiry,
                    created: Utc::now() - Duration::hours(1),
                },
            );
        }

        let cleaned = store.cleanup().await.unwrap();
        assert_eq!(cleaned, 1);

        let count = store.count().await.unwrap();
        assert_eq!(count, 1);
    }

    #[tokio::test]
    async fn test_get_all_filters_expired() {
        let store = InMemoryRefreshTokenStore::new();
        let valid_expiry = Utc::now() + Duration::hours(1);

        store
            .set("valid", serde_json::json!({"id": 1}), valid_expiry)
            .await
            .unwrap();

        // Insert an expired token directly
        {
            let mut tokens = store.tokens.write().await;
            tokens.insert(
                "expired".to_string(),
                RefreshTokenData {
                    user_data: serde_json::json!({"id": 2}),
                    expiry: Utc::now() - Duration::seconds(1),
                    created: Utc::now() - Duration::hours(1),
                },
            );
        }

        let all = store.get_all().await;
        assert_eq!(all.len(), 1);
        assert!(all.contains_key("valid"));
    }

    #[tokio::test]
    async fn test_clear() {
        let store = InMemoryRefreshTokenStore::new();
        let expiry = Utc::now() + Duration::hours(1);

        store
            .set("t1", serde_json::json!({}), expiry)
            .await
            .unwrap();
        store
            .set("t2", serde_json::json!({}), expiry)
            .await
            .unwrap();

        store.clear().await;

        let count = store.count().await.unwrap();
        assert_eq!(count, 0);
    }

    #[tokio::test]
    async fn test_new_store() {
        let store = InMemoryRefreshTokenStore::new();
        let count = store.count().await.unwrap();
        assert_eq!(count, 0, "New store should be empty");
    }

    #[tokio::test]
    async fn test_delete_nonexistent() {
        let store = InMemoryRefreshTokenStore::new();
        // Deleting a token that does not exist should succeed without error
        let result = store.delete("nonexistent_token").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_count() {
        let store = InMemoryRefreshTokenStore::new();
        let valid_expiry = Utc::now() + Duration::hours(1);
        let expired_expiry = Utc::now() - Duration::seconds(1);

        // Add 3 valid tokens
        for i in 0..3 {
            store
                .set(
                    &format!("valid{}", i),
                    serde_json::json!({"id": i}),
                    valid_expiry,
                )
                .await
                .unwrap();
        }

        // Add 2 expired tokens directly
        {
            let mut tokens = store.tokens.write().await;
            for i in 0..2 {
                tokens.insert(
                    format!("expired{}", i),
                    RefreshTokenData {
                        user_data: serde_json::json!({"id": i}),
                        expiry: expired_expiry,
                        created: Utc::now() - Duration::hours(1),
                    },
                );
            }
        }

        // count() returns total (including expired)
        let count = store.count().await.unwrap();
        assert_eq!(
            count, 5,
            "Count should include both valid and expired tokens"
        );

        // After cleanup, only valid tokens remain
        let cleaned = store.cleanup().await.unwrap();
        assert_eq!(cleaned, 2);

        let count = store.count().await.unwrap();
        assert_eq!(count, 3, "Count after cleanup should be 3");
    }

    #[tokio::test]
    async fn test_concurrent_access() {
        use std::sync::Arc;

        let store = Arc::new(InMemoryRefreshTokenStore::new());
        let num_tasks = 100usize;

        // Concurrent writes
        let mut handles = Vec::new();
        for i in 0..num_tasks {
            let store = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                let token = format!("token{}", i);
                let user_data = serde_json::json!({"id": i});
                let expiry = Utc::now() + Duration::hours(1);
                store.set(&token, user_data, expiry).await.unwrap();
            }));
        }
        for h in handles {
            h.await.unwrap();
        }

        let count = store.count().await.unwrap();
        assert_eq!(count, num_tasks);

        // Concurrent reads
        let mut handles = Vec::new();
        for i in 0..num_tasks {
            let store = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                let token = format!("token{}", i);
                let result = store.get(&token).await;
                assert!(result.is_ok(), "Failed to get token{}", i);
            }));
        }
        for h in handles {
            h.await.unwrap();
        }

        // Concurrent deletes
        let mut handles = Vec::new();
        for i in 0..num_tasks {
            let store = Arc::clone(&store);
            handles.push(tokio::spawn(async move {
                let token = format!("token{}", i);
                store.delete(&token).await.unwrap();
            }));
        }
        for h in handles {
            h.await.unwrap();
        }

        let count = store.count().await.unwrap();
        assert_eq!(
            count, 0,
            "All tokens should be deleted after concurrent deletes"
        );
    }

    #[tokio::test]
    async fn test_is_expired() {
        // Non-expired token
        let data = RefreshTokenData {
            user_data: serde_json::json!({"user_id": "123"}),
            expiry: Utc::now() + Duration::hours(1),
            created: Utc::now(),
        };
        assert!(
            !data.is_expired(),
            "Token with future expiry should not be expired"
        );

        // Expired token
        let data = RefreshTokenData {
            user_data: serde_json::json!({"user_id": "123"}),
            expiry: Utc::now() - Duration::hours(1),
            created: Utc::now() - Duration::hours(2),
        };
        assert!(
            data.is_expired(),
            "Token with past expiry should be expired"
        );
    }
}
