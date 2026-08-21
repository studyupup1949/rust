# Quick Start Guide

This guide will help you get actix-web-admin running in minutes.

## Prerequisites

- Rust 1.70+
- Basic knowledge of Actix-web

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
actix-web-admin = "0.1.0"
actix-web = "4"
actix-session = { version = "0.9", features = ["cookie-session"] }
tokio = { version = "1", features = ["full"] }
```

## Your First Admin Panel

### Step 1: Define Your Data Model

```rust
struct Product {
    id: String,
    name: String,
    price: f64,
    active: bool,
}
```

### Step 2: Implement AdminResource

```rust
use actix_web_admin::{AdminResource, types::*};
use async_trait::async_trait;
use std::collections::HashMap;

struct ProductAdmin { db: Vec<Product> }

#[async_trait]
impl AdminResource for ProductAdmin {
    fn name(&self) -> &str { "Product" }
    fn plural_name(&self) -> &str { "Products" }
    fn slug(&self) -> &str { "products" }
    fn icon(&self) -> &str { "📦" }

    fn list_columns(&self) -> Vec<Column> {
        vec![
            Column::text("name", "Name"),
            Column::number("price", "Price"),
            Column::boolean("active", "Active"),
        ]
    }

    fn form_fields(&self) -> Vec<FormField> {
        vec![
            FormField::text("name", "Name").required(),
            FormField::number("price", "Price").required(),
        ]
    }

    // Implement CRUD methods...
}
```

### Step 3: Configure Your Server (UserStore Auth)

```rust
use actix_web::{web, App, HttpServer};
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web_admin::auth::{UserStore, JsonUserStore};
use actix_web_admin::{AdminRegistry, AdminSite, init_templates};
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let templates = init_templates();

    // Set up the authentication store
    let store = Arc::new(JsonUserStore::new("users.json"));
    if store.find_by_username("admin").await.unwrap().is_none() {
        store.create_user("admin", "admin@example.com", "Admin", "admin", true).await.unwrap();
    }

    let mut registry = AdminRegistry::new();
    registry.register(ProductAdmin { db: vec![] });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(templates.clone()))
            .app_data(web::Data::new(store.clone() as Arc<dyn UserStore>))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                actix_web::cookie::Key::generate()
            ))
            .configure(|cfg| {
                AdminSite::new("/admin")
                    .title("My Admin")
                    .with_user_store(store.clone())
                    .mount(cfg, registry)
            })
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

### Step 4: Run and Create a Superuser

```bash
# Install the CLI tool
cargo install actix-web-admin

# Create your first admin user
admin-cli --file users.json createsuperuser

# Or use the binary from your project
cargo run --bin admin-cli -- createsuperuser
```

Visit `http://localhost:8080/admin` and login.

## What You Get

- 📊 **Dashboard** — Overview of all resources
- 📝 **CRUD Operations** — Create, Read, Update, Delete
- 🔍 **Search & Filter** — Built-in search functionality
- 📱 **Responsive Design** — Works on mobile and desktop
- 🔐 **Authentication** — Pluggable UserStore (JsonUserStore included)
- 🛠️ **CLI** — User management via `admin-cli` or custom binary

## Next Steps

- [Advanced Configuration](advanced.md)
- [Security Best Practices](advanced.md#security)
- [Troubleshooting Guide](troubleshooting.md)
- [Custom Field Types](custom-fields.md)
- [Database Integration](database.md)
