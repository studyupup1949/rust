# actix-web-admin

A powerful admin panel library for Actix-web 4 applications that automatically generates CRUD interfaces with a pluggable authentication system.

## Features

- 🚀 **Auto-generated CRUD operations** - Create, Read, Update, Delete interfaces
- 🔐 **Pluggable authentication** - Implement `UserStore` trait (built-in `JsonUserStore` included)
- 🛠️ **CLI tools** - `admin-cli` binary for user management, reusable `cli` module
- 🎨 **Modern responsive UI** - Clean, professional admin interface
- 🔍 **Search & pagination** - Built-in search and paginated lists
- 📝 **Form validation** - Automatic form generation with validation
- 🎯 **Type-safe** - Full Rust type safety with async trait support
- 📦 **Easy integration** - Implement one trait and you're done

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
actix-web-admin = "0.1.0"
actix-web = "4"
actix-session = { version = "0.9", features = ["cookie-session"] }
tokio = { version = "1", features = ["full"] }
```

### Basic Example

```rust,no_run
use actix_web::{web, App, HttpServer};
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web_admin::auth::{UserStore, JsonUserStore};
use actix_web_admin::{AdminRegistry, AdminSite, AdminResource, init_templates};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use actix_web_admin::types::*;

struct Product { id: String, name: String, price: f64, active: bool }

struct ProductAdmin { db: Arc<Mutex<Vec<Product>>> }

#[async_trait]
impl AdminResource for ProductAdmin {
    fn name(&self) -> &str { "Product" }
    fn plural_name(&self) -> &str { "Products" }
    fn slug(&self) -> &str { "products" }
    fn icon(&self) -> &str { "📦" }

    fn list_columns(&self) -> Vec<Column> {
        vec![Column::text("name", "Name"), Column::number("price", "Price"), Column::boolean("active", "Active")]
    }

    fn form_fields(&self) -> Vec<FormField> {
        vec![
            FormField::text("name", "Name").required(),
            FormField::number("price", "Price").required(),
        ]
    }

    async fn list(&self, query: ListQuery) -> Result<ListResult, AdminError> {
        let db = self.db.lock().unwrap();
        let rows: Vec<serde_json::Value> = db.iter().map(|p| serde_json::json!({"id": p.id, "name": p.name, "price": p.price, "active": p.active})).collect();
        Ok(ListResult { rows, total: rows.len() as u64, page: query.page.unwrap_or(1), per_page: query.per_page.unwrap_or(10) })
    }

    async fn get(&self, id: &str) -> Result<serde_json::Value, AdminError> {
        let db = self.db.lock().unwrap();
        db.iter().find(|p| p.id == id).map(|p| serde_json::json!({"id": p.id, "name": p.name, "price": p.price, "active": p.active})).ok_or(AdminError::NotFound)
    }

    async fn create(&self, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
        let mut db = self.db.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let name = data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string();
        let price = data.get("price").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0);
        let active = data.get("active").and_then(|v| v.as_bool()).unwrap_or(true);
        db.push(Product { id: id.clone(), name, price, active });
        Ok(serde_json::json!({ "id": id }))
    }

    async fn update(&self, id: &str, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
        let mut db = self.db.lock().unwrap();
        if let Some(p) = db.iter_mut().find(|p| p.id == id) {
            if let Some(v) = data.get("name").and_then(|v| v.as_str()) { p.name = v.to_string(); }
            if let Some(v) = data.get("price").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()) { p.price = v; }
            if let Some(v) = data.get("active").and_then(|v| v.as_bool()) { p.active = v; }
            return Ok(serde_json::json!({ "id": id }));
        }
        Err(AdminError::NotFound)
    }

    async fn delete(&self, id: &str) -> Result<(), AdminError> {
        let mut db = self.db.lock().unwrap();
        let len_before = db.len();
        db.retain(|p| p.id != id);
        if db.len() < len_before { Ok(()) } else { Err(AdminError::NotFound) }
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let templates = init_templates();

    let store = Arc::new(JsonUserStore::new("users.json"));
    if store.find_by_username("admin").await.unwrap().is_none() {
        store.create_user("admin", "admin@example.com", "Admin", "admin", true).await.unwrap();
    }

    let mut registry = AdminRegistry::new();
    registry.register(ProductAdmin { db: Arc::new(Mutex::new(vec![])) });

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(templates.clone()))
            .app_data(web::Data::new(store.clone() as Arc<dyn UserStore>))
            .wrap(SessionMiddleware::new(
                CookieSessionStore::default(),
                actix_web::cookie::Key::generate()
            ))
            .configure(|cfg| {
                AdminSite::new("/admin").title("My Admin").with_user_store(store.clone()).mount(cfg, registry)
            })
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
```

Run and visit `http://localhost:8080/admin` - login with `admin`/`admin`.

## Authentication

The library uses a `UserStore` trait for authentication. A `JsonUserStore` backed by a JSON file is included:

