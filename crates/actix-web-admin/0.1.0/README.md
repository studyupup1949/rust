# actix-web-admin

A powerful admin panel library for Actix-web 4 applications that automatically generates CRUD interfaces.

## Features

- 🚀 **Auto-generated CRUD operations** - Create, Read, Update, Delete interfaces
- 🔐 **Built-in authentication** - Simple session-based auth
- 🎨 **Modern responsive UI** - Clean, professional admin interface
- 🔍 **Search & pagination** - Built-in search and paginated lists
- 📝 **Form validation** - Automatic form generation with validation
- 🎯 **Type-safe** - Full Rust type safety with async trait support
- 📦 **Easy integration** - Just implement one trait and you're done

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

```rust
use actix_web_admin::{AdminRegistry, AdminSite, AdminResource, init_templates};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct Product {
    id: String,
    name: String,
    price: f64,
}

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
        ]
    }

    fn form_fields(&self) -> Vec<FormField> {
        vec![
            FormField::text("name", "Name").required(),
            FormField::number("price", "Price").required(),
        ]
    }

    async fn list(&self, query: ListQuery) -> Result<ListResult, AdminError> {
        let db = self.db.lock().unwrap();
        let rows: Vec<serde_json::Value> = db.iter().map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "price": p.price,
            })
        }).collect();
        
        Ok(ListResult { 
            rows, 
            total: rows.len() as u64, 
            page: query.page.unwrap_or(1), 
            per_page: query.per_page.unwrap_or(10) 
        })
    }

    async fn get(&self, id: &str) -> Result<serde_json::Value, AdminError> {
        let db = self.db.lock().unwrap();
        db.iter()
            .find(|p| p.id == id)
            .map(|p| serde_json::json!({
                "id": p.id,
                "name": p.name,
                "price": p.price,
            }))
            .ok_or(AdminError::NotFound)
    }

    async fn create(&self, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
        let mut db = self.db.lock().unwrap();
        let id = uuid::Uuid::new_v4().to_string();
        let p = Product {
            id: id.clone(),
            name: data.get("name").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            price: data.get("price").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()).unwrap_or(0.0),
        };
        db.push(p);
        Ok(serde_json::json!({ "id": id }))
    }

    async fn update(&self, id: &str, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
        let mut db = self.db.lock().unwrap();
        if let Some(p) = db.iter_mut().find(|p| p.id == id) {
            if let Some(v) = data.get("name").and_then(|v| v.as_str()) { p.name = v.to_string(); }
            if let Some(v) = data.get("price").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()) { p.price = v; }
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

Run the example and visit `http://localhost:8080/admin` - login with `admin`/`admin`.

## Configuration

### AdminSite

```rust
AdminSite::new("/admin")  // URL prefix
    .title("My Admin Panel")  // Site title
    .mount(cfg, registry)     // Mount to Actix app
```

### Resources

Each resource must implement the `AdminResource` trait:

- `name()` - Singular display name
- `plural_name()` - Plural display name  
- `slug()` - URL slug (must be unique)
- `icon()` - Emoji icon for dashboard
- `list_columns()` - Columns for list view
- `form_fields()` - Fields for create/edit forms
- CRUD operations: `list()`, `get()`, `create()`, `update()`, `delete()`

### Field Types

```rust
FormField::text("name", "Name").required()
FormField::number("price", "Price")
FormField::boolean("active", "Active")
FormField::select("category", "Category", vec![
    ("electronics", "Electronics"),
    ("clothing", "Clothing"),
])
FormField::textarea("description", "Description", 4)
```

### Column Types

```rust
Column::text("name", "Name")
Column::number("price", "Price")
Column::boolean("active", "Active")
Column::date("created_at", "Created")
```

## Authentication

Built-in simple authentication:

```rust
app_data(web::Data::new(actix_web_admin::handlers::auth::SimpleAuth {
    username: "admin".to_string(),
    password: "admin".to_string(),
}))
```

## Routes

```
/admin/                    # Dashboard
/admin/login              # Login page
/admin/logout             # Logout
/admin/{slug}/            # List view
/admin/{slug}/new         # Create form
/admin/{slug}/{id}        # Edit form
/admin/{slug}/{id}/delete # Delete action
```

## Important Notes

### Actix-Web Routing Behavior

This library follows Actix-web's routing philosophy. Some important considerations:

**Slash Handling:**
Actix-web doesn't automatically handle URLs with and without trailing slashes. The library configures both:

```rust
// In site.rs - both routes are configured
.route("", web::get().to(handlers::dashboard::index))
.route("/", web::get().to(handlers::dashboard::index))
```

**URL Construction:**
When constructing URLs in templates, use absolute paths to avoid double slashes:

```rust
// Instead of: format!("/{}/login", prefix.0)
// Use: "/admin/login"
```

**Template Variables:**
Always provide default values in Tera templates:

```jinja2
{{ query.search | default(value='') }}
{{ query.sort_dir | default(value='asc') }}
```

### Field Type Requirements

Some field types have additional required parameters:

```rust
// Textarea requires rows parameter
FormField::textarea("description", "Description", 4)  // rows = 4
```

### Authentication

The library includes simple authentication by default:

```rust
.app_data(web::Data::new(actix_web_admin::handlers::auth::SimpleAuth {
    username: "admin".to_string(),
    password: "admin".to_string(),
}))
```

## License

AGPL-3.0 - See [LICENSE](LICENSE) file for details.
