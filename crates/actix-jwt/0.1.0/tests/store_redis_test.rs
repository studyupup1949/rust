#![cfg(feature = "redis-store")]

use actix_jwt::core::TokenStore;
use actix_jwt::errors::JwtError;
use actix_jwt::store::redis::{RedisConfig, RedisRefreshTokenStore};

use chrono::{Duration, Utc};
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::redis::Redis;

/// Starts a Redis container and returns the container handle and connection URL.
async fn start_redis() -> (testcontainers::ContainerAsync<Redis>, String) {
    let container = Redis::default().start().await.unwrap();
    let port = container.get_host_port_ipv4(6379).await.unwrap();
    let url = format!("redis://127.0.0.1:{}/", port);
    (container, url)
}

/// Helper to create a RedisRefreshTokenStore connected to the test container.
async fn create_store(url: &str) -> RedisRefreshTokenStore {
    let config = RedisConfig {
        addr: url.to_string(),
        key_prefix: "test-jwt:".to_string(),
        ..Default::default()
    };
    RedisRefreshTokenStore::new(&config).await.unwrap()
}

#[tokio::test]
async fn test_redis_basic_operations() {
    let (_container, url) = start_redis().await;
    let store = create_store(&url).await;

    let token = "test-token-basic";
    let user_data = serde_json::json!({"user_id": 123, "username": "testuser"});
    let expiry = Utc::now() + Duration::hours(1);

    // Set
    store.set(token, user_data.clone(), expiry).await.unwrap();

    // Get
    let retrieved = store.get(token).await.unwrap();
    assert_eq!(retrieved, user_data);

    // Delete
    store.delete(token).await.unwrap();

    // Verify deletion
    let result = store.get(token).await;
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), JwtError::RefreshTokenNotFound),
        "Token should not be found after deletion"
    );

    // Ping
    store.ping().await.unwrap();
}

#[tokio::test]
async fn test_redis_get_nonexistent() {
    let (_container, url) = start_redis().await;
    let store = create_store(&url).await;

    let result = store.get("nonexistent-token").await;
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), JwtError::RefreshTokenNotFound),
        "Nonexistent token should return RefreshTokenNotFound"
    );
}

#[tokio::test]
async fn test_redis_empty_token() {
    let (_container, url) = start_redis().await;
    let store = create_store(&url).await;

    let expiry = Utc::now() + Duration::hours(1);

    // Set with empty token
    let result = store.set("", serde_json::json!({}), expiry).await;
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), JwtError::TokenEmpty),
        "Set with empty token should return TokenEmpty"
    );

    // Get with empty token
    let result = store.get("").await;
    assert!(result.is_err());
    assert!(
        matches!(result.unwrap_err(), JwtError::TokenEmpty),
        "Get with empty token should return TokenEmpty"
    );

    // Delete with empty token should succeed (no-op)
    store.delete("").await.unwrap();
}

#[tokio::test]
async fn test_redis_expiration() {
    let (_container, url) = start_redis().await;
    let store = create_store(&url).await;

    let token = "test-token-expiry";
    let user_data = serde_json::json!("test-data");

    // Set token with short expiry (2 seconds)
    let short_expiry = Utc::now() + Duration::seconds(2);
    store
        .set(token, user_data.clone(), short_expiry)
        .await
        .unwrap();

    // Token should be available immediately
    let retrieved = store.get(token).await.unwrap();
    assert_eq!(retrieved, user_data);

    // Wait for expiration
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Token should be gone
    let result = store.get(token).await;
    assert!(
        result.is_err(),
        "Token should not be accessible after expiry"
    );
}

#[tokio::test]
async fn test_redis_cleanup() {
    let (_container, url) = start_redis().await;
    let store = create_store(&url).await;

    // Flush to start clean
    store.flush_db().await.unwrap();

    let tokens = ["cleanup-token-1", "cleanup-token-2", "cleanup-token-3"];
    let user_data = serde_json::json!("cleanup-data");

    // Set two tokens with very short expiry
    let short_expiry = Utc::now() + Duration::seconds(2);
    for token in &tokens[..2] {
        store
            .set(token, user_data.clone(), short_expiry)
            .await
            .unwrap();
    }

    // Set one token with future expiry
    let future_expiry = Utc::now() + Duration::hours(1);
    store
        .set(tokens[2], user_data.clone(), future_expiry)
        .await
        .unwrap();

    // Wait for the short-lived tokens to expire
    tokio::time::sleep(std::time::Duration::from_secs(3)).await;

    // Run cleanup
    let cleaned = store.cleanup().await.unwrap();
    assert!(cleaned >= 0, "Cleanup should return non-negative count");

    // Verify that non-expired token still exists
    let result = store.get(tokens[2]).await;
    assert!(
        result.is_ok(),
        "Non-expired token should still exist after cleanup"
    );
}

