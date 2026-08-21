# Admin Config

配置管理模块，提供统一的配置加载和管理功能。

## 功能特性

- ✅ TOML 配置文件支持
- ✅ 环境变量覆盖
- ✅ 类型安全的配置结构
- ✅ 默认值支持
- ✅ 多数据库实例配置

## 快速开始

### 添加依赖

```toml
[dependencies]
admin-config = { path = "../admin-config" }
```

### 使用配置

```rust
use admin_config::AppConfig;

fn main() -> anyhow::Result<()> {
    // 加载配置（支持环境变量覆盖）
    let config = AppConfig::load()?;

    // 使用配置
    println!("Server: {}:{}", config.server.host, config.server.port);

    // 获取数据库连接字符串
    if let Some(mongodb) = config.database.mongodb.first() {
        println!("MongoDB: {}", mongodb.connection_string());
    }

    Ok(())
}
```

## 配置方式

支持三种配置方式，优先级从高到低：

1. **环境变量**（最高优先级）- 支持两种格式：
    - 简写格式：`PORT`, `REDIS_IP`, `MONGODB_USER` 等
    - 层级格式：`SERVER__PORT`, `REDIS__HOST`, `DATABASE__MONGODB__USERNAME` 等
2. **config.toml 配置文件**
3. **默认值**（最低优先级）

配置文件路径可通过 `CONFIG_PATH` 环境变量指定，默认为 `config.toml`。

## 配置文件示例

### 服务器配置

```toml
[server]
name = "actix-admin-server"
version = "0.1.0"
port = 3400              # 环境变量: PORT 或 SERVER__PORT
host = "0.0.0.0"
log_level = "info"       # 环境变量: RUST_LOG 或 SERVER__LOG_LEVEL
```

### 数据库配置（支持多实例）

```toml
# MongoDB - 支持配置多个实例
[[database.mongodb]]
host = "localhost"       # 环境变量: MONGODB_IP
port = 27017            # 环境变量: MONGODB_PORT
database = "admin_db"
username = "admin"      # 环境变量: MONGODB_USER
password = "password"   # 环境变量: MONGODB_PASSWORD
max_pool_size = 10
connect_timeout = 10

# MySQL - 支持配置多个实例
[[database.mysql]]
host = "localhost"
port = 3306
database = "app_db"
username = "root"
password = "password"
max_pool_size = 10
connect_timeout = 10

# PostgreSQL
[[database.postgresql]]
host = "localhost"
port = 5432
database = "app_db"
username = "postgres"
password = "password"
max_pool_size = 10
connect_timeout = 10

# SQLite
[[database.sqlite]]
path = "./data.db"
max_pool_size = 10

# Qdrant (向量数据库)
[[database.qdrant]]
host = "localhost"
port = 6333
use_https = false
api_key = ""  # 可选

# SurrealDB
[[database.surrealdb]]
host = "localhost"
port = 8000
namespace = "app"
database = "main"
username = "root"
password = "root"
use_https = false
```

### Redis 配置

```toml
[redis]
host = "localhost"       # 环境变量: REDIS_IP 或 REDIS__HOST
port = 6379             # 环境变量: REDIS_PORT 或 REDIS__PORT
username = ""           # 可选
password = ""           # 环境变量: REDIS_PASSWORD 或 REDIS__PASSWORD
database = 0
max_pool_size = 20
connect_timeout = 5
```

### 认证配置

```toml
[auth]
token_secret = "your-secret-key"        # 环境变量: TOKEN_SECRET
token_expiry_hours = 24
refresh_token_expiry_days = 7
```

### LLM 配置（支持多个）

```toml
[[llm]]
provider = "openai"
api_key = "sk-xxx"
model = "gpt-4"
max_tokens = 2048
temperature = 0.7

[[llm]]
provider = "anthropic"
api_key = "sk-ant-xxx"
model = "claude-3-sonnet"
max_tokens = 4096
temperature = 0.7
```

### 其他配置

```toml
[email]
smtp_host = "smtp.gmail.com"
smtp_port = 587
smtp_username = ""
smtp_password = ""
from_name = "系统通知"
from_email = ""
enable_tls = true

[sms]
provider = "tencent"  # tencent/aliyun
app_id = ""
app_key = ""
sign_name = "您的应用"
template_id = "123456"

[verification_code]
length = 6
ttl = 300           # 秒
send_interval = 60  # 秒

[cos]
secret_id = ""
secret_key = ""
bucket = ""
region = "ap-guangzhou"
cdn_domain = ""  # 可选

[security]
aes_key = ""
aes_iv = ""
enable_cors = true
allowed_origins = "*"
enable_csrf = false

[session]
secret_key = ""
max_age = 86400     # 秒
http_only = true
secure = false
cookie_path = "/"

[upload]
allowed_types = "image/jpeg,image/png,image/gif,image/webp,application/pdf"
max_file_size = 10  # MB
upload_dir = "./uploads"
temp_dir = "./temp"

[rate_limit]
enabled = true
requests_per_minute = 60
burst_size = 10

[development]
enabled = false
debug = false
hot_reload = false
show_error_details = true
```

## 环境变量示例

创建 `.env` 文件：

```bash
# 服务器
PORT=8080
RUST_LOG=debug

# 数据库
MONGODB_IP=127.0.0.1
MONGODB_PORT=27017
MONGODB_USER=admin
MONGODB_PASSWORD=secret

# Redis
REDIS_IP=127.0.0.1
REDIS_PORT=6379
REDIS_PASSWORD=secret

# 认证
TOKEN_SECRET=your-secret-key-here

# LLM
LLM_PROVIDER=openai
LLM_API_KEY=sk-xxx
LLM_MODEL=gpt-4
```

## 辅助方法

```rust
// 获取数据库连接字符串
let mongo_url = config.database.mongodb[0].connection_string();
let mysql_url = config.database.mysql[0].connection_string();
let redis_url = config.redis.connection_string();

// Token 过期时间（秒）
let token_ttl = config.auth.token_expiry_seconds();
let refresh_ttl = config.auth.refresh_token_expiry_seconds();

// 上传文件大小限制（字节）
let max_size = config.upload.max_file_size_bytes();

// 允许的文件类型列表
let allowed_types = config.upload.allowed_types_list();
```

## 生产环境建议

1. 使用环境变量存储敏感信息（密码、密钥）
2. 定期更新密钥和密码
3. 启用 HTTPS 和 Secure Cookie
4. 设置适当的 CORS 策略
5. 启用限流保护

## 许可

MIT
