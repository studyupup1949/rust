// An overriding resource must still be audited.
//
// Its own binary: the auditor is a set-once global, so a second `set_auditor` in
// a process that already has one is ignored and the entries would land in the
// other test's recorder.

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

struct Mock;

#[async_trait]
impl Storage for Mock {
    async fn list(&self, _t: &str, _o: &QueryOptions) -> Result<ListPage, StorageError> {
        Ok(ListPage { rows: vec![], total: 0 })
    }
    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
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

/// A resource that *extends* the default CRUD — the shape `adminx-rbac`'s
/// `PermissionResource` uses to reload its cache after a write.
#[derive(Clone)]
struct Extending;

#[async_trait]
impl Resource for Extending {
    fn resource_name(&self) -> &'static str {
        "Extending"
    }
    fn base_path(&self) -> &'static str {
        "extending"
    }
    fn table_name(&self) -> &'static str {
        "extending"
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    fn permit_keys(&self) -> Vec<&'static str> {
        vec!["name"]
    }

    async fn create(&self, ctx: &ReqCtx, body: Value) -> ApiResponse {
        adminx_core::crud::create(self, ctx, body).await
    }
    async fn update(&self, ctx: &ReqCtx, id: &str, body: Value) -> ApiResponse {
        adminx_core::crud::update(self, ctx, id, body).await
    }
    async fn delete(&self, ctx: &ReqCtx, id: &str) -> ApiResponse {
        adminx_core::crud::delete(self, ctx, id).await
    }
}

/// The regression this guards: an overriding impl cannot call the trait's own
/// default, so before `crud` existed such a resource copied the body — and
/// silently recorded nothing once auditing was added to the default. Delegating
/// to the shared body keeps the invariant.
#[tokio::test]
async fn an_overriding_resource_is_still_audited() {
    let entries = Arc::new(Mutex::new(Vec::new()));
    set_auditor(Box::new(Recorder { entries: entries.clone() }));
    set_storage(Box::new(Mock));

    // Auth deliberately left unconfigured here: this binary's other test owns
    // `configure_auth`, and with auth off the access check allows through — the
    // recording path is what's under test.
    let ctx = ReqCtx::new().with_mount("/adminx");
    let e = Extending;

    e.create(&ctx, json!({"name": "a"})).await;
    e.update(&ctx, "1", json!({"name": "changed"})).await;
    e.delete(&ctx, "1").await;

    let seen = entries.lock().unwrap();
    let events: Vec<&str> = seen.iter().map(|e| e.event.as_str()).collect();
    assert_eq!(
        events,
        vec!["create", "update", "delete"],
        "an overriding resource that delegates must record all three"
    );
    assert!(
        seen.iter().all(|e| e.item_type == "extending"),
        "entries must carry the overriding resource's own base_path"
    );
}

