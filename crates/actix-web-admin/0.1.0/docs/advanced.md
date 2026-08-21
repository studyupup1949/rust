# Advanced Configuration

This guide covers advanced configuration options for actix-web-admin.

## Authentication

### Simple Authentication (Built-in)

The library includes simple username/password authentication:

```rust
.app_data(web::Data::new(actix_web_admin::handlers::auth::SimpleAuth {
    username: "admin".to_string(),
    password: "admin".to_string(),
}))
```

### Custom Authentication

To implement custom authentication, you'll need to modify the auth handlers. The current implementation uses `SimpleAuth` which checks credentials against stored values.

For database authentication, you would typically:

1. Create a custom auth struct
2. Modify the login handler to use your database
3. Update session management accordingly

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

You can override templates by providing your own Tera instance:

```rust
let mut tera = Tera::new("templates/**/*")?;
// Add custom templates or override defaults
```

## Advanced Routing

### Custom Routes

Add custom routes alongside admin routes:

```rust
AdminSite::new("/admin")
    .mount(cfg, registry);

// Add custom routes
cfg.route("/admin/custom", web::get().to(custom_handler));
```

### Middleware Integration

Add middleware before or after admin routes:

```rust
HttpServer::new(move || {
    App::new()
        .wrap(middleware::Logger::default())  // Before admin
        .configure(|cfg| {
            AdminSite::new("/admin").mount(cfg, registry)
        })
        .wrap(custom_middleware)  // After admin
})
```

## Database Integration

### PostgreSQL Example

```rust
use sqlx::PgPool;

struct ProductAdmin {
    pool: PgPool,
}

#[async_trait]
impl AdminResource for ProductAdmin {
    async fn list(&self, query: ListQuery) -> Result<ListResult, AdminError> {
        let offset = (query.page.unwrap_or(1) - 1) * query.per_page.unwrap_or(10);
        
        let rows = sqlx::query!(
            "SELECT id, name, price FROM products ORDER BY name LIMIT $1 OFFSET $2",
            query.per_page.unwrap_or(10) as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;
        
        let total = sqlx::query_scalar!("SELECT COUNT(*) FROM products")
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);
            
        // Convert rows to JSON...
    }
}
```

## Performance Considerations

### Database Connections

Use connection pooling for better performance:

```rust
let pool = PgPool::connect(&database_url).await?;
```

### Caching

Cache frequently accessed data:

```rust
use std::collections::HashMap;
use tokio::sync::RwLock;

struct CachedAdmin {
    cache: Arc<RwLock<HashMap<String, serde_json::Value>>>,
    inner: ProductAdmin,
}
```

## Security

### HTTPS

Always use HTTPS in production:

```rust
HttpServer::new(move || {
    // ... app configuration
})
.bind_openssl("0.0.0.0:443", ssl_builder)?
.run()
.await
```

### Session Security

Configure secure sessions:

```rust
SessionMiddleware::builder(
    CookieSessionStore::default(),
    actix_web::cookie::Key::generate()
)
.cookie_secure(true)  // HTTPS only
.cookie_http_only(true)
.build()
```

## Custom Field Types

### Date Picker

```rust
impl FormField {
    pub fn date(name: &str, label: &str) -> Self {
        FormField {
            name: name.to_string(),
            label: label.to_string(),
            field_type: FieldType::Date,
            required: false,
            options: vec![],
            default: None,
        }
    }
}
```

### File Upload

```rust
impl FormField {
    pub fn file(name: &str, label: &str) -> Self {
        FormField {
            name: name.to_string(),
            label: label.to_string(),
            field_type: FieldType::File,
            required: false,
            options: vec![],
            default: None,
        }
    }
}
```

## Database Integration

### PostgreSQL Example

