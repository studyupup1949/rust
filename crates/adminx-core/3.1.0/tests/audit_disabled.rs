// With no auditor registered, the history route still answers — it renders an
// explanatory page rather than 404 — and the record pages issue no extra query.
//
// Its own binary precisely *because* it must leave the audit globals unset.

use adminx_core::prelude::*;
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

/// Counts `get` calls, so the "no extra read when auditing is off" claim is
/// measured rather than asserted.
struct Counting {
    gets: Arc<AtomicUsize>,
}

#[async_trait]
impl Storage for Counting {
    async fn list(&self, _t: &str, _o: &QueryOptions) -> Result<ListPage, StorageError> {
        Ok(ListPage { rows: vec![], total: 0 })
    }
    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
        self.gets.fetch_add(1, Ordering::SeqCst);
        Ok(Some(json!({"id": 1, "name": "row"})))
    }
    async fn create(&self, _t: &str, _d: Map<String, Value>) -> Result<CreateOutcome, StorageError> {
        Ok(CreateOutcome { last_insert_id: Some("1".into()) })
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
    fn permit_keys(&self) -> Vec<&'static str> {
        vec!["name"]
    }
}

#[tokio::test]
async fn an_unaudited_app_still_serves_history_and_pays_nothing() {
    let gets = Arc::new(AtomicUsize::new(0));
    set_storage(Box::new(Counting { gets: gets.clone() }));

    let ctx = ReqCtx::new().with_mount("/adminx");
    let w = Widget;

    // The page exists and explains itself rather than 404ing.
    let resp = w.history_page(&ctx, "1").await;
    assert_eq!(resp.status, 200);
    // HTML is carried as bytes with a content type; there is no Html variant.
    let body = match &resp.body {
        adminx_core::ApiBody::Bytes { content_type, data } => {
            assert!(content_type.starts_with("text/html"), "got {content_type}");
            String::from_utf8_lossy(data).to_string()
        }
        other => panic!("expected an html body, got {other:?}"),
    };
    assert!(
        body.contains("Audit logging is not enabled"),
        "history page should say why it is empty"
    );

    // The cost claim: update and delete issue no before-state read when there
    // is no auditor to hand it to.
    gets.store(0, Ordering::SeqCst);
    w.update(&ctx, "1", json!({"name": "a"})).await;
    w.delete(&ctx, "1").await;
    assert_eq!(
        gets.load(Ordering::SeqCst),
        0,
        "an unaudited app must not issue the before-state read"
    );
}
