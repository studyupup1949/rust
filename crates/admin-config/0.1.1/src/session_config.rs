//! Session 配置模块
//!
//! 提供会话管理相关的配置

use serde::{Deserialize, Serialize};

/// Session 配置
///
/// 用于配置服务器端会话和 Cookie 行为
///
/// # 字段说明
///
/// - `secret_key`: Session 加密密钥，默认自动生成 32 字节随机密钥
/// - `max_age`: Session 过期时间（秒），默认 86400 秒（24 小时）
/// - `http_only`: Cookie 是否仅允许 HTTP 访问，禁止 JavaScript 读取，默认 true
/// - `secure`: Cookie 是否仅在 HTTPS 下传输，默认 false
/// - `cookie_path`: Cookie 有效路径，默认 "/"
/// - `cookie_domain`: Cookie 有效域名，默认 None（当前域名）
///
/// # 示例
///
/// ```rust
/// use admin_config::SessionConfig;
///
/// let config = SessionConfig::default();
/// assert_eq!(config.max_age, 86400);
/// assert!(config.http_only);
/// assert_eq!(config.cookie_path, "/");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session 密钥
    pub secret_key: String,
    /// 过期时间(秒)
    pub max_age: u64,
    /// 是否仅 HTTP 访问
    pub http_only: bool,
    /// 是否仅 HTTPS 访问
    pub secure: bool,
    /// Cookie 路径
    pub cookie_path: String,
    /// Cookie 域名
    pub cookie_domain: Option<String>,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            secret_key: Self::generate_secret_key(),
            max_age: 86400,
            http_only: true,
            secure: false,
            cookie_path: "/".to_string(),
            cookie_domain: None,
        }
    }
}

impl SessionConfig {
    /// 生成32字节（64位十六进制）的会话密钥
    fn generate_secret_key() -> String {
        (0..32).map(|_| format!("{:02x}", rand::random::<u8>())).collect()
    }
}
