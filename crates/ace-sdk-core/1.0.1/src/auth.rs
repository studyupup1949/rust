//! Device code authentication flow (RFC 8628).
//!
//! Implements OAuth 2.0 Device Authorization Grant for CLI/terminal clients.
//!
//! Flow:
//! 1. Client requests device code: POST /api/v1/auth/device
//! 2. User visits URL and enters code
//! 3. Client polls for token: POST /api/v1/auth/device/token
//! 4. On success, save credentials to config

use crate::config::load_config;
use crate::errors::AceError;
use crate::logger::ILogger;
use crate::types::{CurrentUser, DeviceCodeResponse, TokenResponse};

/// Default server URL.
const DEFAULT_SERVER_URL: &str = "https://ace-api.code-engine.app";

/// Get server URL from config or default.
fn get_server_url() -> String {
    load_config(Default::default(), None)
        .map(|c| c.server_url)
        .unwrap_or_else(|_| DEFAULT_SERVER_URL.to_string())
}

/// Request device code from server.
///
/// POST /api/v1/auth/device
pub async fn request_device_code(
    client_type: &str,
    device_id: Option<&str>,
) -> Result<DeviceCodeResponse, AceError> {
    let server_url = get_server_url();
    let url = format!("{}/api/v1/auth/device", server_url);

    let mut body = serde_json::json!({
        "client_type": client_type
    });

    if let Some(did) = device_id {
        body["device_id"] = serde_json::json!(did);
    }

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        return Err(AceError::Auth(format!(
            "Failed to request device code: {}",
            error_text
        )));
    }

    let device_code: DeviceCodeResponse = response.json().await?;
    Ok(device_code)
}

/// Poll for token after user authorizes.
///
/// POST /api/v1/auth/device/token
pub async fn poll_for_token(device_code: &str) -> Result<PollResult, AceError> {
    let server_url = get_server_url();
    let url = format!("{}/api/v1/auth/device/token", server_url);

    let body = serde_json::json!({
        "device_code": device_code
    });

    let client = reqwest::Client::new();
    let response = client
        .post(&url)
        .header("Content-Type", "application/json")
        .json(&body)
        .send()
        .await?;

    let raw: serde_json::Value = response.json().await?;

    // Server wraps error responses in 'detail' object
    let data = raw.get("detail").cloned().unwrap_or(raw);

    // Check for error response
    if let Some(error) = data.get("error").and_then(|v| v.as_str()) {
        let description = data
            .get("error_description")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();
        return Ok(PollResult::Error {
            error: error.to_string(),
            description,
        });
    }

    // Parse as token response
    let token_response: TokenResponse = serde_json::from_value(data)?;
    Ok(PollResult::Success(token_response))
}

/// Result of polling for a token.
#[derive(Debug)]
#[allow(clippy::large_enum_variant)]
pub enum PollResult {
    /// Token successfully obtained.
    Success(TokenResponse),
    /// Error during polling (may be retryable).
    Error { error: String, description: String },
}

/// Login options for device code flow.
#[derive(Debug, Clone)]
pub struct LoginOptions {
    /// Client type for analytics.
    pub client_type: String,
    /// Timeout in milliseconds (default: 300000 = 5 minutes).
    pub timeout_ms: u64,
    /// Skip opening browser.
    pub no_browser: bool,
}

impl Default for LoginOptions {
    fn default() -> Self {
        Self {
            client_type: "cli".to_string(),
            timeout_ms: 300_000,
            no_browser: false,
        }
    }
}

