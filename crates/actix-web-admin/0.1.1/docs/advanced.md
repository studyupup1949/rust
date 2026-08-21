# Advanced Configuration

This guide covers advanced configuration options for actix-web-admin.

## Authentication

### UserStore Trait (Recommended)

The library uses the `UserStore` trait for authentication. Implement it for your database:

```rust
use actix_web_admin::auth::{UserStore, AuthError, User};
use async_trait::async_trait;
use std::sync::Arc;

struct MyDatabaseStore {
    pool: sqlx::PgPool,
}

#[async_trait]
impl UserStore for MyDatabaseStore {
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, AuthError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
            .bind(username)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;
        Ok(user)
    }

    async fn find_by_email(&self, email: &str) -> Result<Option<User>, AuthError> {
        let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;
        Ok(user)
    }

    async fn create_user(&self, username: &str, email: &str, name: &str, password: &str, is_superuser: bool) -> Result<User, AuthError> {
        let hash = actix_web_admin::auth::hash_password(password)?;
        let user = sqlx::query_as::<_, User>(
            "INSERT INTO users (username, email, name, password_hash, is_superuser) VALUES ($1, $2, $3, $4, $5) RETURNING *"
        )
        .bind(username).bind(email).bind(name).bind(&hash).bind(is_superuser)
        .fetch_one(&self.pool)
        .await
        .map_err(|e| AuthError::Database(e.to_string()))?;
        Ok(user)
    }

    async fn delete_user(&self, username: &str) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM users WHERE username = $1")
            .bind(username)
            .execute(&self.pool)
            .await
            .map_err(|e| AuthError::Database(e.to_string()))?;
        Ok(())
    }

    fn all_users(&self) -> Result<Vec<User>, AuthError> {
        let users = sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY username")
            .fetch_all(&self.pool);
        // Note: all_users is synchronous in the trait — use a cached list or block_on
        Ok(users)
    }
}
```

Then pass it to actix-web-admin:

```rust
let store = Arc::new(MyDatabaseStore { pool }) as Arc<dyn UserStore>;

App::new()
    .app_data(web::Data::new(store.clone()))
    .configure(|cfg| {
        AdminSite::new("/admin")
            .with_user_store(store.clone())
            .mount(cfg, registry)
    })
```

### JsonUserStore (Built-in)

For local development or simple deployments, `JsonUserStore` persists users to a JSON file:

```rust
use actix_web_admin::auth::JsonUserStore;
use std::sync::Arc;

let store = Arc::new(JsonUserStore::new("users.json"));
```

### SimpleAuth (Deprecated)

`SimpleAuth` is deprecated. Migrate to the `UserStore` trait pattern above.

---

## CLI Module

### Using the `cli` Module in Your Own Binary

The public `cli` module exposes async functions you can use in your own CLI:

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

All functions require an active Tokio runtime (`#[tokio::main]` or `#[actix_web::main]`).

### Custom User Store CLI

If you use a custom database instead of `JsonUserStore`, write your own CLI:

```rust
use actix_web_admin::cli;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let store = Arc::new(MyDatabaseStore::new());
    let args: Vec<String> = std::env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("createsuperuser") => {
            let username = dialoguer::Input::new().with_prompt("Username").interact_text().unwrap();
            let password = dialoguer::Password::new().with_prompt("Password").interact().unwrap();
            store.create_user(&username, "", &username, &password, true).await.unwrap();
            println!("Superuser '{}' created.", username);
        }
        _ => cli::print_help("my-cli"),
    }
}
```

---

## Template Customization

### Template Variables

All templates receive these common variables:

```rust
ctx.insert("title", &site_title);
ctx.insert("page_title", &current_page);
ctx.insert("user", &authenticated_user);
ctx.insert("path_dashboard", &dashboard_path);
ctx.insert("path_logout", &logout_path);
```

### Custom Templates

Override templates by providing your own Tera instance:

```rust
let mut tera = Tera::new("templates/**/*")?;
tera.add_raw_template("base.html", include_str!("templates/base.html"))?;
```

## Routing

### Dynamic Admin Prefix

The prefix is stored without a leading slash. All handlers construct paths dynamically:

```rust
// prefix.0 == "admin" (no leading slash)
let dashboard_url = format!("/{}/", prefix.0);
let login_url = format!("/{}/login", prefix.0);
let logout_url = format!("/{}/logout", prefix.0);
```

This avoids double-slash issues (e.g. `//admin/login`).

### Custom Routes

Add custom routes alongside admin routes:

