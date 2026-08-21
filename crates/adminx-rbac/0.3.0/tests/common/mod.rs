// Shared fixtures for the RBAC integration tests. Each test binary registers
// process-global singletons (storage, auth config, authorizer), so the two
// scenarios live in separate test files — hence this shared module rather than
// one file with two tests.
#![allow(dead_code)]

use adminx_core::prelude::*;
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::sync::Mutex;

/// Mock backed by an in-memory permissions table: `rbac::init` seeds into it and
/// `reload` reads back what it wrote, exercising the real path.
pub struct Mock {
    pub permissions: Mutex<Vec<Value>>,
}

impl Mock {
    pub fn with(rows: Vec<Value>) -> Self {
        Self { permissions: Mutex::new(rows) }
    }
}

#[async_trait]
impl Storage for Mock {
    async fn list(&self, table: &str, _o: &QueryOptions) -> Result<ListPage, StorageError> {
        if table == "adminx_permissions" {
            let rows = self.permissions.lock().unwrap().clone();
            let total = rows.len() as u64;
            return Ok(ListPage { rows, total });
        }
        Ok(ListPage { rows: vec![json!({"id": 1})], total: 1 })
    }
    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
        Ok(Some(json!({"id": 1})))
    }
    async fn create(&self, table: &str, data: Map<String, Value>) -> Result<CreateOutcome, StorageError> {
        if table == "adminx_permissions" {
            self.permissions.lock().unwrap().push(Value::Object(data));
        }
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

fn publish(_ctx: ReqCtx, id: String, _body: Value) -> ActionFuture {
    Box::pin(async move { ApiResponse::ok(json!({ "published": id })) })
}

/// Resource `posts` with a custom action `publish`. Its `allowed_roles` is
/// deliberately restrictive so any editor/viewer access proves the RBAC
/// authorizer — not the fallback role list — is deciding.
#[derive(Clone)]
pub struct Posts;

#[async_trait]
impl Resource for Posts {
    fn resource_name(&self) -> &'static str {
        "Posts"
    }
    fn base_path(&self) -> &'static str {
        "posts"
    }
    fn table_name(&self) -> &'static str {
        "posts"
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    fn permit_keys(&self) -> Vec<&'static str> {
        vec!["title"]
    }
    fn allowed_roles(&self) -> Vec<String> {
        vec!["nobody".into()]
    }
    fn custom_actions(&self) -> Vec<CustomAction> {
        vec![CustomAction::labeled("publish", "Publish", publish)]
    }
}

/// A second resource, to test resource scoping and the `"*"` wildcard.
#[derive(Clone)]
pub struct Reports;

#[async_trait]
impl Resource for Reports {
    fn resource_name(&self) -> &'static str {
        "Reports"
    }
    fn base_path(&self) -> &'static str {
        "reports"
    }
    fn table_name(&self) -> &'static str {
        "reports"
    }
    fn clone_box(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }
    fn allowed_roles(&self) -> Vec<String> {
        vec!["nobody".into()]
    }
}

pub fn ctx_for(role: &str) -> ReqCtx {
    ReqCtx::new().with_mount("/adminx").with_claims(Claims {
        sub: "1".into(),
        email: "u@x.io".into(),
        role: role.into(),
        roles: vec![role.into()],
        mfa: "ok".into(),
    })
}

pub fn configure_test_auth() {
    configure_auth(AuthConfig {
        jwt_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        admin_table: "adminx_users".into(),
        secure_cookie: false,
    });
}
