use actix_web::{web, App, HttpServer};
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web_admin::{AdminRegistry, AdminSite, AdminResource, init_templates};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use actix_web_admin::types::*;

struct Product {
    id: String,
    name: String,
    price: f64,
    active: bool,
    category: String,
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
            Column::boolean("active", "Active"),
            Column::text("category", "Category"),
        ]
    }

    fn form_fields(&self) -> Vec<FormField> {
        vec![
            FormField::text("name", "Name").required(),
            FormField::number("price", "Price").required(),
            FormField::boolean("active", "Active"),
            FormField::select("category", "Category", vec![
                ("electronics", "Electronics"),
                ("clothing", "Clothing"),
                ("food", "Food"),
            ]).required(),
        ]
    }

    async fn list(&self, query: ListQuery) -> Result<ListResult, AdminError> {
        let db = self.db.lock().unwrap();
        let mut rows: Vec<serde_json::Value> = db.iter().map(|p| {
            serde_json::json!({
                "id": p.id,
                "name": p.name,
                "price": p.price,
                "active": p.active,
                "category": p.category,
            })
        }).collect();

        if let Some(ref s) = query.search {
            let s = s.to_lowercase();
            rows.retain(|r| {
                r["name"].as_str().map(|n| n.to_lowercase().contains(&s)).unwrap_or(false) ||
                r["category"].as_str().map(|c| c.to_lowercase().contains(&s)).unwrap_or(false)
            });
        }

        let total = rows.len() as u64;
        let per_page = query.per_page.unwrap_or(10);
        let page = query.page.unwrap_or(1);
        let start = ((page - 1) * per_page) as usize;
        
        let rows = if start < rows.len() {
            let end = (start + per_page as usize).min(rows.len());
            rows[start..end].to_vec()
        } else {
            vec![]
        };

        Ok(ListResult { rows, total, page, per_page })
    }

    async fn get(&self, id: &str) -> Result<serde_json::Value, AdminError> {
        let db = self.db.lock().unwrap();
        db.iter()
            .find(|p| p.id == id)
            .map(|p| serde_json::json!({
                "id": p.id,
                "name": p.name,
                "price": p.price,
                "active": p.active,
                "category": p.category,
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
            active: data.get("active").and_then(|v| v.as_bool()).unwrap_or(true),
            category: data.get("category").and_then(|v| v.as_str()).unwrap_or("").to_string(),
        };
        db.push(p);
        Ok(serde_json::json!({ "id": id }))
    }

    async fn update(&self, id: &str, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
        let mut db = self.db.lock().unwrap();
        if let Some(p) = db.iter_mut().find(|p| p.id == id) {
            if let Some(v) = data.get("name").and_then(|v| v.as_str()) { p.name = v.to_string(); }
            if let Some(v) = data.get("price").and_then(|v| v.as_str()).and_then(|s| s.parse().ok()) { p.price = v; }
            if let Some(v) = data.get("active").and_then(|v| v.as_bool()) { p.active = v; }
            if let Some(v) = data.get("category").and_then(|v| v.as_str()) { p.category = v.to_string(); }
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
    // Initialize logging
    env_logger::init();
    
    let templates = init_templates();
    let mut registry = AdminRegistry::new();
    registry.register(ProductAdmin {
        db: Arc::new(Mutex::new(vec![
            Product { id: "1".to_string(), name: "Laptop".to_string(), price: 999.99, active: true, category: "electronics".to_string() },
            Product { id: "2".to_string(), name: "T-Shirt".to_string(), price: 19.99, active: true, category: "clothing".to_string() },
        ])),
    });

    let _admin_site = AdminSite::new("/admin").title("My Store Admin");
    
    let secret_key = actix_web::cookie::Key::from(b"a_very_long_secret_key_that_is_exactly_64_bytes_long_123456789012");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(templates.clone()))
            .app_data(web::Data::new(actix_web_admin::handlers::auth::SimpleAuth {
                username: "admin".to_string(),
                password: "admin".to_string(),
            }))
            .wrap(SessionMiddleware::new(CookieSessionStore::default(), secret_key.clone()))
            .configure(|cfg| {
                let mut reg = AdminRegistry::new();
                reg.register(ProductAdmin {
                    db: Arc::new(Mutex::new(vec![
                        Product { id: "1".to_string(), name: "Laptop".to_string(), price: 999.99, active: true, category: "electronics".to_string() },
                        Product { id: "2".to_string(), name: "T-Shirt".to_string(), price: 19.99, active: true, category: "clothing".to_string() },
                    ])),
                });
                AdminSite::new("/admin").title("My Store Admin").mount(cfg, reg)
            })
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