/// Complete device code login flow.
///
/// 1. Request device code
/// 2. Display code to user (via callback)
/// 3. Poll for token
/// 4. Return authenticated user
pub async fn login(
    options: LoginOptions,
    on_user_code: impl Fn(&str, &str),
    on_progress: impl Fn(&str),
    _logger: Option<&dyn ILogger>,
) -> Result<CurrentUser, AceError> {
    on_progress("Requesting device code...");

    let device_code = request_device_code(&options.client_type, None).await?;

    // Show code to user
    on_user_code(
        &device_code.user_code,
        &device_code.verification_uri_complete,
    );

    // Poll for token
    on_progress("Waiting for authorization...");

    let mut interval_ms = device_code.interval * 1000;
    let start = std::time::Instant::now();
    let timeout = std::time::Duration::from_millis(options.timeout_ms);
    let expires_at = std::time::Duration::from_secs(device_code.expires_in);

    loop {
        if start.elapsed() > timeout {
            return Err(AceError::Other("Login timed out".to_string()));
        }
        if start.elapsed() > expires_at {
            return Err(AceError::Other(
                "Device code expired. Please try again.".to_string(),
            ));
        }

        tokio::time::sleep(std::time::Duration::from_millis(interval_ms)).await;

        match poll_for_token(&device_code.device_code).await? {
            PollResult::Success(token_response) => {
                let user_data = token_response.resolved_user().map_err(AceError::Other)?;

                let current_user = CurrentUser {
                    user_id: user_data.user_id,
                    email: user_data.email,
                    name: user_data.name,
                    image_url: user_data.image_url,
                    organizations: token_response.organizations,
                    default_org_id: None,
                    authenticated_at: Some(chrono::Utc::now().to_rfc3339()),
                };

                return Ok(current_user);
            }
            PollResult::Error { error, description } => match error.as_str() {
                "authorization_pending" => continue,
                "slow_down" => {
                    interval_ms += 5000;
                    continue;
                }
                "expired_token" => {
                    return Err(AceError::Other(
                        "Device code expired. Please try again.".to_string(),
                    ));
                }
                "access_denied" => {
                    return Err(AceError::Other("Authorization denied by user.".to_string()));
                }
                _ => {
                    return Err(AceError::Auth(if description.is_empty() {
                        format!("Auth error: {}", error)
                    } else {
                        description
                    }));
                }
            },
        }
    }
}

/// Logout - clear credentials.
pub fn logout(_logger: Option<&dyn ILogger>) {
    // In a full implementation, this would clear the config file auth section.
    // For now, this is a placeholder.
}

// =============================================================================
// Auth Utility Functions
// =============================================================================

/// Mask token for display (show first 15 chars + ...).
pub fn mask_token(token: &str) -> String {
    if token.is_empty() {
        return "(none)".to_string();
    }
    if token.len() <= 15 {
        return token.to_string();
    }
    format!("{}...", &token[..15])
}

/// Get effective API token from config file.
///
/// Priority:
/// 1. Environment: ACE_API_TOKEN
/// 2. Config: auth.token (user token)
/// 3. Config: apiToken (legacy org token)
pub fn get_effective_token() -> Option<String> {
    // Priority 1: Environment variable
    if let Ok(token) = std::env::var("ACE_API_TOKEN") {
        if !token.is_empty() {
            return Some(token);
        }
    }

    // Priority 2 & 3: Config file
    let config_path = crate::config::get_xdg_config_path();
    if !config_path.exists() {
        return None;
    }

    let contents = std::fs::read_to_string(&config_path).ok()?;
    let config: serde_json::Value = serde_json::from_str(&contents).ok()?;

    // Priority 2: User token (new auth block)
    if let Some(token) = config
        .get("auth")
        .and_then(|a| a.get("token"))
        .and_then(|t| t.as_str())
    {
        if !token.is_empty() {
            return Some(token.to_string());
        }
    }

    // Priority 3: Legacy org token
    if let Some(token) = config.get("apiToken").and_then(|t| t.as_str()) {
        if !token.is_empty() {
            return Some(token.to_string());
        }
    }

    None
}

/// Get effective organization ID.
///
/// Priority:
/// 1. Environment: ACE_ORG_ID
/// 2. Config: default_org_id
/// 3. First org in auth.organizations
pub fn get_effective_org_id() -> Option<String> {
    // Priority 1: Environment variable
    if let Ok(org_id) = std::env::var("ACE_ORG_ID") {
        if !org_id.is_empty() {
            return Some(org_id);
        }
    }

    // Load config file
    let config_path = crate::config::get_xdg_config_path();
    if !config_path.exists() {
        return None;
    }

    let contents = std::fs::read_to_string(&config_path).ok()?;
    let config: serde_json::Value = serde_json::from_str(&contents).ok()?;

    // Priority 2: Explicit default_org_id
    if let Some(org_id) = config.get("default_org_id").and_then(|v| v.as_str()) {
        if !org_id.is_empty() {
            return Some(org_id.to_string());
        }
    }

    // Priority 3: First org from user auth
    if let Some(orgs) = config
        .get("auth")
        .and_then(|a| a.get("organizations"))
        .and_then(|o| o.as_array())
    {
        if let Some(first_org) = orgs.first() {
            if let Some(org_id) = first_org.get("org_id").and_then(|v| v.as_str()) {
                return Some(org_id.to_string());
            }
        }
    }

    None
}

