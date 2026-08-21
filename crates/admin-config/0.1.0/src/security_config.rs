use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// AES 密钥
    pub aes_key: String,
    /// AES 向量
    pub aes_iv: String,
    /// 是否启用 CORS
    pub enable_cors: bool,
    /// 允许的来源 (逗号分隔)
    pub allowed_origins: String,
    /// 是否启用 CSRF 防护
    pub enable_csrf: bool,
}

impl Default for SecurityConfig {
    fn default() -> Self {
        Self {
            aes_key: "".to_string(),
            aes_iv: "".to_string(),
            enable_cors: true,
            allowed_origins: "".to_string(),
            enable_csrf: false,
        }
    }
}

impl SecurityConfig {
    pub fn allowed_origins_list(&self) -> Vec<String> {
        self.allowed_origins.split(',').map(|s| s.trim().to_string()).collect()
    }
}
