# actix-web-schema

[![Crates.io](https://img.shields.io/crates/v/actix-web-schema)](https://crates.io/crates/actix-web-schema)
[![Documentation](https://img.shields.io/docsrs/actix-web-schema)](https://docs.rs/actix-web-schema)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

一个为 [actix-web](https://github.com/actix/actix-web) 提供模式化开发支持的工具库。通过派生宏简化路由定义和响应处理，让你的代码更加简洁和声明式。

## 特性

- **`#[service]`** - 通过 trait 定义 HTTP 服务，自动生成路由配置
- **`#[response]`** - 通过结构体定义响应格式，自动实现 `Responder`
- **`#[request]`** - 通过结构体定义请求格式，自动实现 `Deserialize`
- 统一的响应格式包装（`{code: 0, data: ...}`）
- 类型安全的路由定义
- 零运行时开销

## 安装

在 `Cargo.toml` 中添加：

```toml
[dependencies]
actix-web-schema = "0.1"
```

## 快速开始

### 定义服务

使用 `#[service]` 宏将 trait 转换为 HTTP 服务：

```rust
use actix_web_schema::service;
use actix_web::web;

/// 用户服务
#[service]
pub trait UserService {
    /// 获取用户信息
    #[get("/users/{id}")]
    async fn get_user(id: web::Path<u32>) -> User;

    /// 创建用户
    #[post("/users")]
    async fn create_user(user: web::Json<CreateUserRequest>) -> User;

    /// 删除用户
    #[delete("/users/{id}")]
    async fn delete_user(id: web::Path<u32>) -> ();
}
```

### 定义请求

使用 `#[request]` 宏定义请求结构：

```rust
use actix_web_schema::request;

#[request]
pub struct CreateUserRequest {
    name: String,
    email: String,
}
```

### 定义响应

使用 `#[response]` 宏定义响应结构：

```rust
use actix_web_schema::response;
use serde::Serialize;

#[response]
pub struct User {
    id: u32,
    name: String,
    email: String,
}

// User 自动实现 Responder，响应格式为：
// {"code": 0, "data": {"id": 1, "name": "...", "email": "..."}}
```

### 完整示例

```rust
use actix_web::{App, HttpServer, web};
use actix_web_schema::{service, response, request};

// 定义响应结构
#[response]
pub struct GreetingResponse {
    message: String,
}

// 定义服务
#[service]
pub trait HelloService {
    /// 问候接口
    #[get("/hello/{name}")]
    async fn hello(name: web::Path<String>) -> GreetingResponse;

    /// 登录接口
    #[post("/login")]
    async fn login(req: web::Json<LoginRequest>) -> LoginResponse;
}

// 定义请求结构
#[request]
pub struct LoginRequest {
    username: String,
    password: String,
}

#[response]
pub struct LoginResponse {
    token: String,
}

// 实现服务
impl HelloService for HelloService {
    async fn hello(name: web::Path<String>) -> GreetingResponse {
        GreetingResponse {
            message: format!("Hello, {}!", name.into_inner()),
        }
    }

    async fn login(req: web::Json<LoginRequest>) -> LoginResponse {
        // 验证逻辑...
        LoginResponse {
            token: format!("token_for_{}", req.username),
        }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(HelloService)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

访问 `GET /hello/world` 返回：
```json
{"code": 0, "data": {"message": "Hello, world!"}}
```

访问 `POST /login` 返回：
```json
{"code": 0, "data": {"token": "token_for_user"}}
```

## 工作原理

### `#[service]` 宏

`#[service]` 宏处理 trait 定义：

1. 为每个带有 HTTP 方法属性的方法生成路由配置
2. 创建对应的 `Service` 结构体并实现 `HttpServiceFactory`
3. 过滤掉 HTTP 方法属性，保持原有 trait 定义

### `#[response]` 宏

`#[response]` 宏处理结构体定义：

1. 自动添加 `#[derive(Serialize)]`
2. 实现 `Responder` trait
3. 以统一格式 `{"code": 0, "data": ...}` 返回 JSON 响应

### `#[request]` 宏

`#[request]` 宏处理结构体定义：

1. 自动添加 `#[derive(Deserialize)]`
2. 用于定义请求体结构，可直接作为 `web::Json<T>` 的类型参数

## 支持的 HTTP 方法

- `GET` - `#[get("/path")]`
- `POST` - `#[post("/path")]`
- `PUT` - `#[put("/path")]`
- `DELETE` - `#[delete("/path")]`
- `PATCH` - `#[patch("/path")]`
- `HEAD` - `#[head("/path")]`
- `OPTIONS` - `#[options("/path")]`

## 项目结构

```
actix-web-schema/
├── actix-web-schema/         # 主库
│   └── src/lib.rs            # 导出宏
└── actix-web-schema-macro/   # 过程宏实现
    └── src/lib.rs            # 宏定义
```

## License

MIT OR Apache-2.0

## 贡献

欢迎提交 Issue 和 Pull Request！

## 相关项目

- [actix-web](https://github.com/actix/actix-web) - 强大的 Rust Web 框架
- [serde](https://github.com/serde-rs/serde) - 序列化/反序列化框架