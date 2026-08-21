//! Error types for the ACE SDK.
//!
//! Uses `thiserror` for ergonomic error definitions.

use thiserror::Error;

/// Primary error type for ACE SDK operations.
#[derive(Error, Debug)]
pub enum AceError {
    /// HTTP request failed.
    #[error("HTTP error ({status}): {message}")]
    Http {
        status: u16,
        message: String,
        code: Option<String>,
    },

    /// Authentication error (401/403).
    #[error("Authentication error: {0}")]
    Auth(String),

    /// Token expired (7-day hard cap reached).
    #[error("Session expired (7-day hard cap reached). Please re-authenticate.")]
    TokenExpired,

    /// Quota exceeded (429).
    #[error("Quota exceeded for {resource}: {current}/{limit}. Upgrade at {upgrade_url}")]
    QuotaExceeded {
        code: String,
        resource: String,
        current: u32,
        limit: u32,
        upgrade_url: String,
    },

    /// Feature not available on current plan (403).
    #[error("Feature '{feature}' requires {required_plan}")]
    FeatureNotAvailable {
        code: String,
        feature: String,
        required_plan: String,
        upgrade_url: String,
    },

    /// Payment required - read-only mode (402).
    #[error("Payment required: {message}. {days_until_block} days until block.")]
    PaymentRequired {
        code: String,
        message: String,
        days_until_block: u32,
        upgrade_url: String,
    },

    /// Account blocked (403).
    #[error("Account blocked: {message}")]
    AccountBlocked {
        code: String,
        message: String,
        upgrade_url: String,
    },

    /// Insufficient permissions (403).
    #[error("Insufficient permissions: {message}. Required role: {required_role}")]
    InsufficientPermissions {
        code: String,
        message: String,
        required_role: String,
    },

    /// Configuration error.
    #[error("Configuration error: {0}")]
    Config(String),

    /// Cache (SQLite) error.
    #[error("Cache error: {0}")]
    Cache(String),

    /// Serialization/deserialization error.
    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    /// Network/reqwest error.
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// SQLite error.
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    /// Timeout error.
    #[error("Request timed out after {0}ms")]
    Timeout(u64),

    /// Generic error.
    #[error("{0}")]
    Other(String),
}

impl AceError {
    /// Check if error is authentication related.
    pub fn is_auth_error(&self) -> bool {
        matches!(self, AceError::Auth(_) | AceError::TokenExpired)
    }

    /// Check if error is a rate limit / quota error.
    pub fn is_quota_error(&self) -> bool {
        matches!(self, AceError::QuotaExceeded { .. })
    }

    /// Create an HTTP error from status and response body.
    pub fn from_http_response(status: u16, body: &str) -> Self {
        // Try to parse as JSON error
        if let Ok(data) = serde_json::from_str::<serde_json::Value>(body) {
            let error_type = data.get("error").and_then(|v| v.as_str());
            let message = data.get("message").and_then(|v| v.as_str()).unwrap_or(body);

            match (status, error_type) {
                (429, Some("quota_exceeded")) => {
                    return AceError::QuotaExceeded {
                        code: data
                            .get("code")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        resource: data
                            .get("resource")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        current: data.get("current").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        limit: data.get("limit").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                        upgrade_url: data
                            .get("upgrade_url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    };
                }
                (403, Some("feature_not_available")) => {
                    return AceError::FeatureNotAvailable {
                        code: data
                            .get("code")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        feature: data
                            .get("feature")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        required_plan: data
                            .get("required_plan")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        upgrade_url: data
                            .get("upgrade_url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    };
                }
                (402, Some("payment_required")) => {
                    return AceError::PaymentRequired {
                        code: data
                            .get("code")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        message: message.to_string(),
                        days_until_block: data
                            .get("days_until_block")
                            .and_then(|v| v.as_u64())
                            .unwrap_or(0) as u32,
                        upgrade_url: data
                            .get("upgrade_url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    };
                }
                (403, Some("account_blocked")) => {
                    return AceError::AccountBlocked {
                        code: data
                            .get("code")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        message: message.to_string(),
                        upgrade_url: data
                            .get("upgrade_url")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    };
                }
                (403, Some("insufficient_permissions")) => {
                    return AceError::InsufficientPermissions {
                        code: data
                            .get("code")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                        message: message.to_string(),
                        required_role: data
                            .get("required_role")
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string(),
                    };
                }
                (401, _) => {
                    return AceError::Auth(message.to_string());
                }
                _ => {
                    return AceError::Http {
                        status,
                        message: message.to_string(),
                        code: data
                            .get("code")
                            .and_then(|v| v.as_str())
                            .map(|s| s.to_string()),
                    };
                }
            }
        }

        AceError::Http {
            status,
            message: body.to_string(),
            code: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_http_response_quota() {
        let body = r#"{"error":"quota_exceeded","code":"PATTERNS_LIMIT","resource":"patterns","current":50,"limit":50,"upgrade_url":"https://example.com","message":"Quota exceeded"}"#;
        let err = AceError::from_http_response(429, body);
        assert!(err.is_quota_error());
    }

    #[test]
    fn test_from_http_response_auth() {
        let body = r#"{"message":"Unauthorized"}"#;
        let err = AceError::from_http_response(401, body);
        assert!(err.is_auth_error());
    }

    #[test]
    fn test_from_http_response_plain_text() {
        let err = AceError::from_http_response(500, "Internal Server Error");
        match err {
            AceError::Http { status, .. } => assert_eq!(status, 500),
            _ => panic!("Expected Http error"),
        }
    }
}
