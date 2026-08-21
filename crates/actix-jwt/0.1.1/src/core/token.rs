//! Token data structures.
//!
//! Mirrors
//! [`core/token.go`](https://github.com/LdDl/echo-jwt/blob/master/core/token.go)
//! from the Go implementation.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Data stored alongside each refresh token in the [`crate::core::TokenStore`].
///
/// # Examples
///
/// ```
/// use chrono::{Duration, Utc};
/// use actix_jwt::RefreshTokenData;
///
/// let data = RefreshTokenData {
///     user_data: serde_json::json!({"user_id": 42}),
///     expiry: Utc::now() + Duration::hours(24),
///     created: Utc::now(),
/// };
/// assert!(!data.is_expired());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshTokenData {
    /// Arbitrary JSON payload associated with the token (e.g. user profile).
    pub user_data: serde_json::Value,
    /// Point in time after which the token is no longer valid.
    pub expiry: DateTime<Utc>,
    /// When the token was originally issued.
    pub created: DateTime<Utc>,
}

impl RefreshTokenData {
    /// Returns `true` when [`Utc::now`] is past the [`expiry`](Self::expiry).
    pub fn is_expired(&self) -> bool {
        Utc::now() > self.expiry
    }
}

/// A complete JWT token pair returned by login / refresh handlers.
///
/// Follows the [RFC 6749 §5.1](https://datatracker.ietf.org/doc/html/rfc6749#section-5.1)
/// response format.
///
/// # Examples
///
/// ```
/// use chrono::{Duration, Utc};
/// use actix_jwt::Token;
///
/// let now = Utc::now();
/// let token = Token {
///     access_token: "eyJ...".to_string(),
///     token_type: "Bearer".to_string(),
///     refresh_token: Some("dGVzdA".to_string()),
///     expires_at: (now + Duration::hours(1)).timestamp(),
///     created_at: now.timestamp(),
/// };
///
/// assert!(token.expires_in() > 0);
/// assert_eq!(token.token_type, "Bearer");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Token {
    /// The signed JWT access token string.
    pub access_token: String,
    /// Token type, typically `"Bearer"`.
    pub token_type: String,
    /// Opaque refresh token (present when refresh-token rotation is enabled).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Unix timestamp (seconds) when the access token expires.
    pub expires_at: i64,
    /// Unix timestamp (seconds) when the token pair was created.
    pub created_at: i64,
}

impl Token {
    /// Returns the number of seconds until the access token expires.
    ///
    /// A negative value means the token has already expired.
    pub fn expires_in(&self) -> i64 {
        self.expires_at - Utc::now().timestamp()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;

    #[test]
    fn test_token_fields() {
        let now = Utc::now();
        let expires_at = (now + Duration::hours(1)).timestamp();
        let created_at = now.timestamp();

        let token = Token {
            access_token: "test.access.token".to_string(),
            token_type: "Bearer".to_string(),
            refresh_token: Some("test-refresh-token".to_string()),
            expires_at,
            created_at,
        };

        assert_eq!(token.access_token, "test.access.token");
        assert_eq!(token.token_type, "Bearer");
        assert_eq!(token.refresh_token, Some("test-refresh-token".to_string()));
        assert_eq!(token.expires_at, expires_at);
        assert_eq!(token.created_at, created_at);
    }

    #[test]
    fn test_token_expires_in() {
        // Future expiry (~30 minutes from now)
        let future_token = Token {
            access_token: String::new(),
            token_type: String::new(),
            refresh_token: None,
            expires_at: (Utc::now() + Duration::minutes(30)).timestamp(),
            created_at: Utc::now().timestamp(),
        };
        let expires_in = future_token.expires_in();
        // Allow 5 seconds tolerance for test execution time
        assert!(
            (expires_in - 1800).abs() <= 5,
            "Future expiry: expires_in={}, expected ~1800",
            expires_in
        );

        // Past expiry (~30 minutes ago)
        let past_token = Token {
            access_token: String::new(),
            token_type: String::new(),
            refresh_token: None,
            expires_at: (Utc::now() - Duration::minutes(30)).timestamp(),
            created_at: Utc::now().timestamp(),
        };
        let expires_in = past_token.expires_in();
        assert!(
            (expires_in + 1800).abs() <= 5,
            "Past expiry: expires_in={}, expected ~-1800",
            expires_in
        );
    }

    #[test]
    fn test_refresh_token_data_is_expired() {
        // Valid (not expired) token
        let valid = RefreshTokenData {
            user_data: serde_json::json!({"user_id": "abc"}),
            expiry: Utc::now() + Duration::hours(1),
            created: Utc::now(),
        };
        assert!(
            !valid.is_expired(),
            "Token with future expiry should not be expired"
        );

        // Expired token
        let expired = RefreshTokenData {
            user_data: serde_json::json!({"user_id": "abc"}),
            expiry: Utc::now() - Duration::hours(1),
            created: Utc::now() - Duration::hours(2),
        };
        assert!(
            expired.is_expired(),
            "Token with past expiry should be expired"
        );
    }
}
