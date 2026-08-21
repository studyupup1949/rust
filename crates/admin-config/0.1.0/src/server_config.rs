use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// 服务名称
    pub name: String,
    /// 服务版本
    pub version: String,
    /// 监听端口
    pub port: u16,
    /// 监听地址
    pub host: String,
    /// 日志级别
    pub log_level: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            name: "actix-admin-server".to_string(),
            version: "0.1.0".to_string(),
            port: 3400,
            host: "0.0.0.0".to_string(),
            log_level: "info".to_string(),
        }
    }
}
