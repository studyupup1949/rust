// The default: a broken auditor is logged and stepped over, so a sick audit
// table cannot take the panel down.
//
// Its own binary because the auditor is a set-once global.

use adminx_core::audit::{set_auditor, AuditEntry, Auditor};
use adminx_core::prelude::*;
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use async_trait::async_trait;
use serde_json::{json, Map, Value};

/// Always fails, and leaves `strict()` at its default `false`.
struct Broken;

#[async_trait]
impl Auditor for Broken {
    async fn record(&self, _entry: AuditEntry) -> Result<(), StorageError> {
        Err(StorageError::Backend("audit table is gone".into()))
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
async fn a_failed_audit_write_does_not_fail_the_request() {
    set_auditor(Box::new(Broken));
    set_storage(Box::new(Mock));
    configure_auth(AuthConfig {
        jwt_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        admin_table: "adminx_users".into(),
        secure_cookie: false,
    });

    let ctx = ReqCtx::new().with_mount("/adminx").with_claims(Claims {
        sub: "1".into(),
        email: "a@x.io".into(),
        role: "admin".into(),
        roles: vec!["admin".into()],
        mfa: "ok".into(),
    });

    let w = Widget;

    assert_eq!(w.create(&ctx, json!({"name": "a"})).await.status, 201);
    assert_eq!(w.update(&ctx, "1", json!({"name": "b"})).await.status, 200);
    assert_eq!(w.delete(&ctx, "1").await.status, 200);
}
