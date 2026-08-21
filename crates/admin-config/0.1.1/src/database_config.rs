//! 数据库配置模块
//!
//! 提供多种数据库的连接配置，包括：
//! - MongoDB（文档数据库）
//! - MySQL（关系型数据库）
//! - PostgreSQL（关系型数据库）
//! - SQLite（嵌入式数据库）
//! - Redis（缓存数据库）
//! - Neo4j（图数据库）
//! - Qdrant（向量数据库）
//! - SeekDB（多模型数据库）

use serde::{Deserialize, Serialize};

/// 数据库连接 URL 转换 trait
///
/// 用于将配置对象转换为标准连接字符串
pub trait ToConnectionUrl {
    /// 生成数据库连接 URL
    fn to_connection_url(&self) -> String;
}

/// 数据库配置集合
///
/// 包含所有支持的数据库配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DatabaseConfig {
    /// MongoDB 配置
    #[serde(default)]
    pub mongodb: MongoDbConfig,
    /// MySQL 配置
    #[serde(default)]
    pub mysql: MySqlConfig,
    /// PostgreSQL 配置
    #[serde(default)]
    pub postgresql: PostgreSqlConfig,
    /// SQLite 配置
    #[serde(default)]
    pub sqlite: SqliteConfig,
    /// Qdrant 配置
    #[serde(default)]
    pub qdrant: QdrantConfig,
    /// SeekDB 配置
    #[serde(default)]
    pub seekdb: SeekDbConfig,
    /// Neo4j 图数据库配置
    #[serde(default)]
    pub neo4j: Neo4jConfig,
}

/// MongoDB 数据库配置
///
/// # 示例
///
/// ```rust
/// use admin_config::{MongoDbConfig, ToConnectionUrl};
///
/// let config = MongoDbConfig {
///     host: "localhost".to_string(),
///     port: 27017,
///     database: "mydb".to_string(),
///     username: "admin".to_string(),
///     password: "password".to_string(),
///     max_pool_size: 10,
///     connect_timeout: 10,
/// };
///
/// let url = config.to_connection_url();
/// assert!(url.contains("mongodb://"));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MongoDbConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 数据库名
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 最大连接池大小
    pub max_pool_size: u32,
    /// 连接超时时间(秒)
    pub connect_timeout: u64,
}

impl Default for MongoDbConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 27017,
            database: String::new(),
            username: String::new(),
            password: String::new(),
            max_pool_size: 10,
            connect_timeout: 10,
        }
    }
}

impl ToConnectionUrl for MongoDbConfig {
    fn to_connection_url(&self) -> String {
        if self.username.is_empty() {
            format!("mongodb://{}:{}/{}", self.host, self.port, self.database)
        } else {
            format!(
                "mongodb://{}:{}@{}:{}/{}",
                self.username, self.password, self.host, self.port, self.database
            )
        }
    }
}

/// MySQL 数据库配置
///
/// # 示例
///
/// ```rust
/// use admin_config::{MySqlConfig, ToConnectionUrl};
///
/// let config = MySqlConfig::default();
/// assert_eq!(config.port, 3306);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MySqlConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 数据库名
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 最大连接池大小
    pub max_pool_size: u32,
    /// 连接超时时间(秒)
    pub connect_timeout: u64,
}

impl Default for MySqlConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 3306,
            database: String::new(),
            username: String::new(),
            password: String::new(),
            max_pool_size: 10,
            connect_timeout: 10,
        }
    }
}

impl ToConnectionUrl for MySqlConfig {
    fn to_connection_url(&self) -> String {
        let mut url = "mysql://".to_string();

        if !self.username.is_empty() && !self.password.is_empty() {
            url.push_str(&format!("{}:{}@", self.username, self.password));
        } else if !self.username.is_empty() {
            url.push_str(&format!("{}@", self.username));
        }

        url.push_str(&format!("{}:{}", self.host, self.port));

        if !self.database.is_empty() {
            url.push_str(&format!("/{}", self.database));
        }

        url
    }
}

/// PostgreSQL 数据库配置
///
/// # 示例
///
/// ```rust
/// use admin_config::{PostgreSqlConfig, ToConnectionUrl};
///
/// let config = PostgreSqlConfig::default();
/// assert_eq!(config.port, 5432);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgreSqlConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 数据库名
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 最大连接池大小
    pub max_pool_size: u32,
    /// 连接超时时间(秒)
    pub connect_timeout: u64,
}

