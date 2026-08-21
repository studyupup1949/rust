// End-to-end CSRF check through the real Axum router: a genuine browser flow
// (GET the form, POST it back with the cookie) must log in, and a forged
// cross-site POST must not.
//
// The core's own tests call `handle_login` directly, which can't catch a broken
// adapter — that the `_csrf` field is actually deserialized off the form body,
// and the `adminx_csrf` cookie actually parsed out of the `Cookie` header. Only
// driving real requests through the router covers that seam.

use adminx_core::actions::{ActionFuture, CustomAction};
use adminx_core::prelude::*;
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use async_trait::async_trait;
use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use serde_json::{json, Map, Value};
use tower::util::ServiceExt;

struct AuthMock;

#[async_trait]
impl Storage for AuthMock {
    async fn list(&self, _t: &str, _o: &QueryOptions) -> Result<ListPage, StorageError> {
        Ok(ListPage { rows: vec![], total: 0 })
    }
    async fn get(&self, _t: &str, _pk: &str, _id: &str) -> Result<Option<Value>, StorageError> {
        Ok(None)
    }
    async fn find_one_by(&self, _t: &str, column: &str, value: &str) -> Result<Option<Value>, StorageError> {
        if column == "email" && value == "admin@x.io" {
            return Ok(Some(json!({
                "id": 1,
                "email": "admin@x.io",
                "encrypted_password": adminx_core::auth::hash_password("secret").unwrap(),
                "role": "admin",
            })));
        }
        Ok(None)
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

/// A resource with a writable field and one custom action, so the create,
/// delete, and action form posts all have something real to do.
#[derive(Clone)]
struct Widget;

fn toggle(_ctx: ReqCtx, id: String, _body: Value) -> ActionFuture {
    Box::pin(async move { ApiResponse::ok(json!({ "toggled": id })) })
}

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

fn app() -> Router {
    Router::new().nest("/adminx", adminx_axum::router())
}

fn setup() {
    configure_auth(AuthConfig {
        jwt_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        admin_table: "adminx_users".into(),
        secure_cookie: false,
    });
    set_storage(Box::new(AuthMock));
}

/// All `Set-Cookie` values on a response.
fn cookies(resp: &axum::response::Response) -> Vec<String> {
    resp.headers()
        .get_all("set-cookie")
        .iter()
        .filter_map(|v| v.to_str().ok().map(str::to_string))
        .collect()
}

fn cookie_named<'a>(cookies: &'a [String], name: &str) -> Option<&'a str> {
    cookies
        .iter()
        .find_map(|c| c.strip_prefix(&format!("{name}=")))
        .and_then(|v| v.split(';').next())
}

fn post_login(cookie_header: Option<&str>, body: String) -> Request<Body> {
    let mut req = Request::builder()
        .method("POST")
        .uri("/adminx/login")
        .header("content-type", "application/x-www-form-urlencoded");
    if let Some(c) = cookie_header {
        req = req.header("cookie", c);
    }
    req.body(Body::from(body)).unwrap()
}

