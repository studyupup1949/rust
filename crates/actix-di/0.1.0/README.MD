# actix-di

[![Crates.io](https://img.shields.io/crates/v/actix-di.svg)](https://crates.io/crates/actix-di)
[![Documentation](https://docs.rs/actix-di/badge.svg)](https://docs.rs/actix-di)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A type-safe, ergonomic dependency injection system for Actix web applications.

## Features

- 🔒 **Type-safe dependency injection**: Compile-time dependency checking
- 🎯 **Zero-cost abstractions**: Minimal runtime overhead
- 🔄 **Service lifecycle management**: Controlled initialization and shutdown
- 📦 **Easy integration**: Seamless integration with Actix web
- 🧩 **Modular design**: Flexible service registration and retrieval
- 🛡️ **Error handling**: Comprehensive error types and handling
- 📝 **Logging**: Built-in tracing support

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
actix-di = "0.1.0"
```

## Quick Start

Here's a simple example of how to use actix-di:

```rust
use actix_di::prelude::*;
use async_trait::async_trait;
use std::sync::Arc;

// Define your services
struct DatabaseService;

#[async_trait]
impl Service for DatabaseService {
    async fn init(&self) -> Result<(), ServiceError> {
        // Initialize database connection
        Ok(())
    }
}

impl DependencyProvider for DatabaseService {
    fn required_services() -> Vec<std::any::TypeId> {
        vec![] // No dependencies
    }
}

// Service that depends on DatabaseService
struct UserService {
    db: Arc<DatabaseService>,
}

#[async_trait]
impl Service for UserService {}

impl DependencyProvider for UserService {
    provide_dependencies!(DatabaseService);
}

// Use in Actix handler
async fn handler(data: web::Data<AppState>) -> impl Responder {
    let user_service = inject_service!(data, UserService)?;
    // Use user_service...
    HttpResponse::Ok().finish()
}

// Setup in main
#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let mut registry = ServiceRegistry::new();
    
    // Register services
    let db_service = Arc::new(DatabaseService);
    registry.register(db_service.clone())?;
    
    let user_service = Arc::new(UserService { db: db_service });
    registry.register(user_service)?;
    
    // Initialize all services
    registry.init_all().await?;
    
    let app_state = AppState::new(registry);
    
    HttpServer::new(move || {
        App::new()
            .app_data(app_state.clone().into())
            .service(handler)
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}
```

## Advanced Usage

### Custom Service Initialization

```rust
#[async_trait]
impl Service for MyService {
    async fn init(&self) -> Result<(), ServiceError> {
        // Custom initialization logic
        Ok(())
    }
    
    async fn shutdown(&self) -> Result<(), ServiceError> {
        // Custom shutdown logic
        Ok(())
    }
}
```

### Multiple Dependencies

```rust
impl DependencyProvider for MyService {
    provide_dependencies!(DatabaseService, CacheService, EmailService);
}
```

### Error Handling

```rust
async fn handler(data: web::Data<AppState>) -> Result<HttpResponse, actix_web::Error> {
    let service = inject_service!(data, MyService)
        .map_err(|e| actix_web::error::ErrorInternalServerError(e))?;
    
    Ok(HttpResponse::Ok().finish())
}
```

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. For major changes, please open an issue first to discuss what you would like to change.

### Development

1. Clone the repository
2. Install dependencies: `cargo build`
3. Run tests: `cargo test`

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by dependency injection systems in other ecosystems
- Built on top of the excellent [Actix web](https://actix.rs/) framework