#[tokio::test]
async fn test_redis_count() {
    let (_container, url) = start_redis().await;
    let store = create_store(&url).await;

    // Flush to start clean
    store.flush_db().await.unwrap();

    let initial_count = store.count().await.unwrap();
    assert_eq!(initial_count, 0, "Count should be 0 after flush");

    // Add tokens
    let expiry = Utc::now() + Duration::hours(1);
    let keys = ["count-token-1", "count-token-2", "count-token-3"];

    for (i, token) in keys.iter().enumerate() {
        store
            .set(token, serde_json::json!({"index": i}), expiry)
            .await
            .unwrap();
    }

    let new_count = store.count().await.unwrap();
    assert_eq!(
        new_count,
        initial_count + keys.len(),
        "Count should include new tokens"
    );

    // Clean up
    for token in &keys {
        store.delete(token).await.unwrap();
    }
}

#[tokio::test]
async fn test_redis_connection_failure() {
    // Use 127.0.0.1:1 to get instant connection refused instead of DNS timeout.
    // Wrap in timeout because ConnectionManager retries internally.
    let config = RedisConfig {
        addr: "redis://127.0.0.1:1/".to_string(),
        ..Default::default()
    };

    let result = tokio::time::timeout(
        std::time::Duration::from_secs(2),
        RedisRefreshTokenStore::new(&config),
    )
    .await;

    let result = match result {
        Ok(r) => r,
        Err(_) => Err(JwtError::Internal("timeout".into())),
    };
    assert!(
        result.is_err(),
        "Should return error for invalid Redis configuration"
    );

    let err_msg = result.unwrap_err().to_string();
    assert!(
        err_msg.contains("Redis") || err_msg.contains("redis") || err_msg.contains("timeout"),
        "Error should mention Redis or timeout, got: {}",
        err_msg
    );
}

#[tokio::test]
async fn test_redis_invalid_token_past_expiry() {
    let (_container, url) = start_redis().await;
    let store = create_store(&url).await;

    // Try setting a token with expiry in the past
    let past_expiry = Utc::now() - Duration::hours(1);
    let result = store
        .set("expired-token", serde_json::json!("data"), past_expiry)
        .await;

    assert!(result.is_err(), "Should return error for past expiry");
    assert!(
        matches!(result.unwrap_err(), JwtError::ExpiryInPast),
        "Error should be ExpiryInPast"
    );
}

#[tokio::test]
async fn test_redis_default_config() {
    let config = RedisConfig::default();

    assert_eq!(config.addr, "redis://127.0.0.1:6379/");
    assert!(config.password.is_none(), "Default password should be None");
    assert_eq!(config.db, 0, "Default DB should be 0");
    assert_eq!(config.pool_size, 10, "Default pool_size should be 10");
    assert_eq!(
        config.key_prefix, "actix-jwt:",
        "Default key prefix should be actix-jwt:"
    );
    assert!(!config.tls, "Default TLS should be false");
}

#[tokio::test]
async fn test_redis_flush_db() {
    let (_container, url) = start_redis().await;
    let store = create_store(&url).await;

    let expiry = Utc::now() + Duration::hours(1);

    // Add some tokens
    store
        .set("flush-token-1", serde_json::json!("data1"), expiry)
        .await
        .unwrap();
    store
        .set("flush-token-2", serde_json::json!("data2"), expiry)
        .await
        .unwrap();

    let count = store.count().await.unwrap();
    assert!(count >= 2, "Should have at least 2 tokens");

    // Flush
    store.flush_db().await.unwrap();

    let count = store.count().await.unwrap();
    assert_eq!(count, 0, "Count should be 0 after flush");
}
