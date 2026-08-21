//! 邮件配置模块
//!
//! 提供 SMTP 邮件发送相关配置

use serde::{Deserialize, Serialize};

/// 邮件配置
///
/// 用于配置 SMTP 邮件服务器和发件人信息
///
/// # 字段说明
///
/// - `smtp_host`: SMTP 服务器地址
/// - `smtp_port`: SMTP 端口，默认 587（TLS）或 465（SSL）
/// - `smtp_username`: SMTP 登录用户名
/// - `smtp_password`: SMTP 登录密码
/// - `from_name`: 发件人显示名称
/// - `from_email`: 发件人邮箱地址
/// - `enable_tls`: 是否启用 TLS 加密，默认 true
///
/// # 示例
///
/// ```rust
/// use admin_config::EmailConfig;
///
/// let config = EmailConfig::default();
/// assert_eq!(config.smtp_port, 587);
/// assert!(config.enable_tls);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailConfig {
    /// SMTP 服务器地址
    pub smtp_host: String,
    /// SMTP 端口
    pub smtp_port: u16,
    /// SMTP 用户名
    pub smtp_username: String,
    /// SMTP 密码
    pub smtp_password: String,
    /// 发件人名称
    pub from_name: String,
    /// 发件人邮箱
    pub from_email: String,
    /// 是否启用 TLS
    pub enable_tls: bool,
}

impl Default for EmailConfig {
    fn default() -> Self {
        Self {
            smtp_host: String::new(),
            smtp_port: 587,
            smtp_username: String::new(),
            smtp_password: String::new(),
            from_name: String::new(),
            from_email: String::new(),
            enable_tls: true,
        }
    }
}
