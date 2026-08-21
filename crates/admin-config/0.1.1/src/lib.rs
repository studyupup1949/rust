//! 统一配置管理库
//!
//! 提供应用程序所需的各类配置管理功能，支持：
//! - 多数据库配置（MongoDB、MySQL、PostgreSQL、SQLite、Redis、Neo4j、Qdrant、SeekDB）
//! - 认证与安全配置（JWT、Session、CORS、加密）
//! - 第三方服务配置（邮件、短信、对象存储）
//! - 环境变量覆盖
//! - 配置文件自动查找
//!
//! # 功能特性
//!
//! - 支持从 config.toml 加载配置
//! - 支持环境变量覆盖配置项
//! - 自动查找配置文件（多路径策略）
//! - 配置验证与默认值
//! - 敏感信息自动生成（密钥、盐值等）
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use admin_config::AppConfig;
//!
//! let config = AppConfig::load()?;
//!
//! println!("Server running on {}:{}", config.server.host, config.server.port);
//! println!("MongoDB connection: {}", config.database.mongodb.to_connection_url());
//! println!("Redis connection: {}", config.redis.connection_string());
//! ```
//!
//! # 配置文件示例
//!
//! ```toml
//! [server]
//! name = "actix-admin-server"
//! version = "0.1.0"
//! port = 3400
//! host = "0.0.0.0"
//! log_level = "info"
//!
//! [database.mongodb]
//! host = "localhost"
//! port = 27017
//! database = "admin"
//! username = "admin"
//! password = "password"
//!
//! [redis]
//! host = "localhost"
//! port = 6379
//! database = 0
//! ```

mod app_config;
mod auth_config;
mod cos_config;
mod database_config;
mod email_config;
mod redis_config;
mod security_config;
mod server_config;
mod session_config;
mod sms_config;
mod verification_config;

pub use app_config::AppConfig;
pub use auth_config::AuthConfig;
pub use cos_config::CosConfig;
pub use database_config::{
    MongoDbConfig, MySqlConfig, PostgreSqlConfig, QdrantConfig, SeekDbConfig, SqliteConfig, ToConnectionUrl,
};
pub use email_config::EmailConfig;
pub use redis_config::RedisConfig;
pub use security_config::SecurityConfig;
pub use server_config::ServerConfig;
pub use session_config::SessionConfig;
pub use sms_config::{AliyunSmsConfig, SmsConfig, TencentSmsConfig};
pub use verification_config::VerificationCodeConfig;
