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

use crate::database_config::DatabaseConfig;
use crate::{
    AuthConfig, CosConfig, EmailConfig, RedisConfig, SecurityConfig, ServerConfig, SessionConfig, SmsConfig,
    VerificationCodeConfig,
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
}

impl AppConfig {
    /// 加载配置
    pub fn load() -> Result<Self> {
        Self::load_from_default_path()
    }

    /// 从默认路径加载配置
    pub fn load_from_default_path() -> Result<Self> {
        let path = match Self::find_config_file() {
            Ok(p) => p,
            Err(_) => {
                log::warn!("Config file not found. Using default configuration with environment variables.");
                return Ok(Self::default_with_env());
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
    /// 2. 当前 crate 目录下的 `config.toml`
    /// 3. 当前工作目录下的 `config.toml`
    /// 4. workspace 根目录下的 `config.toml`
    /// 5. ~/.config/admin-config/config.toml
    fn find_config_file() -> Result<PathBuf> {
        // 1. 检查环境变量
        if let Ok(config_path) = std::env::var("CONFIG_PATH") {
            let path = Path::new(&config_path);
            if path.exists() {
                return Ok(path.to_path_buf());
            }
        }

        let mut possible_paths = Vec::new();

        // 2. 当前 crate 目录（通过 CARGO_MANIFEST_DIR 获取）
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR") {
            let crate_config = PathBuf::from(&manifest_dir).join("config.toml");
            possible_paths.push(crate_config.clone());
            if crate_config.exists() {
                return Ok(crate_config);
            }
        }

        // 3. 当前工作目录
        let cwd_config = PathBuf::from("config.toml");
        possible_paths.push(cwd_config.clone());
        if cwd_config.exists() {
            return Ok(cwd_config);
        }

        // 4. workspace 根目录（向上查找 Cargo.toml 中包含 [workspace] 的目录）
        if let Ok(manifest_dir) = std::env::var("CARGO_MANIFEST_DIR")
            && let Some(workspace_root) = Self::find_workspace_root(&manifest_dir)
        {
            let workspace_config = workspace_root.join("config.toml");
            possible_paths.push(workspace_config.clone());
            if workspace_config.exists() {
                return Ok(workspace_config);
            }
        }

        // 5. ~/.config/admin-config/config.toml
        if let Some(home_dir) = std::env::var_os("HOME") {
            let user_config = PathBuf::from(home_dir)
                .join(".config")
                .join("admin-config")
                .join("config.toml");
            possible_paths.push(user_config.clone());
            if user_config.exists() {
                return Ok(user_config);
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
            2. 在当前 crate 目录创建 config.toml\n\
            3. 在当前工作目录创建 config.toml\n\
            4. 在 workspace 根目录创建 config.toml\n\
            5. 在 ~/.config/admin-config/ 目录创建 config.toml",
            cwd,
            possible_paths
                .iter()
                .map(|p| format!("  - {:?}", p))
                .collect::<Vec<_>>()
                .join("\n")
        ))
    }

    /// 查找 workspace 根目录
    ///
    /// 从当前 crate 目录向上查找包含 [workspace] 的 Cargo.toml
    fn find_workspace_root(start_dir: &str) -> Option<PathBuf> {
        let mut current = PathBuf::from(start_dir);

        while let Some(parent) = current.parent() {
            current = parent.to_path_buf();
            let cargo_toml = current.join("Cargo.toml");

            if cargo_toml.exists()
                && let Ok(content) = std::fs::read_to_string(&cargo_toml)
                && content.contains("[workspace]")
            {
                return Some(current);
            }

            if current.parent().is_none() {
                break;
            }
        }

        None
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

        // Database (Neo4j)
        if let Ok(host) = std::env::var("NEO4J_HOST").or_else(|_| std::env::var("DATABASE__NEO4J__HOST")) {
            self.database.neo4j.host = host;
        }
        if let Ok(port) = std::env::var("NEO4J_PORT").or_else(|_| std::env::var("DATABASE__NEO4J__PORT"))
            && let Ok(p) = port.parse()
        {
            self.database.neo4j.port = p;
        }
        if let Ok(user) = std::env::var("NEO4J_USER").or_else(|_| std::env::var("DATABASE__NEO4J__USERNAME")) {
            self.database.neo4j.username = user;
        }
        if let Ok(password) = std::env::var("NEO4J_PASSWORD").or_else(|_| std::env::var("DATABASE__NEO4J__PASSWORD")) {
            self.database.neo4j.password = password;
        }
        if let Ok(database) = std::env::var("NEO4J_DATABASE").or_else(|_| std::env::var("DATABASE__NEO4J__DATABASE")) {
            self.database.neo4j.database = database;
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
        if let Ok(provider) = std::env::var("COS_PROVIDER").or_else(|_| std::env::var("COS__PROVIDER")) {
            self.cos.provider = provider;
        }

        // COS - Tencent
        if let Ok(secret_id) = std::env::var("COS_SECRET_ID").or_else(|_| std::env::var("COS__TENCENT__SECRET_ID")) {
            self.cos.tencent.secret_id = secret_id;
        }
        if let Ok(secret_key) = std::env::var("COS_SECRET_KEY").or_else(|_| std::env::var("COS__TENCENT__SECRET_KEY")) {
            self.cos.tencent.secret_key = secret_key;
        }
        if let Ok(bucket) = std::env::var("COS_BUCKET").or_else(|_| std::env::var("COS__TENCENT__BUCKET")) {
            self.cos.tencent.bucket = bucket;
        }
        if let Ok(region) = std::env::var("COS_REGION").or_else(|_| std::env::var("COS__TENCENT__REGION")) {
            self.cos.tencent.region = region;
        }

        // COS - Aliyun
        if let Ok(access_key_id) =
            std::env::var("OSS_ACCESS_KEY_ID").or_else(|_| std::env::var("COS__ALIYUN__ACCESS_KEY_ID"))
        {
            self.cos.aliyun.access_key_id = access_key_id;
        }
        if let Ok(access_key_secret) =
            std::env::var("OSS_ACCESS_KEY_SECRET").or_else(|_| std::env::var("COS__ALIYUN__ACCESS_KEY_SECRET"))
        {
            self.cos.aliyun.access_key_secret = access_key_secret;
        }
        if let Ok(bucket) = std::env::var("OSS_BUCKET").or_else(|_| std::env::var("COS__ALIYUN__BUCKET")) {
            self.cos.aliyun.bucket = bucket;
        }
        if let Ok(endpoint) = std::env::var("OSS_ENDPOINT").or_else(|_| std::env::var("COS__ALIYUN__ENDPOINT")) {
            self.cos.aliyun.endpoint = endpoint;
        }

        // COS - AWS S3
        if let Ok(access_key_id) =
            std::env::var("AWS_ACCESS_KEY_ID").or_else(|_| std::env::var("COS__AWS__ACCESS_KEY_ID"))
        {
            self.cos.aws.access_key_id = access_key_id;
        }
        if let Ok(secret_access_key) =
            std::env::var("AWS_SECRET_ACCESS_KEY").or_else(|_| std::env::var("COS__AWS__SECRET_ACCESS_KEY"))
        {
            self.cos.aws.secret_access_key = secret_access_key;
        }
        if let Ok(bucket) = std::env::var("AWS_S3_BUCKET").or_else(|_| std::env::var("COS__AWS__BUCKET")) {
            self.cos.aws.bucket = bucket;
        }
        if let Ok(region) = std::env::var("AWS_REGION").or_else(|_| std::env::var("COS__AWS__REGION")) {
            self.cos.aws.region = region;
        }

        // COS - MinIO
        if let Ok(access_key) = std::env::var("MINIO_ACCESS_KEY").or_else(|_| std::env::var("COS__MINIO__ACCESS_KEY")) {
            self.cos.minio.access_key = access_key;
        }
        if let Ok(secret_key) = std::env::var("MINIO_SECRET_KEY").or_else(|_| std::env::var("COS__MINIO__SECRET_KEY")) {
            self.cos.minio.secret_key = secret_key;
        }
        if let Ok(bucket) = std::env::var("MINIO_BUCKET").or_else(|_| std::env::var("COS__MINIO__BUCKET")) {
            self.cos.minio.bucket = bucket;
        }
        if let Ok(endpoint) = std::env::var("MINIO_ENDPOINT").or_else(|_| std::env::var("COS__MINIO__ENDPOINT")) {
            self.cos.minio.endpoint = endpoint;
        }

        // COS - Huawei OBS
        if let Ok(access_key_id) =
            std::env::var("HUAWEI_ACCESS_KEY_ID").or_else(|_| std::env::var("COS__HUAWEI__ACCESS_KEY_ID"))
        {
            self.cos.huawei.access_key_id = access_key_id;
        }
        if let Ok(secret_access_key) =
            std::env::var("HUAWEI_SECRET_ACCESS_KEY").or_else(|_| std::env::var("COS__HUAWEI__SECRET_ACCESS_KEY"))
        {
            self.cos.huawei.secret_access_key = secret_access_key;
        }
        if let Ok(bucket) = std::env::var("HUAWEI_BUCKET").or_else(|_| std::env::var("COS__HUAWEI__BUCKET")) {
            self.cos.huawei.bucket = bucket;
        }
        if let Ok(endpoint) = std::env::var("HUAWEI_ENDPOINT").or_else(|_| std::env::var("COS__HUAWEI__ENDPOINT")) {
            self.cos.huawei.endpoint = endpoint;
        }

        // COS - RustFS
        if let Ok(root_path) = std::env::var("RUSTFS_ROOT_PATH").or_else(|_| std::env::var("COS__RUSTFS__ROOT_PATH")) {
            self.cos.rustfs.root_path = root_path;
        }
        if let Ok(public_url_prefix) =
            std::env::var("RUSTFS_PUBLIC_URL_PREFIX").or_else(|_| std::env::var("COS__RUSTFS__PUBLIC_URL_PREFIX"))
        {
            self.cos.rustfs.public_url_prefix = public_url_prefix;
        }
    }

    /// 生成配置文件到指定路径
    pub fn generate_to_file(path: &str) -> Result<Self> {
        let config_path = Path::new(path);

        if config_path.exists() {
            Self::load_from_path(config_path)
        } else {
            let default_config = Self::default();

            let toml_content = toml::to_string_pretty(&default_config).context("序列化配置为 TOML 失败")?;

            if let Some(parent) = config_path.parent() {
                std::fs::create_dir_all(parent).with_context(|| format!("创建配置文件目录失败: {:?}", parent))?;
            }

            std::fs::write(config_path, toml_content)
                .with_context(|| format!("写入配置文件失败: {:?}", config_path))?;

            Self::print_security_warning();

            Ok(default_config)
        }
    }

    fn print_security_warning() {
        log::warn!("⚠️  安全提示：");
        log::warn!("   1. 系统已自动生成安全密钥，请妥善保管配置文件");
        log::warn!("   2. 修改配置文件中的其他敏感信息（数据库密码、API密钥等）");
        log::warn!("   3. 不要将包含真实密钥的配置文件提交到 Git 仓库");
        log::warn!("   4. 生产环境建议使用环境变量或密钥管理服务");
    }
}

impl AppConfig {
    /// 生成默认配置
    ///
    /// 创建默认的 AppConfig 实例，存储到 `config.toml` 文件中
    ///
    /// # 行为说明
    ///
    /// 1. 如果 config.toml 文件不存在，则创建文件并写入默认配置
    /// 2. 如果 config.toml 文件存在，则读取现有配置并返回
    ///
    /// # 返回
    ///
    /// 返回加载或生成的配置对象
    pub fn generate(&self) -> anyhow::Result<AppConfig> {
        Self::generate_to_file("config.toml")
    }
}
