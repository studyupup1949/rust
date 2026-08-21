use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    /// 主机地址
    pub host: String,
    /// 端口
    pub port: u16,
    /// 用户名
    pub username: Option<String>,
    /// 密码
    pub password: Option<String>,
    /// 数据库索引
    pub database: u8,
    /// 最大连接池大小
    pub max_pool_size: u32,
    /// 连接超时时间(秒)
    pub connect_timeout: u64,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 6379,
            username: Some("".to_string()),
            password: Some("".to_string()),
            database: 0,
            max_pool_size: 20,
            connect_timeout: 5,
        }
    }
}

impl RedisConfig {
    pub fn connection_string(&self) -> String {
        match (&self.username, &self.password) {
            (Some(username), Some(password)) => {
                format!(
                    "redis://{}:{}@{}:{}/{}",
                    username, password, self.host, self.port, self.database
                )
            }
            (None, Some(password)) => {
                format!("redis://:{}@{}:{}/{}", password, self.host, self.port, self.database)
            }
            _ => {
                format!("redis://{}:{}/{}", self.host, self.port, self.database)
            }
        }
    }
}
