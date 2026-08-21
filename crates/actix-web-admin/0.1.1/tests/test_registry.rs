use actix_web_admin::{AdminRegistry, AdminResource};
use actix_web_admin::types::*;
use async_trait::async_trait;
use std::collections::HashMap;

struct MockResource {
    slug: String,
}

#[async_trait]
impl AdminResource for MockResource {
    fn name(&self) -> &str { "Mock" }
    fn plural_name(&self) -> &str { "Mocks" }
    fn slug(&self) -> &str { &self.slug }
    fn list_columns(&self) -> Vec<Column> { vec![] }
    fn form_fields(&self) -> Vec<FormField> { vec![] }
    async fn list(&self, _: ListQuery) -> Result<ListResult, AdminError> {
        Ok(ListResult { rows: vec![], total: 0, page: 1, per_page: 10 })
    }
    async fn get(&self, _: &str) -> Result<serde_json::Value, AdminError> {
        Ok(serde_json::json!({}))
    }
    async fn create(&self, _: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
        Ok(serde_json::json!({}))
    }
    async fn update(&self, _: &str, _: HashMap<String, serde_json::Value>) -> Result<serde_json::Value, AdminError> {
        Ok(serde_json::json!({}))
    }
    async fn delete(&self, _: &str) -> Result<(), AdminError> {
        Ok(())
    }
}

#[test]
fn test_register_and_get() {
    let mut registry = AdminRegistry::new();
    registry.register(MockResource { slug: "mock".to_string() });
    assert!(registry.get("mock").is_some());
    assert!(registry.get("nonexistent").is_none());
}

#[test]
fn test_insertion_order() {
    let mut registry = AdminRegistry::new();
    registry.register(MockResource { slug: "a".to_string() });
    registry.register(MockResource { slug: "b".to_string() });
    registry.register(MockResource { slug: "c".to_string() });
    
    let all = registry.all();
    assert_eq!(all[0].slug(), "a");
    assert_eq!(all[1].slug(), "b");
    assert_eq!(all[2].slug(), "c");
}

#[test]
#[should_panic(expected = "already registered")]
fn test_duplicate_slug_panics() {
    let mut registry = AdminRegistry::new();
    registry.register(MockResource { slug: "mock".to_string() });
    registry.register(MockResource { slug: "mock".to_string() });
}
