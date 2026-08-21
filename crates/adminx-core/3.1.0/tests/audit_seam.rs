// Proves the audit seam fires from the default CRUD methods with the right
// event, target, actor and diff. A recording `Auditor` captures what it is
// handed; a mock `Storage` returns a known "before" row so the update and delete
// diffs can be checked against it.
//
// The seam's globals are set-once per process, so the strict-mode behaviour
// lives in its own test binary (`audit_strict.rs`).

use adminx_core::audit::{set_auditor, AuditEntry, Auditor};
use adminx_core::prelude::*;
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct Recorder {
    entries: Arc<Mutex<Vec<AuditEntry>>>,
}

#[async_trait]
impl Auditor for Recorder {
    async fn record(&self, entry: AuditEntry) -> Result<(), StorageError> {
        self.entries.lock().unwrap().push(entry);
        Ok(())
    }
}

/// The row every `get` returns, so update/delete have a known before-state.
fn stored_row() -> Value {
    json!({"id": 1, "name": "row", "views": 5})
}

struct Mock;

#[async_trait]
impl Storage for Mock {
    async fn list(&self, _t: &str, _o: &QueryOptions) -> Result<ListPage, StorageError> {
        Ok(ListPage { rows: vec![stored_row()], total: 1 })
    }
    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
        Ok(Some(stored_row()))
    }
    async fn create(&self, _t: &str, _d: Map<String, Value>) -> Result<CreateOutcome, StorageError> {
        Ok(CreateOutcome { last_insert_id: Some("77".into()) })
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
        vec!["name", "views"]
    }
}

#[tokio::test]
async fn crud_records_who_changed_what() {
    let entries = Arc::new(Mutex::new(Vec::new()));
    set_auditor(Box::new(Recorder { entries: entries.clone() }));
    set_storage(Box::new(Mock));
    configure_auth(AuthConfig {
        jwt_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        admin_table: "adminx_users".into(),
        secure_cookie: false,
    });

    let ctx = ReqCtx::new().with_mount("/adminx").with_claims(Claims {
        sub: "9".into(),
        email: "admin@example.com".into(),
        role: "admin".into(),
        roles: vec!["admin".into()],
        mfa: "ok".into(),
    });

    let w = Widget;
    let drain = || -> Vec<AuditEntry> {
        let mut e = entries.lock().unwrap();
        let seen = e.clone();
        e.clear();
        seen
    };

    // --- create: id comes from the insert, diff has null on the left ---------
    w.create(&ctx, json!({"name": "fresh"})).await;
    let seen = drain();
    assert_eq!(seen.len(), 1, "create should record exactly one entry");
    let e = &seen[0];
    assert_eq!(e.item_type, "widgets", "item_type is the resource base_path");
    assert_eq!(e.item_id, "77", "item_id comes from last_insert_id");
    assert_eq!(e.event.as_str(), "create");
    assert_eq!(e.whodunnit.as_deref(), Some("9"));
    assert_eq!(e.whodunnit_email.as_deref(), Some("admin@example.com"));
    assert_eq!(e.changes["name"], json!([Value::Null, "fresh"]));

    // --- update: only the column that actually moved -------------------------
    w.update(&ctx, "1", json!({"name": "changed", "views": 5})).await;
    let seen = drain();
    assert_eq!(seen.len(), 1);
    let e = &seen[0];
    assert_eq!(e.item_id, "1");
    assert_eq!(e.event.as_str(), "update");
    assert_eq!(e.changes["name"], json!(["row", "changed"]));
    assert_eq!(
        e.changes.len(),
        1,
        "`views` was submitted unchanged and must not appear: {:?}",
        e.changes
    );

    // --- a save that changes nothing is not history --------------------------
    w.update(&ctx, "1", json!({"name": "row", "views": 5})).await;
    assert!(drain().is_empty(), "a no-op save should record nothing");

    // --- delete: the whole record, as it last existed ------------------------
    w.delete(&ctx, "1").await;
    let seen = drain();
    assert_eq!(seen.len(), 1);
    let e = &seen[0];
    assert_eq!(e.event.as_str(), "delete");
    assert_eq!(e.changes["name"], json!(["row", Value::Null]));
    assert_eq!(
        e.changes["views"],
        json!([5, Value::Null]),
        "delete captures every stored column, not just the writable ones"
    );

    // --- reads never write to the log ----------------------------------------
    w.list(&ctx).await;
    w.get(&ctx, "1").await;
    w.view_page(&ctx, "1").await;
    assert!(drain().is_empty(), "reads must not produce audit entries");
}

// `AuditEntry` needs Clone for the recorder to hand copies back out; assert it
// here so the derive can't be dropped without a test failing.
fn _assert_clone(e: AuditEntry) -> AuditEntry {
    e.clone()
}
