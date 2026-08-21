// Proves the authorization seam threads the *correct* `Action` at every one of
// the Resource methods. A recording `Authorizer` captures the (resource, action)
// it is asked about; each method is invoked once and the recorded action is
// checked against the intended mapping. This is what pins, e.g., that a custom
// action authorizes as `Custom(name)` and the edit form as `Update` (not `Read`).

use adminx_core::authz::{set_authorizer, Action, Authorizer};
use adminx_core::prelude::*;
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use async_trait::async_trait;
use serde_json::{json, Map, Value};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Records every `(resource, action)` `can` is asked, and always allows so the
/// method body runs on past the check.
#[derive(Clone)]
struct Recorder {
    log: Arc<Mutex<Vec<(String, String)>>>,
}

impl Authorizer for Recorder {
    fn can(&self, _roles: &[String], resource: &str, action: &Action<'_>) -> bool {
        self.log
            .lock()
            .unwrap()
            .push((resource.to_string(), action.as_str().to_string()));
        true
    }
}

struct Mock;

#[async_trait]
impl Storage for Mock {
    async fn list(&self, _t: &str, _o: &QueryOptions) -> Result<ListPage, StorageError> {
        Ok(ListPage { rows: vec![json!({"id": 1, "name": "row"})], total: 1 })
    }
    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
        Ok(Some(json!({"id": 1, "name": "row"})))
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
    fn permit_keys(&self) -> Vec<&'static str> {
        vec!["name"]
    }
    fn custom_actions(&self) -> Vec<CustomAction> {
        vec![CustomAction::labeled("toggle", "Toggle", toggle)]
    }
}

#[tokio::test]
async fn each_method_authorizes_with_the_right_action() {
    let log = Arc::new(Mutex::new(Vec::new()));
    set_authorizer(Box::new(Recorder { log: log.clone() }));
    set_storage(Box::new(Mock));
    configure_auth(AuthConfig {
        jwt_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        admin_table: "adminx_users".into(),
        secure_cookie: false,
    });

    // An authenticated, MFA-cleared admin — so the check reaches the authorizer
    // rather than short-circuiting on unconfigured/MFA-pending.
    let ctx = ReqCtx::new().with_mount("/adminx").with_claims(Claims {
        sub: "1".into(),
        email: "a@x.io".into(),
        role: "admin".into(),
        roles: vec!["admin".into()],
        mfa: "ok".into(),
    });

    let w = Widget;
    let mut form = HashMap::new();
    form.insert("name".to_string(), "x".to_string());

    // Drain the log and assert every recorded action for the just-run method is
    // exactly `expected` on resource "widgets".
    let check = |expected: &str| {
        let mut l = log.lock().unwrap();
        let seen = l.clone();
        l.clear();
        assert!(!seen.is_empty(), "expected an authorize call for {expected}");
        for (resource, action) in &seen {
            assert_eq!(resource, "widgets", "resource key should be base_path()");
            assert_eq!(action, expected, "wrong action threaded for this method");
        }
    };

    w.list(&ctx).await;
    check("list");
    w.get(&ctx, "1").await;
    check("read");
    w.create(&ctx, json!({"name": "a"})).await;
    check("create");
    w.update(&ctx, "1", json!({"name": "a"})).await;
    check("update");
    w.delete(&ctx, "1").await;
    check("delete");

    w.list_page(&ctx).await;
    check("list");
    w.new_page(&ctx).await;
    check("create");
    w.edit_page(&ctx, "1").await;
    check("update");
    w.view_page(&ctx, "1").await;
    check("read");

    // The *_form handlers authorize, then hit the CSRF guard (no token here) —
    // which is after the authorize call we're checking.
    w.create_form(&ctx, form.clone()).await;
    check("create");
    w.update_form(&ctx, "1", form.clone()).await;
    check("update");
    w.delete_form(&ctx, "1", None).await;
    check("delete");

    w.run_action(&ctx, "toggle", "1".into(), json!({}), None).await;
    check("toggle"); // Action::Custom("toggle") serializes to its name

    w.export(&ctx, "csv").await;
    check("export");
}
