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
use std::sync::{Arc, Mutex};

struct ProductAdmin {
    db: Arc<Mutex<Vec<Product>>>,
}

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
            FormField::textarea("description", "Description", 4),
        ]
    }

    // Implement CRUD methods...
}
```

### Step 3: Configure Your Server

```rust
use actix_web_admin::{AdminRegistry, AdminSite, init_templates};
use actix_session::SessionMiddleware;
use actix_session::storage::CookieSessionStore;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let templates = init_templates();
    let mut registry = AdminRegistry::new();
    registry.register(ProductAdmin {
        db: Arc::new(Mutex::new(vec![])),
    });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(templates.clone()))
            .app_data(web::Data::new(actix_web_admin::handlers::auth::SimpleAuth {
                username: "admin".to_string(),
                password: "admin".to_string(),
            }))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(), 
                actix_web::cookie::Key::generate()
            ))
            .configure(|cfg| {
                AdminSite::new("/admin").title("My Admin").mount(cfg, registry)
            })
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

### Step 4: Run and Access

```bash
cargo run
```

Visit `http://localhost:8080/admin` and login with:
- Username: `admin`
- Password: `admin`

## What You Get

- 📊 **Dashboard** - Overview of all resources
- 📝 **CRUD Operations** - Create, Read, Update, Delete
- 🔍 **Search & Filter** - Built-in search functionality
- 📱 **Responsive Design** - Works on mobile and desktop
- 🔐 **Authentication** - Session-based login system

## Next Steps

- [Advanced Configuration](advanced.md)
- [Custom Field Types](custom-fields.md)
- [Database Integration](database.md)
- [Authentication Methods](auth.md)