/// Token status information for UI display.
#[derive(Debug, Clone)]
pub struct TokenStatus {
    pub has_token: bool,
    pub is_access_expired: bool,
    pub is_refresh_expired: bool,
    pub is_hard_cap_expired: bool,
    pub access_expires_at: Option<String>,
    pub refresh_expires_at: Option<String>,
    pub absolute_expires_at: Option<String>,
    pub can_auto_refresh: bool,
    pub needs_re_login: bool,
}

/// Check if an ISO 8601 timestamp is in the past.
fn is_expired(timestamp: &str) -> bool {
    chrono::DateTime::parse_from_rfc3339(timestamp)
        .map(|dt| dt.timestamp_millis() < chrono::Utc::now().timestamp_millis())
        .unwrap_or(false)
}

/// Get detailed token status information.
pub fn get_token_status() -> TokenStatus {
    let config_path = crate::config::get_xdg_config_path();
    if !config_path.exists() {
        return TokenStatus {
            has_token: false,
            is_access_expired: false,
            is_refresh_expired: false,
            is_hard_cap_expired: false,
            access_expires_at: None,
            refresh_expires_at: None,
            absolute_expires_at: None,
            can_auto_refresh: false,
            needs_re_login: true,
        };
    }

    let contents = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => {
            return TokenStatus {
                has_token: false,
                is_access_expired: false,
                is_refresh_expired: false,
                is_hard_cap_expired: false,
                access_expires_at: None,
                refresh_expires_at: None,
                absolute_expires_at: None,
                can_auto_refresh: false,
                needs_re_login: true,
            };
        }
    };

    let config: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(c) => c,
        Err(_) => {
            return TokenStatus {
                has_token: false,
                is_access_expired: false,
                is_refresh_expired: false,
                is_hard_cap_expired: false,
                access_expires_at: None,
                refresh_expires_at: None,
                absolute_expires_at: None,
                can_auto_refresh: false,
                needs_re_login: true,
            };
        }
    };

    let auth = match config.get("auth") {
        Some(a) => a,
        None => {
            return TokenStatus {
                has_token: false,
                is_access_expired: false,
                is_refresh_expired: false,
                is_hard_cap_expired: false,
                access_expires_at: None,
                refresh_expires_at: None,
                absolute_expires_at: None,
                can_auto_refresh: false,
                needs_re_login: true,
            };
        }
    };

    let has_token = auth
        .get("token")
        .and_then(|t| t.as_str())
        .map(|t| !t.is_empty())
        .unwrap_or(false);
    if !has_token {
        return TokenStatus {
            has_token: false,
            is_access_expired: false,
            is_refresh_expired: false,
            is_hard_cap_expired: false,
            access_expires_at: None,
            refresh_expires_at: None,
            absolute_expires_at: None,
            can_auto_refresh: false,
            needs_re_login: true,
        };
    }

    let access_expires = auth
        .get("expires_at")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let refresh_expires = auth
        .get("refresh_expires_at")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let absolute_expires = auth
        .get("absolute_expires_at")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());
    let has_refresh = auth
        .get("refresh_token")
        .and_then(|t| t.as_str())
        .map(|t| !t.is_empty())
        .unwrap_or(false);

    let is_access_expired = access_expires.as_deref().map(is_expired).unwrap_or(false);
    let is_refresh_expired = refresh_expires.as_deref().map(is_expired).unwrap_or(false);
    let is_hard_cap_expired = absolute_expires.as_deref().map(is_expired).unwrap_or(false);
    let can_auto_refresh = !is_refresh_expired && !is_hard_cap_expired && has_refresh;
    let needs_re_login = is_hard_cap_expired || is_refresh_expired;

    TokenStatus {
        has_token,
        is_access_expired,
        is_refresh_expired,
        is_hard_cap_expired,
        access_expires_at: access_expires,
        refresh_expires_at: refresh_expires,
        absolute_expires_at: absolute_expires,
        can_auto_refresh,
        needs_re_login,
    }
}

/// Check if user is authenticated (local check).
///
/// Performs a fast, offline check that includes:
/// 1. Token existence
/// 2. Local expiry timestamps (hard cap and refresh token)
pub fn is_authenticated() -> bool {
    if get_effective_token().is_none() {
        return false;
    }

    // Check local expiry - if definitely expired, return false
    if is_token_locally_expired() == Some(true) {
        return false;
    }

    true
}

/// Check if using user token (vs legacy org token).
pub fn is_user_authenticated() -> bool {
    match get_effective_token() {
        Some(token) => crate::types::detect_token_type(&token) == crate::types::TokenType::User,
        None => false,
    }
}

