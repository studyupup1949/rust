use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// JWT 密钥
    pub token_secret: String,
    /// Token 过期时间(小时)
    pub token_expiry_hours: u64,
    /// Refresh Token 过期时间(天)
    pub refresh_token_expiry_days: u64,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            token_secret: "".to_string(),
            token_expiry_hours: 24,
            refresh_token_expiry_days: 7,
        }
    }
}

impl AuthConfig {
    pub fn token_expiry_seconds(&self) -> u64 {
        self.token_expiry_hours * 3600
    }

    pub fn refresh_token_expiry_seconds(&self) -> u64 {
        self.refresh_token_expiry_days * 86400
    }
}
