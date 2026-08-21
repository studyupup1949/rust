// Integration test: exercise the neutral UI + CRUD pipeline against a mock
// storage backend, proving pages render and form handlers behave — no database.

use adminx_core::prelude::*;
use adminx_core::response::ApiBody;
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::collections::HashMap;

struct MockStorage;

#[async_trait]
impl Storage for MockStorage {
    async fn list(&self, _t: &str, _o: &QueryOptions) -> Result<ListPage, StorageError> {
        Ok(ListPage {
            rows: vec![json!({"id": 1, "name": "Ada Lovelace", "email": "ada@analytical.io"})],
            total: 1,
        })
    }
    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
        Ok(Some(
            json!({"id": 1, "name": "Ada Lovelace", "email": "ada@analytical.io"}),
        ))
    }
    async fn create(&self, _t: &str, _d: Map<String, Value>) -> Result<CreateOutcome, StorageError> {
        Ok(CreateOutcome {
            last_insert_id: Some("2".into()),
        })
    }
    async fn update(&self, _t: &str, _pk: &str, _id: &str, _d: Map<String, Value>) -> Result<u64, StorageError> {
        Ok(1)
    }
    async fn delete(&self, _t: &str, _pk: &str, _id: &str, _soft: bool) -> Result<u64, StorageError> {
        Ok(1)
    }
    async fn health(&self) -> bool {
        true
    }
}

#[derive(Clone)]
struct UserResource;

#[async_trait]
impl Resource for UserResource {
    fn resource_name(&self) -> &'static str {
        "Users"
    }
    fn base_path(&self) -> &'static str {
        "users"
    }
    fn table_name(&self) -> &'static str {
        "users"
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    fn permit_keys(&self) -> Vec<&'static str> {
        vec!["name", "email"]
    }
}

fn body(resp: &ApiResponse) -> String {
    match &resp.body {
        ApiBody::Bytes { data, .. } => String::from_utf8_lossy(data).to_string(),
        ApiBody::Json(v) => v.to_string(),
        ApiBody::Empty => String::new(),
    }
}

#[tokio::test]
async fn ui_pipeline_renders_and_redirects() {
    set_storage(Box::new(MockStorage));
    register_resource(Box::new(UserResource));

    let ctx = ReqCtx::new().with_mount("/adminx");
    let res = UserResource;

    // List page renders the row and links.
    let list = res.list_page(&ctx).await;
    assert_eq!(list.status, 200);
    let html = body(&list);
    assert!(html.contains("Ada Lovelace"), "list should show the record");
    assert!(html.contains("/adminx/users/new"), "list should link to new");
    assert!(html.contains("/adminx/users/view/1"), "list should link to view");

    // New form has inputs for permitted fields.
    let new = res.new_page(&ctx).await;
    assert_eq!(new.status, 200);
    let new_html = body(&new);
    assert!(new_html.contains("name=\"name\""));
    assert!(new_html.contains("name=\"email\""));

    // View page renders the record.
    let view = res.view_page(&ctx, "1").await;
    assert!(body(&view).contains("ada@analytical.io"));

    // Dashboard lists the resource.
    let dash = adminx_core::ui::dashboard(&ctx);
    assert!(body(&dash).contains("Users"));

    // Create form redirects (303) to the list.
    let mut form = HashMap::new();
    form.insert("name".to_string(), "Bob".to_string());
    form.insert("email".to_string(), "bob@x.io".to_string());
    let created = res.create_form(&ctx, form).await;
    assert_eq!(created.status, 303);
    assert!(created
        .headers
        .iter()
        .any(|(k, v)| k == "Location" && v == "/adminx/users/list"));

    // JSON API list still works.
    let api = res.list(&ctx).await;
    assert_eq!(api.status, 200);
    match &api.body {
        ApiBody::Json(v) => assert_eq!(v["total"], 1),
        _ => panic!("API list should be JSON"),
    }
}