impl Default for PostgreSqlConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: String::new(),
            username: String::new(),
            password: String::new(),
            max_pool_size: 10,
            connect_timeout: 10,
        }
    }
}

impl ToConnectionUrl for PostgreSqlConfig {
    fn to_connection_url(&self) -> String {
        let mut url = "postgresql://".to_string();

        if !self.username.is_empty() && !self.password.is_empty() {
            url.push_str(&format!("{}:{}@", self.username, self.password));
        } else if !self.username.is_empty() {
            url.push_str(&format!("{}@", self.username));
        }

        url.push_str(&format!("{}:{}", self.host, self.port));

        if !self.database.is_empty() {
            url.push_str(&format!("/{}", self.database));
        }

        url
    }
}

/// SQLite 数据库配置
///
/// # 示例
///
/// ```rust
/// use admin_config::{SqliteConfig, ToConnectionUrl};
///
/// let config = SqliteConfig::default();
/// assert_eq!(config.path, "./data.db");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SqliteConfig {
    /// 数据库文件路径
    pub path: String,
    /// 最大连接池大小
    pub max_pool_size: u32,
}

impl Default for SqliteConfig {
    fn default() -> Self {
        Self {
            path: "./data.db".to_string(),
            max_pool_size: 10,
        }
    }
}

impl ToConnectionUrl for SqliteConfig {
    fn to_connection_url(&self) -> String {
        format!("sqlite://{}", self.path)
    }
}

/// Qdrant 向量数据库配置
///
/// # 示例
///
/// ```rust
/// use admin_config::{QdrantConfig, ToConnectionUrl};
///
/// let config = QdrantConfig::default();
/// assert_eq!(config.port, 6333);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QdrantConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// API Key
    pub api_key: Option<String>,
    /// 是否使用 HTTPS
    pub use_https: bool,
}

impl Default for QdrantConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 6333,
            api_key: None,
            use_https: false,
        }
    }
}

impl ToConnectionUrl for QdrantConfig {
    fn to_connection_url(&self) -> String {
        let protocol = if self.use_https { "https" } else { "http" };
        format!("{}://{}:{}", protocol, self.host, self.port)
    }
}

/// SeekDB 多模型数据库配置
///
/// # 示例
///
/// ```rust
/// use admin_config::{SeekDbConfig, ToConnectionUrl};
///
/// let config = SeekDbConfig::default();
/// assert_eq!(config.port, 2881);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeekDbConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 数据库名
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
}

impl Default for SeekDbConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 2881,
            database: String::new(),
            username: String::new(),
            password: String::new(),
        }
    }
}

impl ToConnectionUrl for SeekDbConfig {
    fn to_connection_url(&self) -> String {
        if self.username.is_empty() {
            format!("mysql://{}:{}/{}", self.host, self.port, self.database)
        } else {
            format!(
                "mysql://{}:{}@{}:{}/{}",
                self.username, self.password, self.host, self.port, self.database
            )
        }
    }
}

/// Neo4j 图数据库配置
///
/// # 示例
///
/// ```rust
/// use admin_config::{Neo4jConfig, ToConnectionUrl};
///
/// let config = Neo4jConfig::default();
/// assert_eq!(config.port, 7687);
/// assert_eq!(config.database, "neo4j");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Neo4jConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 数据库名
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 是否使用加密连接
    pub use_encryption: bool,
    /// 最大连接池大小
    pub max_pool_size: u32,
    /// 连接超时时间(秒)
    pub connect_timeout: u64,
}

impl Default for Neo4jConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 7687,
            database: "neo4j".to_string(),
            username: String::new(),
            password: String::new(),
            use_encryption: false,
            max_pool_size: 10,
            connect_timeout: 10,
        }
    }
}

impl ToConnectionUrl for Neo4jConfig {
    fn to_connection_url(&self) -> String {
        let protocol = if self.use_encryption { "neo4j+s" } else { "neo4j" };

        if self.username.is_empty() {
            format!("{}://{}:{}", protocol, self.host, self.port)
        } else {
            format!(
                "{}://{}:{}@{}:{}",
                protocol, self.username, self.password, self.host, self.port
            )
        }
    }
}
