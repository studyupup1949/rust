//! 验证码配置模块
//!
//! 提供验证码生成和发送的相关配置

use serde::{Deserialize, Serialize};

/// 验证码配置
///
/// 用于配置验证码的长度、有效期和发送限制
///
/// # 字段说明
///
/// - `length`: 验证码长度（字符数），默认 6
/// - `ttl`: 验证码有效期（秒），默认 300 秒（5 分钟）
/// - `send_interval`: 两次发送的最小间隔（秒），默认 60 秒
///
/// # 示例
///
/// ```rust
/// use admin_config::VerificationCodeConfig;
///
/// let config = VerificationCodeConfig::default();
/// assert_eq!(config.length, 6);
/// assert_eq!(config.ttl, 300);
/// assert_eq!(config.send_interval, 60);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerificationCodeConfig {
    /// 验证码长度
    pub length: usize,
    /// 过期时间(秒)
    pub ttl: u64,
    /// 发送间隔(秒)
    pub send_interval: u64,
}

impl Default for VerificationCodeConfig {
    fn default() -> Self {
        Self {
            length: 6,
            ttl: 300,
            send_interval: 60,
        }
    }
}
