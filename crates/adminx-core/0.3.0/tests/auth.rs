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
        // `throttle@x.io` exists so the rate-limit test can key off an account no
        // other test touches — the counter map is process-global and these tests
        // run in parallel.
        if column == "email" && matches!(value, "admin@x.io" | "throttle@x.io" | "clears@x.io") {
            let hash = auth::hash_password("secret").unwrap();
            Ok(Some(json!({
                "id": 1,
                "email": value,
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

    let admin_ctx = build_ctx("/adminx", "", Some(&token), None);
    assert!(is_authorized(&admin_ctx, &admin_roles), "admin allowed");

    let editor_token = issue_token("2", "ed@x.io", "editor", auth::MFA_OK).unwrap();
    let editor_ctx = build_ctx("/adminx", "", Some(&editor_token), None);
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
    // A real login carries a matching CSRF cookie + form field; `csrf_ctx` is the
    // browser that fetched the form, `CSRF_TOK` the hidden field it echoes back.
    const CSRF_TOK: &str = "csrf-token-value";
    let csrf_ctx = build_ctx("/adminx", "", None, Some(CSRF_TOK));

    let good = handle_login(&csrf_ctx, "admin@x.io", "secret", Some(CSRF_TOK)).await;
    assert_eq!(good.status, 303);
    assert!(has_auth_cookie(&good), "login sets auth cookie");

    let bad = handle_login(&csrf_ctx, "admin@x.io", "nope", Some(CSRF_TOK)).await;
    assert_eq!(bad.status, 200, "bad login re-renders the form");
    assert!(!has_auth_cookie(&bad));
}

/// The login post is the one endpoint `SameSite=Strict` can't protect (it needs
/// no prior cookie), so the CSRF pair is all that stands between a forged post
/// and logging a victim into the attacker's account.
#[tokio::test]
async fn login_requires_a_matching_csrf_pair() {
    // `configure`/`set_storage` are process-global `OnceCell`s and the other test
    // may have won the race; either way the state it sets is what we need.
    configure(AuthConfig {
        jwt_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        admin_table: "adminx_users".into(),
        secure_cookie: false,
    });
    set_storage(Box::new(AuthMock));

    const TOK: &str = "csrf-token-value";
    let with_cookie = build_ctx("/adminx", "", None, Some(TOK));
    let no_cookie = build_ctx("/adminx", "", None, None);

    // Forged cross-site post: `SameSite=Strict` withholds the CSRF cookie, so
    // whatever the attacker puts in the field has nothing to match.
    let forged = handle_login(&no_cookie, "admin@x.io", "secret", Some(TOK)).await;
    assert_eq!(forged.status, 403, "no cookie -> rejected");
    assert!(!has_auth_cookie(&forged), "forged post must not log anyone in");

    // Bare form post with no hidden field at all.
    let bare = handle_login(&with_cookie, "admin@x.io", "secret", None).await;
    assert_eq!(bare.status, 403, "no form field -> rejected");
    assert!(!has_auth_cookie(&bare));

    // Cookie and field both present but not equal.
    let mismatch = handle_login(&with_cookie, "admin@x.io", "secret", Some("other")).await;
    assert_eq!(mismatch.status, 403, "mismatched pair -> rejected");
    assert!(!has_auth_cookie(&mismatch));

    // Neither present: must not fall through to an "empty == empty" match.
    let neither = handle_login(&no_cookie, "admin@x.io", "secret", None).await;
    assert_eq!(neither.status, 403, "absent pair -> rejected");
    assert!(!has_auth_cookie(&neither));

    // And the happy path still works, proving the above fail on CSRF alone and
    // not because the credentials were wrong.
    let ok = handle_login(&with_cookie, "admin@x.io", "secret", Some(TOK)).await;
    assert_eq!(ok.status, 303);
    assert!(has_auth_cookie(&ok));
}

/// Password guessing must run out of road. Keyed to `throttle@x.io` so the
/// process-global counters aren't disturbed by the other tests in this binary.
#[tokio::test]
async fn login_throttles_repeated_password_guessing() {
    configure(AuthConfig {
        jwt_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        admin_table: "adminx_users".into(),
        secure_cookie: false,
    });
    set_storage(Box::new(AuthMock));

    const TOK: &str = "csrf-token-value";
    let ctx = build_ctx("/adminx", "", None, Some(TOK));
    let guess = |pw: &'static str| handle_login(&ctx, "throttle@x.io", pw, Some(TOK));

    // Burn exactly the default budget (10). Each is a plain rejection, not a
    // throttle — the limit must not bite early.
    for i in 1..=10 {
        let resp = guess("wrong").await;
        assert_eq!(resp.status, 200, "attempt {i} should re-render, not throttle");
    }

    // Budget spent: further guessing is refused outright.
    let over = guess("wrong").await;
    assert_eq!(over.status, 429, "attempt 11 must be throttled");

    // The point of the throttle: even the *correct* password is refused while
    // the window is open, so an attacker who eventually guesses right is still
    // shut out. This is also what proves the limiter gates the lookup rather
    // than merely counting.
    let correct = guess("secret").await;
    assert_eq!(correct.status, 429, "a throttled account is closed to everyone");
    assert!(!has_auth_cookie(&correct), "throttled login must not issue a token");
}

/// A user who fumbles and then gets it right shouldn't stay penalised.
#[tokio::test]
async fn a_successful_login_clears_the_count() {
    configure(AuthConfig {
        jwt_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        admin_table: "adminx_users".into(),
        secure_cookie: false,
    });
    set_storage(Box::new(AuthMock));

    const TOK: &str = "csrf-token-value";
    let ctx = build_ctx("/adminx", "", None, Some(TOK));
    let attempt = |pw: &'static str| handle_login(&ctx, "clears@x.io", pw, Some(TOK));

    // Fumble 9 times — one short of the default budget of 10.
    for _ in 0..9 {
        assert_eq!(attempt("wrong").await.status, 200);
    }

    // Then get it right. This is the reset, driven through the real handler
    // rather than by poking the limiter directly.
    let good = attempt("secret").await;
    assert_eq!(good.status, 303, "the 10th attempt, correct, must succeed");
    assert!(has_auth_cookie(&good));

    // 9 more failures must again stay under the limit. Without the reset the
    // running total would be 18 and this would throttle at the second one.
    for i in 1..=9 {
        assert_eq!(
            attempt("wrong").await.status,
            200,
            "failure {i} after a success should start from a clean count"
        );
    }
}

/// The rendered login form must carry a token that matches the cookie it sets,
/// otherwise every real login would 403.
#[test]
fn login_page_issues_a_usable_token() {
    let fresh = auth::login_page(&ReqCtx::new().with_mount("/adminx"), None);
    let cookie = fresh
        .headers
        .iter()
        .find(|(k, _)| k == "Set-Cookie")
        .map(|(_, v)| v.clone())
        .expect("a visitor with no CSRF cookie gets one minted");
    assert!(cookie.contains("adminx_csrf="));
    assert!(cookie.contains("HttpOnly"));
    assert!(cookie.contains("SameSite=Strict"));

    let token = cookie
        .strip_prefix("adminx_csrf=")
        .and_then(|s| s.split(';').next())
        .expect("token in cookie");
    let html = match &fresh.body {
        ApiBody::Bytes { data, .. } => String::from_utf8_lossy(data).to_string(),
        other => panic!("expected an HTML body, got {other:?}"),
    };
    assert!(
        html.contains(&format!(r#"name="_csrf" value="{token}""#)),
        "the form field must mirror the cookie the same response sets"
    );

    // A visitor that already has a token keeps it, so a second tab doesn't
    // invalidate the first tab's form.
    let repeat = auth::login_page(&ReqCtx::new().with_mount("/adminx").with_csrf("existing"), None);
    assert!(
        !repeat.headers.iter().any(|(k, _)| k == "Set-Cookie"),
        "an existing CSRF cookie must be reused, not replaced"
    );
}