```rust
use actix_web_admin::auth::{UserStore, JsonUserStore};

let store = Arc::new(JsonUserStore::new("users.json"));
store.create_user("admin", "admin@example.com", "Admin", "secret", true).await?;
```

Pass the store to both the app data and `AdminSite`:

```rust
.app_data(web::Data::new(store.clone() as Arc<dyn UserStore>))
.configure(|cfg| {
    AdminSite::new("/admin")
        .with_user_store(store.clone())
        .mount(cfg, registry)
})
```

The `SimpleAuth` struct is deprecated — use `UserStore` instead.

## CLI Tools

### `admin-cli` binary

Install the crate and manage users from the command line:

```bash
cargo install actix-web-admin
admin-cli createsuperuser              # interactive user creation
admin-cli deleteuser <username>         # delete a user
admin-cli listusers                     # list all users
admin-cli --file /path/to/users.json    # specify custom JSON path
```

### Reusable `cli` module

Build your own CLI in ~10 lines by using the public `cli` module:

```rust
use actix_web_admin::cli;

#[tokio::main]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    match args.get(1).map(|s| s.as_str()) {
        Some("createsuperuser") => cli::create_superuser_interactive(&args[2..]).await.unwrap(),
        Some("deleteuser") => cli::delete_user(&args[2..]).await.unwrap(),
        Some("listusers") => cli::list_users(&args[2..]).await.unwrap(),
        _ => cli::print_help("my-cli"),
    }
}
```

All CLI functions are `async` — call them from within a Tokio runtime (`#[tokio::main]` or `#[actix_web::main]`).

## Configuration

### AdminSite

```rust
AdminSite::new("/admin")     // URL prefix (no trailing slash)
    .title("My Admin Panel")  // Site title
    .with_user_store(store)   // UserStore implementation
    .mount(cfg, registry)     // Mount routes
```

### Resources

Implement the `AdminResource` trait:

- `name()` — Singular display name
- `plural_name()` — Plural display name
- `slug()` — URL slug (must be unique)
- `icon()` — Emoji icon for dashboard
- `list_columns()` — Columns for list view
- `form_fields()` — Fields for create/edit forms
- CRUD: `list()`, `get()`, `create()`, `update()`, `delete()`

### Field Types

```rust
FormField::text("name", "Name").required()
FormField::number("price", "Price")
FormField::boolean("active", "Active")
FormField::select("category", "Category", vec![("a", "A"), ("b", "B")])
FormField::textarea("description", "Description", 4)
```

## Routes

All routes use the configured prefix dynamically:

```
/admin/             # Dashboard
/admin/login        # Login page
/admin/logout       # Logout
/admin/{slug}/      # List view
/admin/{slug}/new   # Create form
/admin/{slug}/{id}  # Edit form
/admin/{slug}/{id}/delete  # Delete action
```

The prefix is stored without a leading slash to avoid double-slash issues (e.g. `admin` not `/admin`).

## Testing

```bash
cargo test    # 23+ tests covering auth, middleware, handlers, CLI, resources
```

## Security Best Practices

This library provides secure defaults (Argon2 hashing, session management, HTML escaping), but the following should be configured in your deployment:

### CSRF Protection

Admin CRUD operations modify state via POST requests. Add a CSRF middleware to prevent cross-site request forgery:

```toml
actix-csrf = "0.3"
```

```rust
// Wrap your App with CSRF protection
use actix_csrf::Csrf;
use actix_csrf::storage::CookieStore;

App::new()
    .wrap(Csrf::new(CookieStore::default()))
    // ...
```

### Secure Cookies

Configure session cookies for production:

```rust
SessionMiddleware::builder(CookieSessionStore::default(), actix_web::cookie::Key::generate())
    .cookie_secure(true)       // HTTPS only
    .cookie_http_only(true)    // Not accessible via JavaScript
    .cookie_same_site(actix_web::cookie::SameSite::Lax)  // CSRF mitigation
    .build()
```

### Rate Limiting

Prevent brute force attacks on the login endpoint:

```toml
actix-governor = "0.6"
```

```rust
use actix_governor::{Governor, GovernorConfigBuilder};

let governor_conf = GovernorConfigBuilder::default()
    .per_second(5)
    .burst_size(10)
    .finish()
    .unwrap();

App::new()
    .wrap(Governor::new(&governor_conf))
    // ...
```

### HTTPS

Always use HTTPS in production:

```rust
HttpServer::new(move || { /* ... */ })
    .bind_openssl("0.0.0.0:443", ssl_builder)?
    .run().await
```

Or terminate TLS at a reverse proxy (nginx, Caddy, etc.).

### Resource IDs

The default `generate_id()` produces sequential IDs. Use UUIDs or ULIDs if enumeration is a concern for your application.

```rust
// In your AdminResource implementation:
let id = uuid::Uuid::new_v4().to_string();
```

## License

AGPL-3.0 — See [LICENSE](LICENSE) for details.
