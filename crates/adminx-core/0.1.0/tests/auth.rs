// Integration test for the auth layer: JWT round-trip, RBAC enforcement on both
// the JSON API (401) and the HTML UI (redirect to login), and the login handler.
// Runs against a mock storage backend — no database.

use adminx_core::auth::{
    self, build_ctx, configure, handle_login, is_authorized, issue_token, verify_password,
    verify_token, AuthConfig,
};
use adminx_core::prelude::*;
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use async_trait::async_trait;
use serde_json::{json, Map, Value};

struct AuthMock;

#[async_trait]
impl Storage for AuthMock {
    async fn list(&self, _t: &str, _o: &QueryOptions) -> Result<ListPage, StorageError> {
        Ok(ListPage {
            rows: vec![json!({"id": 1, "name": "Row"})],
            total: 1,
        })
    }
    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
        Ok(None)
    }
    async fn find_one_by(
        &self,
        _table: &str,
        column: &str,
        value: &str,
    ) -> Result<Option<Value>, StorageError> {
        if column == "email" && value == "admin@x.io" {
            let hash = auth::hash_password("secret").unwrap();
            Ok(Some(json!({
                "id": 1,
                "email": "admin@x.io",
                "encrypted_password": hash,
                "role": "admin",
            })))
        } else {
            Ok(None)
        }
    }
    async fn create(&self, _t: &str, _d: Map<String, Value>) -> Result<CreateOutcome, StorageError> {
        Ok(CreateOutcome::default())
    }
    async fn update(&self, _t: &str, _pk: &str, _id: &str, _d: Map<String, Value>) -> Result<u64, StorageError> {
        Ok(1)
    }
    async fn delete(&self, _t: &str, _pk: &str, _id: &str, _s: bool) -> Result<u64, StorageError> {
        Ok(1)
    }
    async fn health(&self) -> bool {
        true
    }
}

#[derive(Clone)]
struct Widgets;

#[async_trait]
impl Resource for Widgets {
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
    // default allowed_roles() == ["admin"]
}

fn has_auth_cookie(resp: &ApiResponse) -> bool {
    resp.headers
        .iter()
        .any(|(k, v)| k == "Set-Cookie" && v.contains("adminx_token="))
}

#[tokio::test]
async fn auth_enforced_across_api_and_ui() {
    configure(AuthConfig {
        jwt_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        admin_table: "adminx_users".into(),
        secure_cookie: false,
    });
    set_storage(Box::new(AuthMock));
    register_resource(Box::new(Widgets));

    // --- JWT round-trip ---
    let token = issue_token("1", "admin@x.io", "admin", auth::MFA_OK).expect("issue");
    let claims = verify_token(&token).expect("verify");
    assert_eq!(claims.role, "admin");
    assert!(verify_token("not-a-real-token").is_none());

    // --- password hashing ---
    let hash = auth::hash_password("secret").unwrap();
    assert!(verify_password("secret", &hash));
    assert!(!verify_password("wrong", &hash));

    // --- is_authorized ---
    let admin_roles = ["admin".to_string()];
    let anon = ReqCtx::new().with_mount("/adminx");
    assert!(!is_authorized(&anon, &admin_roles), "anon denied");

    let admin_ctx = build_ctx("/adminx", "", Some(&token));
    assert!(is_authorized(&admin_ctx, &admin_roles), "admin allowed");

    let editor_token = issue_token("2", "ed@x.io", "editor", auth::MFA_OK).unwrap();
    let editor_ctx = build_ctx("/adminx", "", Some(&editor_token));
    assert!(
        !is_authorized(&editor_ctx, &admin_roles),
        "editor denied on admin-only"
    );

    let res = Widgets;

    // --- API guard: 401 for anon, 200 for admin ---
    assert_eq!(res.list(&anon).await.status, 401);
    assert_eq!(res.list(&admin_ctx).await.status, 200);

    // --- UI guard: redirect to login for anon, 200 for admin ---
    let ui_anon = res.list_page(&anon).await;
    assert_eq!(ui_anon.status, 303);
    assert!(ui_anon
        .headers
        .iter()
        .any(|(k, v)| k == "Location" && v == "/adminx/login"));
    assert_eq!(res.list_page(&admin_ctx).await.status, 200);

    // --- login handler: good creds set cookie + redirect; bad creds re-render ---
    let good = handle_login(&anon, "admin@x.io", "secret").await;
    assert_eq!(good.status, 303);
    assert!(has_auth_cookie(&good), "login sets auth cookie");

    let bad = handle_login(&anon, "admin@x.io", "nope").await;
    assert_eq!(bad.status, 200, "bad login re-renders the form");
    assert!(!has_auth_cookie(&bad));
}