/// Get token type of current effective token.
pub fn get_token_type_of_current() -> crate::types::TokenType {
    match get_effective_token() {
        Some(token) => crate::types::detect_token_type(&token),
        None => crate::types::TokenType::Unknown,
    }
}

/// Check if token is locally expired (based on stored timestamps).
///
/// Returns `None` if no expiry info available (legacy tokens).
/// Returns `Some(true)` if hard cap or refresh token is expired.
/// Returns `Some(false)` if session can continue (via auto-refresh or still valid).
pub fn is_token_locally_expired() -> Option<bool> {
    let config_path = crate::config::get_xdg_config_path();
    if !config_path.exists() {
        return None;
    }

    let contents = std::fs::read_to_string(&config_path).ok()?;
    let config: serde_json::Value = serde_json::from_str(&contents).ok()?;
    let auth = config.get("auth")?;

    // Check hard cap first (absolute_expires_at)
    if let Some(abs_exp) = auth.get("absolute_expires_at").and_then(|v| v.as_str()) {
        if is_expired(abs_exp) {
            return Some(true);
        }
    }

    // Check refresh token expiry
    if let Some(ref_exp) = auth.get("refresh_expires_at").and_then(|v| v.as_str()) {
        if is_expired(ref_exp) {
            return Some(true);
        }
    }

    // If we have expiry info and neither is expired, token is still usable
    let has_absolute = auth
        .get("absolute_expires_at")
        .and_then(|v| v.as_str())
        .is_some();
    let has_refresh = auth
        .get("refresh_expires_at")
        .and_then(|v| v.as_str())
        .is_some();
    if has_absolute || has_refresh {
        return Some(false);
    }

    // No expiry info (legacy token)
    None
}

/// Get current user info from config file.
pub fn get_current_user() -> Option<crate::types::CurrentUser> {
    let config_path = crate::config::get_xdg_config_path();
    if !config_path.exists() {
        return None;
    }

    let contents = std::fs::read_to_string(&config_path).ok()?;
    let config: serde_json::Value = serde_json::from_str(&contents).ok()?;
    let auth = config.get("auth")?;

    let user_auth: crate::types::UserAuth = serde_json::from_value(auth.clone()).ok()?;

    Some(crate::types::CurrentUser {
        user_id: user_auth.user_id,
        email: user_auth.email,
        name: None,
        image_url: None,
        organizations: user_auth.organizations,
        default_org_id: config
            .get("default_org_id")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string()),
        authenticated_at: user_auth.authenticated_at,
    })
}

/// Result of async authentication check.
#[derive(Debug, Clone)]
pub struct AuthValidationResult {
    pub authenticated: bool,
    pub reason: Option<String>,
    pub user: Option<crate::types::CurrentUser>,
}

