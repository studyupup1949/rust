//! 服务器配置模块
//!
//! 提供 HTTP 服务器的基础配置

use serde::{Deserialize, Serialize};

/// 服务器配置
///
/// 用于配置 HTTP 服务器的基础信息
///
/// # 字段说明
///
/// - `name`: 服务名称，默认 "actix-admin-server"
/// - `version`: 服务版本，默认 "0.1.0"
/// - `port`: 监听端口，默认 3400
/// - `host`: 监听地址，默认 "0.0.0.0"
/// - `log_level`: 日志级别，默认 "info"，支持 trace/debug/info/warn/error
///
/// # 示例
///
/// ```rust
/// use admin_config::ServerConfig;
///
/// let config = ServerConfig::default();
/// assert_eq!(config.port, 3400);
/// assert_eq!(config.host, "0.0.0.0");
/// ```
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
