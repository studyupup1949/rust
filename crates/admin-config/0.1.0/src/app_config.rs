//! 配置管理模块
//!
//! 提供统一的配置加载和管理功能，支持：
//! - 从 config.toml 文件加载配置
//! - 从环境变量覆盖配置
//! - 从命令行参数覆盖配置
//! - 配置验证和默认值
//!
//! # 配置优先级
//!
//! 1. 命令行参数（最高优先级）
//! 2. 环境变量
//! 3. config.toml 文件
//! 4. 默认值（最低优先级）
//!
//! # 使用示例
//!
//! ```rust,ignore
//! use admin_server_config::AppConfig;
//!
//! let config = AppConfig::load()?;
//! println!("Server port: {}", config.server.port);
//! ```

use crate::LlmConfig;
use crate::{
    AuthConfig, CosConfig, DatabaseConfig, DevelopmentConfig, EmailConfig, RateLimitConfig, RedisConfig,
    SecurityConfig, ServerConfig, SessionConfig, SmsConfig, UploadConfig, VerificationCodeConfig,
};
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    /// 服务器配置
    #[serde(default)]
    pub server: ServerConfig,
    /// 数据库配置
    #[serde(default)]
    pub database: DatabaseConfig,
    /// Redis 配置
    #[serde(default)]
    pub redis: RedisConfig,
    /// 认证配置
    #[serde(default)]
    pub auth: AuthConfig,
    /// 邮件配置
    #[serde(default)]
    pub email: EmailConfig,
    /// 短信配置
    #[serde(default)]
    pub sms: SmsConfig,
    /// 验证码配置
    #[serde(default)]
    pub verification_code: VerificationCodeConfig,
    /// 对象存储配置
    #[serde(default)]
    pub cos: CosConfig,
    /// 安全配置
    #[serde(default)]
    pub security: SecurityConfig,
    /// 会话配置
    #[serde(default)]
    pub session: SessionConfig,
    /// 上传配置
    #[serde(default)]
    pub upload: UploadConfig,
    /// 速率限制配置
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    /// 开发环境配置
    #[serde(default)]
    pub development: DevelopmentConfig,
    /// LLM 配置
    #[serde(default)]
    pub llm: Vec<LlmConfig>,
}

impl AppConfig {
    /// 加载配置
    pub fn load() -> Result<Self> {
        Self::load_from_default_path()
    }

    /// 从默认路径加载配置
    pub fn load_from_default_path() -> Result<Self> {
        let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());

        let path = if Path::new(&config_path).exists() {
            PathBuf::from(config_path)
        } else {
            match Self::find_config_file() {
                Ok(p) => p,
                Err(_) => {
                    log::warn!("Config file not found. Using default configuration with environment variables.");
                    return Ok(Self::default_with_env());
                }
            }
        };