```rust
AdminSite::new("/admin").mount(cfg, registry);
cfg.route("/admin/custom", web::get().to(custom_handler));
```

### Middleware Integration

```rust
HttpServer::new(move || {
    App::new()
        .wrap(middleware::Logger::default())
        .configure(|cfg| {
            AdminSite::new("/admin").mount(cfg, registry)
        })
        .wrap(custom_middleware)
})
```

## Database Integration

### PostgreSQL Example

```rust
use sqlx::PgPool;

struct ProductAdmin { pool: PgPool }

#[async_trait]
impl AdminResource for ProductAdmin {
    async fn list(&self, query: ListQuery) -> Result<ListResult, AdminError> {
        let offset = (query.page.unwrap_or(1) - 1) * query.per_page.unwrap_or(10);
        let rows = sqlx::query!("SELECT id, name, price, active FROM products ORDER BY name LIMIT $1 OFFSET $2",
            query.per_page.unwrap_or(10) as i64, offset as i64)
            .fetch_all(&self.pool).await?;
        let total = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
            .fetch_one(&self.pool).await?.unwrap_or(0);
        let rows: Vec<serde_json::Value> = rows.into_iter().map(|p| serde_json::json!({
            "id": p.id, "name": p.name, "price": p.price, "active": p.active
        })).collect();
        Ok(ListResult { rows, total: total as u64, page: query.page.unwrap_or(1), per_page: query.per_page.unwrap_or(10) })
    }
    // Implement get, create, update, delete similarly...
}
```

## Performance Considerations

### Connection Pooling

```rust
let pool = PgPoolOptions::new().max_connections(20).connect(&database_url).await?;
```

### Caching

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

struct CachedResource {
    cache: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    inner: Arc<dyn AdminResource>,
}

#[async_trait]
impl AdminResource for CachedResource {
    async fn get(&self, id: &str) -> Result<serde_json::Value, AdminError> {
        if let Some(cached) = self.cache.read().await.get(id) {
            return Ok(cached.clone());
        }
        let result = self.inner.get(id).await?;
        self.cache.write().await.insert(id.to_string(), result.clone());
        Ok(result)
    }
}
```

## Security

### CSRF Protection

Admin CRUD operations modify state via POST requests. Add a CSRF middleware to prevent cross-site request forgery:

```toml
actix-csrf = "0.3"
```

```rust
use actix_csrf::Csrf;
use actix_csrf::storage::CookieStore;

App::new()
    .wrap(Csrf::new(CookieStore::default()))
    .configure(|cfg| {
        AdminSite::new("/admin").mount(cfg, registry)
    })
```

### Secure Cookies

Configure session cookies for production to prevent XSS and CSRF attacks:

```rust
SessionMiddleware::builder(CookieSessionStore::default(), actix_web::cookie::Key::generate())
    .cookie_secure(true)           // HTTPS only
    .cookie_http_only(true)        // Not accessible via JavaScript
    .cookie_same_site(actix_web::cookie::SameSite::Lax)  // CSRF mitigation
    .build()
```

### HTTPS

Always use HTTPS in production — either directly or via a reverse proxy (nginx, Caddy):

```rust
HttpServer::new(move || { /* ... */ })
    .bind_openssl("0.0.0.0:443", ssl_builder)?
    .run().await
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

HttpServer::new(move || {
    App::new()
        .wrap(Governor::new(&governor_conf))
        .configure(|cfg| {
            AdminSite::new("/admin").mount(cfg, registry)
        })
})
```

### Resource IDs

The default `generate_id()` produces sequential IDs (timestamp + atomic counter). Use UUIDs or ULIDs if ID enumeration is a concern:

```rust
// In your AdminResource implementation:
async fn create(&self, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
    let id = uuid::Uuid::new_v4().to_string();
    // ...
}
```

### Input Validation

Validate data in your `AdminResource` implementations:

```rust
impl AdminResource for ProductAdmin {
    async fn validate(&self, data: &HashMap<String, serde_json::Value>) -> Result<(), HashMap<String, String>> {
        let mut errors = HashMap::new();
        if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
            if name.len() < 3 { errors.insert("name".to_string(), "Too short".to_string()); }
        } else {
            errors.insert("name".to_string(), "Required".to_string());
        }
        if errors.is_empty() { Ok(()) } else { Err(errors) }
    }
}
```

## Custom Templates

### Override Default Templates

```rust
let mut tera = Tera::new("templates/**/*")?;
tera.add_raw_template("base.html", include_str!("templates/base.html"))?;
```
