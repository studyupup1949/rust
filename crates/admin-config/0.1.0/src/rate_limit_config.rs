use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 是否启用速率限制
    pub enabled: bool,
    /// 每分钟请求数限制
    pub requests_per_minute: u32,
    /// 突发请求数限制
    pub burst_size: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            requests_per_minute: 60,
            burst_size: 10,
        }
    }
}