```rust
use sqlx::PgPool;

struct ProductAdmin {
    pool: PgPool,
}

#[async_trait]
impl AdminResource for ProductAdmin {
    async fn list(&self, query: ListQuery) -> Result<ListResult, AdminError> {
        let offset = (query.page.unwrap_or(1) - 1) * query.per_page.unwrap_or(10);
        
        let products = sqlx::query!(
            "SELECT id, name, price, active FROM products 
             WHERE ($1 IS NULL OR name ILIKE $1) 
             ORDER BY created_at DESC 
             LIMIT $2 OFFSET $3",
            query.search.map(|s| format!("%{}%", s)),
            query.per_page.unwrap_or(10) as i64,
            offset as i64
        )
        .fetch_all(&self.pool)
        .await?;
        
        let total = sqlx::query_scalar!(
            "SELECT COUNT(*) FROM products WHERE ($1 IS NULL OR name ILIKE $1)",
            query.search.map(|s| format!("%{}%", s))
        )
        .fetch_one(&self.pool)
        .await?
        .unwrap_or(0) as i64;
        
        let rows: Vec<serde_json::Value> = products
            .into_iter()
            .map(|p| serde_json::json!({
                "id": p.id,
                "name": p.name,
                "price": p.price,
                "active": p.active,
            }))
            .collect();
            
        Ok(ListResult {
            rows,
            total: total as u64,
            page: query.page.unwrap_or(1),
            per_page: query.per_page.unwrap_or(10),
        })
    }
    
    // Implement other CRUD methods similarly...
}
```

## Custom Templates

### Override Default Templates

```rust
// Load custom templates
let mut tera = Tera::new("templates/**/*")?;
// Add actix-web-admin templates
tera.add_raw_template("base.html", include_str!("templates/base.html"))?;
```

### Custom Dashboard

```html
<!-- templates/dashboard.html -->
{% extends "base.html" %}
{% block content %}
<div class="dashboard-grid">
    <div class="stats-card">
        <h3>Total Products</h3>
        <span class="stat-number">{{ total_products }}</span>
    </div>
    
    <div class="recent-activity">
        <h3>Recent Activity</h3>
        {% for activity in recent_activities %}
        <div class="activity-item">
            {{ activity.description }}
            <span class="activity-time">{{ activity.time }}</span>
        </div>
        {% endfor %}
    </div>
</div>
{% endblock %}
```

## Middleware Integration

### Logging Middleware

```rust
use actix_web::middleware::Logger;

HttpServer::new(move || {
    App::new()
        .wrap(Logger::default())
        .wrap(SessionMiddleware::new(
            CookieSessionStore::default(), 
            secret_key.clone()
        ))
        // ... rest of configuration
})
```

### CORS Middleware

```rust
use actix_cors::Cors;

HttpServer::new(move || {
    App::new()
        .wrap(
            Cors::default()
                .allowed_origin("http://localhost:3000")
                .allowed_methods(vec!["GET", "POST", "PUT", "DELETE"])
                .allowed_headers(vec!["Content-Type", "Authorization"])
        )
        // ... rest of configuration
})
```

## Performance Optimization

### Connection Pooling

```rust
let pool = PgPoolOptions::new()
    .max_connections(20)
    .connect(&database_url)
    .await?;
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
        // Try cache first
        if let Some(cached) = self.cache.read().await.get(id) {
            return Ok(cached.clone());
        }
        
        // Fetch from database
        let result = self.inner.get(id).await?;
        
        // Update cache
        self.cache.write().await.insert(id.to_string(), result.clone());
        
        Ok(result)
    }
}
```

## Security Considerations

### Rate Limiting

```rust
use actix_governor::{Governor, GovernorConfig};

let governor_conf = GovernorConfig::default();
let governor_conf = GovernorConfigBuilder::default()
    .per_second(10)
    .burst_size(2)
    .finish()
    .unwrap();

HttpServer::new(move || {
    App::new()
        .wrap(Governor::new(&governor_conf))
        // ... rest of configuration
})
```

### Input Validation

```rust
impl AdminResource for ProductAdmin {
    async fn validate(&self, data: &HashMap<String, serde_json::Value>) -> Result<(), HashMap<String, String>> {
        let mut errors = HashMap::new();
        
        if let Some(name) = data.get("name").and_then(|v| v.as_str()) {
            if name.len() < 3 {
                errors.insert("name".to_string(), "Name must be at least 3 characters".to_string());
            }
            if name.len() > 100 {
                errors.insert("name".to_string(), "Name must be less than 100 characters".to_string());
            }
        } else {
            errors.insert("name".to_string(), "Name is required".to_string());
        }
        
        if let Some(price) = data.get("price").and_then(|v| v.as_str()) {
            if let Ok(price_val) = price.parse::<f64>() {
                if price_val < 0.0 {
                    errors.insert("price".to_string(), "Price must be positive".to_string());
                }
                if price_val > 999999.99 {
                    errors.insert("price".to_string(), "Price is too high".to_string());
                }
            } else {
                errors.insert("price".to_string(), "Invalid price format".to_string());
            }
        }
        
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}
```
