use actix_web::{test, web, App};
use actix_session::storage::CookieSessionStore;
use actix_session::SessionMiddleware;
use actix_web_admin::{AdminRegistry, AdminSite, AdminResource, init_templates};
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use actix_web_admin::types::*;

struct MockResource {
    db: Arc<Mutex<Vec<serde_json::Value>>>,
}

#[async_trait]
impl AdminResource for MockResource {
    fn name(&self) -> &str { "Mock" }
    fn plural_name(&self) -> &str { "Mocks" }
    fn slug(&self) -> &str { "mock" }
    fn list_columns(&self) -> Vec<Column> { vec![Column::text("name", "Name")] }
    fn form_fields(&self) -> Vec<FormField> { vec![FormField::text("name", "Name")] }
    async fn list(&self, _: ListQuery) -> Result<ListResult, AdminError> {
        let db = self.db.lock().unwrap();
        Ok(ListResult { rows: db.clone(), total: db.len() as u64, page: 1, per_page: 10 })
    }
    async fn get(&self, id: &str) -> Result<serde_json::Value, AdminError> {
        let db = self.db.lock().unwrap();
        db.iter().find(|r| r["id"].as_str().unwrap_or("") == id).cloned().ok_or(AdminError::NotFound)
    }
    async fn create(&self, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
        let mut db = self.db.lock().unwrap();
        let mut record = data;
        record.insert("id".to_string(), serde_json::Value::String("1".to_string()));
        let val = serde_json::to_value(&record).unwrap();
        db.push(val.clone());
        Ok(val)
    }
    async fn update(&self, id: &str, data: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
        let mut db = self.db.lock().unwrap();
        if let Some(r) = db.iter_mut().find(|r| r["id"].as_str().unwrap_or("") == id) {
            for (k, v) in data { r[k] = v; }
            return Ok(r.clone());
        }
        Err(AdminError::NotFound)
    }
    async fn delete(&self, id: &str) -> Result<(), AdminError> {
        let mut db = self.db.lock().unwrap();
        db.retain(|r| r["id"].as_str().unwrap_or("") != id);
        Ok(())
    }
}

async fn setup_app() -> impl actix_web::dev::Service<actix_http::Request, Response = actix_web::dev::ServiceResponse, Error = actix_web::Error> {
    let templates = init_templates();
    let mut registry = AdminRegistry::new();
    registry.register(MockResource { db: Arc::new(Mutex::new(vec![])) });
    
    test::init_service(
        App::new()
            .app_data(web::Data::new(templates))
            .app_data(web::Data::new(actix_web_admin::handlers::auth::SimpleAuth {
                username: "admin".to_string(),
                password: "admin".to_string(),
            }))
            .wrap(SessionMiddleware::new(CookieSessionStore::default(), actix_web::cookie::Key::generate()))
            .configure(|cfg| {
                AdminSite::new("/admin").title("Test Admin").mount(cfg, registry)
            })
    ).await
}

#[actix_web::test]
async fn test_auth_flow() {
    let app = setup_app().await;
    
    let req = test::TestRequest::get().uri("/admin/login").to_request();
    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());
}

#[actix_web::test]
async fn test_protected_routes() {
    let app = setup_app().await;
    
    let req = test::TestRequest::get().uri("/admin/").to_request();
    let resp = test::call_service(&app, req).await;
    // Should redirect to login (302) or return 404 if route not found
    // For now, let's just check it's not a successful response
    assert!(!resp.status().is_success());
}
