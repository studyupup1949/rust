use serde::{Deserialize, Serialize};

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
            smtp_host: "smtp.gmail.com".to_string(),
            smtp_port: 587,
            smtp_username: "".to_string(),
            smtp_password: "".to_string(),
            from_name: "系统通知".to_string(),
            from_email: "".to_string(),
            enable_tls: true,
        }
    }
}
