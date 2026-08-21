# Admin Config

[![Crates.io](https://img.shields.io/crates/v/admin-config.svg)](https://crates.io/crates/admin-config)
[![Documentation](https://docs.rs/admin-config/badge.svg)](https://docs.rs/admin-config)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

配置管理库，为 Rust Web 应用提供统一的配置加载和管理功能。

## 功能特性

- ✅ TOML 配置文件支持
- ✅ 环境变量覆盖
- ✅ 类型安全的配置结构
- ✅ 默认值支持
- ✅ 多数据库支持（MongoDB、MySQL、PostgreSQL、SQLite、Redis、Neo4j、Qdrant、SeekDB）
- ✅ 自动生成安全密钥
- ✅ 第三方服务配置（邮件、短信、对象存储）
- ✅ 配置文件自动查找

## 安装

添加依赖到 `Cargo.toml`：

```toml
[dependencies]
admin-config = "0.1"
```

## 快速开始

### 1. 生成配置文件

首先创建配置文件。你可以从 crate 包含的示例配置开始：

```bash
# 复制示例配置文件（位于 crate 根目录）
cp config.example.toml config.toml
```

或者使用代码生成默认配置：

```rust
use admin_config::AppConfig;

fn main() -> anyhow::Result<()> {
    let config = AppConfig::default();
    config.generate()?;
    Ok(())
}
```

### 2. 使用配置

```rust
use admin_config::{AppConfig, ToConnectionUrl};

fn main() -> anyhow::Result<()> {
    let config = AppConfig::load()?;

    println!("Server: {}:{}", config.server.host, config.server.port);

    println!("MongoDB: {}", config.database.mongodb.to_connection_url());
    println!("Redis: {}", config.redis.connection_string());

    Ok(())
}
```

## 支持的数据库

- **MongoDB** - 文档数据库
- **MySQL** - 关系型数据库
- **PostgreSQL** - 关系型数据库
- **SQLite** - 嵌入式数据库
- **Redis** - 缓存数据库
- **Neo4j** - 图数据库
- **Qdrant** - 向量数据库
- **SeekDB** - 多模型数据库

## 配置优先级

支持三种配置方式，优先级从高到低：

1. **环境变量**（最高优先级）
   - 简写格式：`PORT`, `REDIS_IP`, `MONGODB_USER`
   - 层级格式：`SERVER__PORT`, `REDIS__HOST`, `DATABASE__MONGODB__USERNAME`
2. **config.toml 配置文件**
3. **默认值**（最低优先级）

## 配置文件示例

### 服务器配置

```toml
[server]
name = "actix-admin-server"
port = 3400
host = "0.0.0.0"
log_level = "info"
```

### 数据库配置

```toml
[database.mongodb]
host = "localhost"
port = 27017
database = "admin_db"
username = "admin"
password = "password"
max_pool_size = 10
connect_timeout = 10

[database.mysql]
host = "localhost"
port = 3306
database = "app_db"
username = "root"
password = "password"
max_pool_size = 10
connect_timeout = 10

[database.postgresql]
host = "localhost"
port = 5432
database = "app_db"
username = "postgres"
password = "password"
max_pool_size = 10
connect_timeout = 10

[database.sqlite]
path = "./data.db"
max_pool_size = 10

[database.qdrant]
host = "localhost"
port = 6333
use_https = false

[database.neo4j]
host = "localhost"
port = 7687
database = "neo4j"
username = ""
password = ""
use_encryption = false
```

### Redis 配置

```toml
[redis]
host = "localhost"
port = 6379
password = ""
database = 0
max_pool_size = 20
connect_timeout = 5
```

### 认证配置

```toml
[auth]
token_secret = "your-secret-key"
token_expiry_hours = 24
refresh_token_expiry_days = 7
```

### 安全配置

```toml
[security]
aes_key = "64-char-hex-string"
aes_iv = "32-char-hex-string"
api_key_encrypt_key = "64-char-hex-string"
password_salt = "32-char-hex-string"
enable_cors = true
allowed_origins = ""
enable_csrf = false
```

### 邮件服务配置

```toml
[email]
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = ""
smtp_password = ""
from_name = "System"
from_email = ""
enable_tls = true
```

### 短信服务配置

```toml
[sms]
provider = "tencent"
app_id = ""
app_key = ""
sign_name = ""
template_id = ""
```

### 对象存储配置

```toml
[cos]
provider = "tencent"

[cos.tencent]
secret_id = ""
secret_key = ""
bucket = ""
region = "ap-guangzhou"

[cos.rustfs]
root_path = "./uploads"
public_url_prefix = "/uploads"
auto_create_dir = true
```

完整的配置示例请参考 [config.example.toml](config.example.toml)。

## 环境变量

支持通过环境变量覆盖配置项：

```bash
# 服务器
export PORT=8080
export RUST_LOG=debug

# MongoDB
export MONGODB_IP=127.0.0.1
export MONGODB_PORT=27017
export MONGODB_USER=admin
export MONGODB_PASSWORD=secret

# Redis
export REDIS_IP=127.0.0.1
export REDIS_PORT=6379
export REDIS_PASSWORD=secret

# 认证
export TOKEN_SECRET=your-secret-key
```

## 辅助方法

```rust
use admin_config::{AppConfig, ToConnectionUrl};

let config = AppConfig::load()?;

let mongo_url = config.database.mongodb.to_connection_url();
let mysql_url = config.database.mysql.to_connection_url();
let redis_url = config.redis.connection_string();

let token_ttl = config.auth.token_expiry_seconds();
let refresh_ttl = config.auth.refresh_token_expiry_seconds();

let allowed_origins = config.security.allowed_origins_list();
```

## 安全密钥生成

配置中的所有安全密钥使用 Rust 的加密安全随机数生成器（CSPRNG）自动生成：

```rust
use admin_config::SecurityConfig;

let config = SecurityConfig::default();

assert_eq!(config.aes_key.len(), 64);
assert_eq!(config.aes_iv.len(), 32);

config.validate()?;
```

生成的密钥包括：

- `aes_key`: 64位十六进制（32字节，AES-256）
- `aes_iv`: 32位十六进制（16字节）
- `api_key_encrypt_key`: 64位十六进制（32字节）
- `password_salt`: 32位十六进制（16字节）

## 生产环境建议

1. **密钥管理**
   - 使用环境变量存储敏感信息
   - 使用密钥管理服务（AWS Secrets Manager、HashiCorp Vault）
   - 不要将真实密钥提交到 Git 仓库
   - 不要在多个环境共享密钥

2. **安全配置**
   - 定期更新密钥和密码
   - 启用 HTTPS 和 Secure Cookie
   - 设置适当的 CORS 策略

3. **配置文件管理**
   - 将 `config.toml` 添加到 `.gitignore`
   - 使用 `config.example.toml` 作为模板
   - 使用不同的配置文件区分环境

## API 文档

完整的 API 文档请访问 [docs.rs/admin-config](https://docs.rs/admin-config)

## 示例

查看 [config.example.toml](config.example.toml) 了解完整的配置示例。

## 贡献

欢迎提交 Issue 和 Pull Request！

## 许可

MIT License