        Self::load_from_path(&path)
    }

    /// 从指定路径加载配置
    fn load_from_path(path: &Path) -> Result<Self> {
        let config_content = std::fs::read_to_string(path).with_context(|| format!("无法读取配置文件: {:?}", path))?;
        let mut config: Self = toml::from_str(&config_content).with_context(|| "解析配置文件失败")?;
        config.apply_env_overrides();
        Ok(config)
    }

    /// 创建默认配置并应用环境变量
    fn default_with_env() -> Self {
        let mut config = Self::default();
        config.apply_env_overrides();
        config
    }

    /// 从文件加载配置
    pub fn from_file(path: &str) -> Result<Self> {
        Self::load_from_path(Path::new(path))
    }

    /// 查找配置文件
    ///
    /// 按以下顺序查找配置文件：
    /// 1. CONFIG_PATH 环境变量指定的路径
    /// 2. 当前工作目录下的 `config.toml`
    /// 3. 当前工作目录下的 `apps/admin-server/config.toml`
    /// 4. 可执行文件所在目录的 `config.toml`
    fn find_config_file() -> Result<PathBuf> {
        // 1. 检查环境变量
        if let Ok(config_path) = std::env::var("CONFIG_PATH") {
            let path = Path::new(&config_path);
            if path.exists() {
                return Ok(path.to_path_buf());
            }
        }

        // 2. 尝试查找配置文件的多个可能位置
        let possible_paths = vec![
            PathBuf::from("config.toml"),
            PathBuf::from("apps/admin-server/config.toml"),
        ];

        for path in &possible_paths {
            if path.exists() {
                return Ok(path.clone());
            }
        }

        // 3. 检查可执行文件所在目录
        if let Ok(exe_path) = std::env::current_exe()
            && let Some(exe_dir) = exe_path.parent()
        {
            let config_path = exe_dir.join("config.toml");
            if config_path.exists() {
                return Ok(config_path);
            }
        }

        // 如果都找不到，返回错误
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Err(anyhow::anyhow!(
            "无法找到配置文件 config.toml\n\
            当前工作目录: {:?}\n\
            已尝试以下路径:\n{}\n\
            请确保:\n\
            1. 设置 CONFIG_PATH 环境变量指向配置文件\n\
            2. 在项目根目录运行程序\n\
            3. 或将 config.toml 放在可执行文件同目录",
            cwd,
            possible_paths
                .iter()
                .map(|p| format!("  - {:?}", p))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }

    /// 应用环境变量覆盖
    fn apply_env_overrides(&mut self) {
        // Server
        if let Ok(port) = std::env::var("PORT").or_else(|_| std::env::var("SERVER__PORT"))
            && let Ok(p) = port.parse()
        {
            self.server.port = p;
        }
        if let Ok(log_level) = std::env::var("RUST_LOG").or_else(|_| std::env::var("SERVER__LOG_LEVEL")) {
            self.server.log_level = log_level;
        }

        // Redis
        if let Ok(host) = std::env::var("REDIS_IP").or_else(|_| std::env::var("REDIS__HOST")) {
            self.redis.host = host;
        }
        if let Ok(port) = std::env::var("REDIS_PORT").or_else(|_| std::env::var("REDIS__PORT"))
            && let Ok(p) = port.parse()
        {
            self.redis.port = p;
        }
        if let Ok(password) = std::env::var("REDIS_PASSWORD").or_else(|_| std::env::var("REDIS__PASSWORD")) {
            self.redis.password = Some(password);
        }

        // Database (MongoDB)
        if let Ok(host) = std::env::var("MONGODB_IP").or_else(|_| std::env::var("DATABASE__MONGODB__HOST")) {
            self.database.mongodb.host = host;
        }
        if let Ok(port) = std::env::var("MONGODB_PORT").or_else(|_| std::env::var("DATABASE__MONGODB__PORT"))
            && let Ok(p) = port.parse()
        {
            self.database.mongodb.port = p;
        }
        if let Ok(user) = std::env::var("MONGODB_USER").or_else(|_| std::env::var("DATABASE__MONGODB__USERNAME")) {
            self.database.mongodb.username = user;
        }
        if let Ok(password) =
            std::env::var("MONGODB_PASSWORD").or_else(|_| std::env::var("DATABASE__MONGODB__PASSWORD"))
        {
            self.database.mongodb.password = password;
        }
        if let Ok(database) =
            std::env::var("MONGODB_DATABASE").or_else(|_| std::env::var("DATABASE__MONGODB__DATABASE"))
        {
            self.database.mongodb.database = database;
        }

        // Database (MySQL)
        if let Ok(host) = std::env::var("MYSQL_HOST").or_else(|_| std::env::var("DATABASE__MYSQL__HOST")) {
            self.database.mysql.host = host;
        }
        if let Ok(port) = std::env::var("MYSQL_PORT").or_else(|_| std::env::var("DATABASE__MYSQL__PORT"))
            && let Ok(p) = port.parse()
        {
            self.database.mysql.port = p;
        }
        if let Ok(user) = std::env::var("MYSQL_USER").or_else(|_| std::env::var("DATABASE__MYSQL__USERNAME")) {
            self.database.mysql.username = user;
        }
        if let Ok(password) = std::env::var("MYSQL_PASSWORD").or_else(|_| std::env::var("DATABASE__MYSQL__PASSWORD")) {
            self.database.mysql.password = password;
        }
        if let Ok(database) = std::env::var("MYSQL_DATABASE").or_else(|_| std::env::var("DATABASE__MYSQL__DATABASE")) {
            self.database.mysql.database = database;
        }

        // Database (PostgreSQL)
        if let Ok(host) = std::env::var("POSTGRES_HOST").or_else(|_| std::env::var("DATABASE__POSTGRESQL__HOST")) {
            self.database.postgresql.host = host;
        }
        if let Ok(port) = std::env::var("POSTGRES_PORT").or_else(|_| std::env::var("DATABASE__POSTGRESQL__PORT"))
            && let Ok(p) = port.parse()
        {
            self.database.postgresql.port = p;
        }
        if let Ok(user) = std::env::var("POSTGRES_USER").or_else(|_| std::env::var("DATABASE__POSTGRESQL__USERNAME")) {
            self.database.postgresql.username = user;
        }
        if let Ok(password) =
            std::env::var("POSTGRES_PASSWORD").or_else(|_| std::env::var("DATABASE__POSTGRESQL__PASSWORD"))
        {
            self.database.postgresql.password = password;
        }
        if let Ok(database) =
            std::env::var("POSTGRES_DATABASE").or_else(|_| std::env::var("DATABASE__POSTGRESQL__DATABASE"))
        {
            self.database.postgresql.database = database;
        }

        // Auth
        if let Ok(secret) = std::env::var("TOKEN_SECRET").or_else(|_| std::env::var("AUTH__TOKEN_SECRET")) {
            self.auth.token_secret = secret;
        }

        // Email
        if let Ok(host) = std::env::var("SMTP_HOST").or_else(|_| std::env::var("EMAIL__SMTP_HOST")) {
            self.email.smtp_host = host;
        }
        if let Ok(port) = std::env::var("SMTP_PORT").or_else(|_| std::env::var("EMAIL__SMTP_PORT"))
            && let Ok(p) = port.parse()
        {
            self.email.smtp_port = p;
        }
        if let Ok(user) = std::env::var("SMTP_USERNAME").or_else(|_| std::env::var("EMAIL__SMTP_USERNAME")) {
            self.email.smtp_username = user;
        }
        if let Ok(password) = std::env::var("SMTP_PASSWORD").or_else(|_| std::env::var("EMAIL__SMTP_PASSWORD")) {
            self.email.smtp_password = password;
        }

        // SMS
        if let Ok(provider) = std::env::var("SMS_PROVIDER").or_else(|_| std::env::var("SMS__PROVIDER")) {
            self.sms.provider = provider;
        }
        if let Ok(app_id) = std::env::var("SMS_APP_ID").or_else(|_| std::env::var("SMS__APP_ID")) {
            self.sms.app_id = app_id;
        }
        if let Ok(app_key) = std::env::var("SMS_APP_KEY").or_else(|_| std::env::var("SMS__APP_KEY")) {
            self.sms.app_key = app_key;
        }

        // COS
        if let Ok(secret_id) = std::env::var("COS_SECRET_ID").or_else(|_| std::env::var("COS__SECRET_ID")) {
            self.cos.secret_id = secret_id;
        }
        if let Ok(secret_key) = std::env::var("COS_SECRET_KEY").or_else(|_| std::env::var("COS__SECRET_KEY")) {
            self.cos.secret_key = secret_key;
        }
        if let Ok(bucket) = std::env::var("COS_BUCKET").or_else(|_| std::env::var("COS__BUCKET")) {
            self.cos.bucket = bucket;
        }
        if let Ok(region) = std::env::var("COS_REGION").or_else(|_| std::env::var("COS__REGION")) {
            self.cos.region = region;
        }

        // LLM - 环境变量只覆盖第一个 LLM 配置（如果存在）
        if let Some(llm) = self.llm.first_mut() {
            if let Ok(provider) = std::env::var("LLM_PROVIDER").or_else(|_| std::env::var("LLM__PROVIDER")) {
                llm.provider = provider;
            }
            if let Ok(api_key) = std::env::var("LLM_API_KEY").or_else(|_| std::env::var("LLM__API_KEY")) {
                llm.api_key = api_key;
            }
            if let Ok(model) = std::env::var("LLM_MODEL").or_else(|_| std::env::var("LLM__MODEL")) {
                llm.model = model;
            }
            if let Ok(api_base) = std::env::var("LLM_API_BASE").or_else(|_| std::env::var("LLM__API_BASE")) {
                llm.api_base = Some(api_base);
            }
        }
    }
}
