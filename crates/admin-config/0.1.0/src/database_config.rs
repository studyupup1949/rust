use serde::{Deserialize, Serialize};

pub trait ToConnectionUrl {
    fn to_connection_url(&self) -> String;
}

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
    /// SurrealDB 配置
    #[serde(default)]
    pub surrealdb: SurrealDbConfig,
}

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
            database: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
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
            database: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            max_pool_size: 10,
            connect_timeout: 10,
        }
    }
}

impl ToConnectionUrl for MySqlConfig {
    fn to_connection_url(&self) -> String {
        format!(
            "mysql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

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
            database: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            max_pool_size: 10,
            connect_timeout: 10,
        }
    }
}

impl ToConnectionUrl for PostgreSqlConfig {
    fn to_connection_url(&self) -> String {
        format!(
            "postgresql://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.database
        )
    }
}

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
            database: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
        }
    }
}

impl ToConnectionUrl for SeekDbConfig {
    fn to_connection_url(&self) -> String {
        if self.username.is_empty() {
            format!("seekdb://{}:{}/{}", self.host, self.port, self.database)
        } else {
            format!(
                "seekdb://{}:{}@{}:{}/{}",
                self.username, self.password, self.host, self.port, self.database
            )
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurrealDbConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 命名空间
    pub namespace: String,
    /// 数据库名
    pub database: String,
    /// 用户名
    pub username: String,
    /// 密码
    pub password: String,
    /// 是否使用 HTTPS
    pub use_https: bool,
}

impl Default for SurrealDbConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 9090,
            namespace: "".to_string(),
            database: "".to_string(),
            username: "".to_string(),
            password: "".to_string(),
            use_https: false,
        }
    }
}

impl ToConnectionUrl for SurrealDbConfig {
    fn to_connection_url(&self) -> String {
        let protocol = if self.use_https { "wss" } else { "ws" };
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
