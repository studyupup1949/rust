// Custom actions + CSV/JSON export against a mock storage backend.

use adminx_core::actions::{ActionFuture, CustomAction};
use adminx_core::prelude::*;
use adminx_core::response::ApiBody;
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use async_trait::async_trait;
use serde_json::{json, Map, Value};

struct Mock;

#[async_trait]
impl Storage for Mock {
    async fn list(&self, _t: &str, _o: &QueryOptions) -> Result<ListPage, StorageError> {
        Ok(ListPage {
            rows: vec![
                json!({"id": 1, "name": "Ada"}),
                json!({"id": 2, "name": "Grace, \"the\" one"}),
            ],
            total: 2,
        })
    }
    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
        Ok(None)
    }
    async fn create(&self, _t: &str, _d: Map<String, Value>) -> Result<CreateOutcome, StorageError> {
        Ok(CreateOutcome::default())
    }
    async fn update(&self, _t: &str, _pk: &str, _i: &str, _d: Map<String, Value>) -> Result<u64, StorageError> {
        Ok(1)
    }
    async fn delete(&self, _t: &str, _pk: &str, _i: &str, _s: bool) -> Result<u64, StorageError> {
        Ok(1)
    }
    async fn health(&self) -> bool {
        true
    }
}

fn toggle(_ctx: ReqCtx, id: String, _body: Value) -> ActionFuture {
    Box::pin(async move { ApiResponse::ok(json!({ "toggled": id })) })
}

#[derive(Clone)]
struct Widget;

#[async_trait]
impl Resource for Widget {
    fn resource_name(&self) -> &'static str {
        "Widgets"
    }
    fn base_path(&self) -> &'static str {
        "widgets"
    }
    fn table_name(&self) -> &'static str {
        "widgets"
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    fn custom_actions(&self) -> Vec<CustomAction> {
        vec![CustomAction::labeled("toggle", "Toggle", toggle)]
    }
}

fn body_str(resp: &ApiResponse) -> String {
    match &resp.body {
        ApiBody::Bytes { data, .. } => String::from_utf8_lossy(data).to_string(),
        ApiBody::Json(v) => v.to_string(),
        ApiBody::Empty => String::new(),
    }
}

fn content_type(resp: &ApiResponse) -> String {
    match &resp.body {
        ApiBody::Bytes { content_type, .. } => content_type.clone(),
        _ => String::new(),
    }
}

#[tokio::test]
async fn actions_and_export() {
    set_storage(Box::new(Mock));
    let w = Widget;
    let ctx = ReqCtx::new().with_mount("/adminx");

    // --- custom action dispatch ---
    let ran = w.run_action(&ctx, "toggle", "42".into(), json!({})).await;
    assert_eq!(ran.status, 200);
    assert_eq!(body_str(&ran), json!({"toggled": "42"}).to_string());

    // unknown action -> 404
    let missing = w.run_action(&ctx, "nope", "1".into(), json!({})).await;
    assert_eq!(missing.status, 404);

    // --- CSV export (direct) ---
    let csv = w.export(&ctx, "csv").await;
    assert_eq!(csv.status, 200);
    assert_eq!(content_type(&csv), "text/csv");
    let text = body_str(&csv);
    assert!(text.starts_with("id,name"), "csv has header row");
    assert!(text.contains("\"Grace, \"\"the\"\" one\""), "csv escapes quotes/commas");
    assert!(csv
        .headers
        .iter()
        .any(|(k, v)| k == "Content-Disposition" && v.contains("widgets.csv")));

    // --- JSON export (direct) ---
    let j = w.export(&ctx, "json").await;
    assert_eq!(content_type(&j), "application/json");
    assert!(body_str(&j).contains("Ada"));

    // --- export via the list page download param ---
    let via_list = w
        .list_page(&ReqCtx::new().with_mount("/adminx").with_query("download=csv"))
        .await;
    assert_eq!(content_type(&via_list), "text/csv");

    // unsupported format -> 400
    let bad = w.export(&ctx, "xml").await;
    assert_eq!(bad.status, 400);
}