/// Check authentication status with server validation.
///
/// Makes a server request to validate the token. Use this when you need
/// authoritative confirmation that the session is still valid.
pub async fn is_authenticated_async() -> AuthValidationResult {
    let token = match get_effective_token() {
        Some(t) => t,
        None => {
            return AuthValidationResult {
                authenticated: false,
                reason: Some("no_token".to_string()),
                user: None,
            };
        }
    };

    // Check local expiry first
    if is_token_locally_expired() == Some(true) {
        return AuthValidationResult {
            authenticated: false,
            reason: Some("token_expired".to_string()),
            user: None,
        };
    }

    // Try server validation
    let server_url = crate::config::load_config(Default::default(), None)
        .map(|c| c.server_url)
        .unwrap_or_else(|_| "https://ace-api.code-engine.app".to_string());

    let client = reqwest::Client::new();
    match client
        .get(format!("{}/api/v1/config/verify", server_url))
        .header("Authorization", format!("Bearer {}", token))
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                AuthValidationResult {
                    authenticated: true,
                    reason: None,
                    user: get_current_user(),
                }
            } else {
                AuthValidationResult {
                    authenticated: false,
                    reason: Some("session_expired".to_string()),
                    user: None,
                }
            }
        }
        Err(_) => {
            // Network error - fall back to local check
            AuthValidationResult {
                authenticated: is_authenticated(),
                reason: Some("server_error".to_string()),
                user: None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    // Serialize tests that mutate process-wide env vars to avoid races
    // between parallel test threads (ACE_API_TOKEN, ACE_ORG_ID, XDG_CONFIG_HOME).
    static ENV_LOCK: Mutex<()> = Mutex::new(());

    fn env_guard() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }

    #[test]
    fn test_login_options_default() {
        let opts = LoginOptions::default();
        assert_eq!(opts.client_type, "cli");
        assert_eq!(opts.timeout_ms, 300_000);
        assert!(!opts.no_browser);
    }

    #[test]
    fn test_mask_token_empty() {
        assert_eq!(mask_token(""), "(none)");
    }

    #[test]
    fn test_mask_token_short() {
        assert_eq!(mask_token("abc"), "abc");
    }

    #[test]
    fn test_mask_token_long() {
        assert_eq!(mask_token("ace_user_test12345678"), "ace_user_test12...");
    }

    #[test]
    fn test_token_status_no_config() {
        let _g = env_guard();
        // With no config file at the default path, should return no token
        std::env::remove_var("ACE_API_TOKEN");
        std::env::set_var("XDG_CONFIG_HOME", "/nonexistent/path");
        let status = get_token_status();
        assert!(!status.has_token);
        assert!(status.needs_re_login);
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_is_token_locally_expired_no_config() {
        let _g = env_guard();
        std::env::set_var("XDG_CONFIG_HOME", "/nonexistent/path");
        let result = is_token_locally_expired();
        assert_eq!(result, None);
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_get_current_user_no_config() {
        let _g = env_guard();
        std::env::set_var("XDG_CONFIG_HOME", "/nonexistent/path");
        let result = get_current_user();
        assert!(result.is_none());
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_is_authenticated_no_token() {
        let _g = env_guard();
        std::env::remove_var("ACE_API_TOKEN");
        std::env::set_var("XDG_CONFIG_HOME", "/nonexistent/path");
        assert!(!is_authenticated());
        std::env::remove_var("XDG_CONFIG_HOME");
    }

    #[test]
    fn test_is_authenticated_with_env_token() {
        let _g = env_guard();
        std::env::set_var("ACE_API_TOKEN", "ace_user_test123");
        assert!(is_authenticated());
        std::env::remove_var("ACE_API_TOKEN");
    }

    #[test]
    fn test_is_user_authenticated_with_user_token() {
        let _g = env_guard();
        std::env::set_var("ACE_API_TOKEN", "ace_user_test123");
        assert!(is_user_authenticated());
        std::env::remove_var("ACE_API_TOKEN");
    }

    #[test]
    fn test_is_user_authenticated_with_org_token() {
        let _g = env_guard();
        std::env::set_var("ACE_API_TOKEN", "ace_12345678test");
        assert!(!is_user_authenticated());
        std::env::remove_var("ACE_API_TOKEN");
    }

    #[test]
    fn test_get_token_type_user() {
        let _g = env_guard();
        std::env::set_var("ACE_API_TOKEN", "ace_user_test123");
        assert_eq!(get_token_type_of_current(), crate::types::TokenType::User);
        std::env::remove_var("ACE_API_TOKEN");
    }

    #[test]
    fn test_get_token_type_org() {
        let _g = env_guard();
        std::env::set_var("ACE_API_TOKEN", "ace_12345678test");
        assert_eq!(get_token_type_of_current(), crate::types::TokenType::Org);
        std::env::remove_var("ACE_API_TOKEN");
    }

    #[test]
    fn test_get_effective_token_from_env() {
        let _g = env_guard();
        std::env::set_var("ACE_API_TOKEN", "ace_user_envtoken");
        let token = get_effective_token();
        assert_eq!(token, Some("ace_user_envtoken".to_string()));
        std::env::remove_var("ACE_API_TOKEN");
    }

    #[test]
    fn test_get_effective_org_id_from_env() {
        let _g = env_guard();
        std::env::set_var("ACE_ORG_ID", "org_testenv");
        let org_id = get_effective_org_id();
        assert_eq!(org_id, Some("org_testenv".to_string()));
        std::env::remove_var("ACE_ORG_ID");
    }

    #[test]
    fn test_auth_validation_result_struct() {
        let result = AuthValidationResult {
            authenticated: true,
            reason: None,
            user: None,
        };
        assert!(result.authenticated);
        assert!(result.reason.is_none());
    }

    #[tokio::test]
    async fn test_is_authenticated_async_no_token() {
        let _g = env_guard();
        std::env::remove_var("ACE_API_TOKEN");
        std::env::set_var("XDG_CONFIG_HOME", "/nonexistent/path");
        let result = is_authenticated_async().await;
        assert!(!result.authenticated);
        assert_eq!(result.reason, Some("no_token".to_string()));
        std::env::remove_var("XDG_CONFIG_HOME");
    }
}