#[tokio::test]
async fn browser_login_succeeds_and_forged_post_is_rejected() {
    setup();

    // --- 1. A browser fetches the login form. ---
    let form = app()
        .oneshot(Request::builder().uri("/adminx/login").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(form.status(), StatusCode::OK);

    let set = cookies(&form);
    let token = cookie_named(&set, "adminx_csrf")
        .expect("GET /login must mint a CSRF cookie")
        .to_string();
    let html = {
        let bytes = axum::body::to_bytes(form.into_body(), usize::MAX).await.unwrap();
        String::from_utf8(bytes.to_vec()).unwrap()
    };
    assert!(
        html.contains(&format!(r#"name="_csrf" value="{token}""#)),
        "the served form must carry the token matching the cookie it set"
    );

    let jar = format!("adminx_csrf={token}");
    let creds = format!("email=admin%40x.io&password=secret&_csrf={token}");

    // --- 2. The browser posts it back: cookie + field agree -> logged in. ---
    let ok = app().oneshot(post_login(Some(&jar), creds.clone())).await.unwrap();
    assert_eq!(ok.status(), StatusCode::SEE_OTHER, "genuine login must succeed");
    assert!(
        cookie_named(&cookies(&ok), "adminx_token").is_some(),
        "genuine login must set the auth cookie"
    );

    // --- 3. Forged cross-site post: SameSite=Strict withholds the CSRF cookie,
    //        so the attacker's field has nothing to match. ---
    let forged = app().oneshot(post_login(None, creds)).await.unwrap();
    assert_eq!(forged.status(), StatusCode::FORBIDDEN, "no cookie -> rejected");
    assert!(
        cookie_named(&cookies(&forged), "adminx_token").is_none(),
        "a forged post must never log anyone in"
    );

    // --- 4. Bare post with valid creds and no token at all. ---
    let bare = app()
        .oneshot(post_login(Some(&jar), "email=admin%40x.io&password=secret".into()))
        .await
        .unwrap();
    assert_eq!(bare.status(), StatusCode::FORBIDDEN, "no _csrf field -> rejected");
    assert!(cookie_named(&cookies(&bare), "adminx_token").is_none());

    // --- 5. Cookie and field present but mismatched. ---
    let mismatch = app()
        .oneshot(post_login(
            Some(&jar),
            "email=admin%40x.io&password=secret&_csrf=wrong".into(),
        ))
        .await
        .unwrap();
    assert_eq!(mismatch.status(), StatusCode::FORBIDDEN, "mismatch -> rejected");
    assert!(cookie_named(&cookies(&mismatch), "adminx_token").is_none());
}

/// Log in and return the `adminx_token` cookie value, so resource-form tests can
/// get past the auth gate and reach the CSRF check.
async fn login_as_admin() -> String {
    let form = app()
        .oneshot(Request::builder().uri("/adminx/login").body(Body::empty()).unwrap())
        .await
        .unwrap();
    let token = cookie_named(&cookies(&form), "adminx_csrf").unwrap().to_string();
    let jar = format!("adminx_csrf={token}");
    let creds = format!("email=admin%40x.io&password=secret&_csrf={token}");
    let ok = app().oneshot(post_login(Some(&jar), creds)).await.unwrap();
    cookie_named(&cookies(&ok), "adminx_token")
        .expect("login should set the auth cookie")
        .to_string()
}

fn post_form(uri: &str, cookie_header: Option<&str>, body: String) -> Request<Body> {
    let mut req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("content-type", "application/x-www-form-urlencoded");
    if let Some(c) = cookie_header {
        req = req.header("cookie", c);
    }
    req.body(Body::from(body)).unwrap()
}

/// The create/update/delete/action forms must all reject a POST whose `_csrf`
/// doesn't match — proving the token is threaded through each adapter route,
/// including delete (which gained a form body) and the action route (whose body
/// parsing changed from JSON to form).
#[tokio::test]
async fn resource_forms_require_csrf() {
    setup();
    register_resource(Box::new(Widget));

    let auth = login_as_admin().await;

    // Fetch the "new" form as the logged-in admin; it mints a CSRF cookie whose
    // token is echoed into the page.
    let new_page = app()
        .oneshot(
            Request::builder()
                .uri("/adminx/widgets/new")
                .header("cookie", format!("adminx_token={auth}"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(new_page.status(), StatusCode::OK, "admin can open the new form");
    let csrf = cookie_named(&cookies(&new_page), "adminx_csrf")
        .expect("the form page mints a CSRF cookie")
        .to_string();
    let body = axum::body::to_bytes(new_page.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(body.to_vec()).unwrap();
    assert!(html.contains(&format!(r#"name="_csrf" value="{csrf}""#)));

    // Both cookies travel together on a real same-site request.
    let jar = format!("adminx_token={auth}; adminx_csrf={csrf}");

    // Each row: (label, uri, valid body, tokenless body, success status).
    // `create` and the `toggle` action return 2xx/3xx on success; delete 303.
    let cases = [
        ("create", "/adminx/widgets/create", StatusCode::SEE_OTHER),
        ("delete", "/adminx/widgets/1/delete", StatusCode::SEE_OTHER),
        ("action", "/adminx/widgets/1/action/toggle", StatusCode::OK),
    ];

    for (label, uri, ok_status) in cases {
        // With the token -> succeeds.
        let good = app()
            .oneshot(post_form(uri, Some(&jar), format!("name=Ada&_csrf={csrf}")))
            .await
            .unwrap();
        assert_eq!(good.status(), ok_status, "{label}: valid token should pass");

        // Same request, token missing -> 403.
        let bad = app()
            .oneshot(post_form(uri, Some(&jar), "name=Ada".into()))
            .await
            .unwrap();
        assert_eq!(bad.status(), StatusCode::FORBIDDEN, "{label}: no token -> rejected");

        // Token present but wrong -> 403.
        let wrong = app()
            .oneshot(post_form(uri, Some(&jar), "name=Ada&_csrf=nope".into()))
            .await
            .unwrap();
        assert_eq!(wrong.status(), StatusCode::FORBIDDEN, "{label}: bad token -> rejected");
    }
}
