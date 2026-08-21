use serde::{Deserialize, Serialize};

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
            secret_key: "".to_string(),
            max_age: 86400,
            http_only: true,
            secure: false,
            cookie_path: "/".to_string(),
            cookie_domain: None,
        }
    }
}
