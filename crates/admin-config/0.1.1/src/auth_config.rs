//! 认证配置模块
//!
//! 提供 JWT Token 相关的配置管理

use serde::{Deserialize, Serialize};

/// 认证配置
///
/// 用于配置 JWT Token 的密钥和过期时间
///
/// # 字段说明
///
/// - `token_secret`: JWT 签名密钥，默认自动生成 32 字节随机密钥
/// - `token_expiry_hours`: Access Token 过期时间（小时），默认 24 小时
/// - `refresh_token_expiry_days`: Refresh Token 过期时间（天），默认 7 天
///
/// # 示例
///
/// ```rust
/// use admin_config::AuthConfig;
///
/// let config = AuthConfig::default();
/// assert_eq!(config.token_expiry_hours, 24);
/// assert_eq!(config.refresh_token_expiry_days, 7);
/// ```
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
            token_secret: Self::generate_token_secret(),
            token_expiry_hours: 24,
            refresh_token_expiry_days: 7,
        }
    }
}

impl AuthConfig {
    /// 获取 Token 过期时间（秒）
    pub fn token_expiry_seconds(&self) -> u64 {
        self.token_expiry_hours * 3600
    }

    /// 获取 Refresh Token 过期时间（秒）
    pub fn refresh_token_expiry_seconds(&self) -> u64 {
        self.refresh_token_expiry_days * 86400
    }

    /// 生成32字节（64位十六进制）的JWT密钥
    fn generate_token_secret() -> String {
        (0..32).map(|_| format!("{:02x}", rand::random::<u8>())).collect()
    }
}
