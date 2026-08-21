// End-to-end CSRF check through the real Actix scope — the mirror of
// adminx-axum/tests/csrf.rs. Worth duplicating rather than trusting symmetry:
// the two adapters read cookies by different routes (`HttpRequest::cookie` here
// vs. hand-parsing the `Cookie` header on Axum), so this seam can break on one
// framework while the other stays green.

use adminx_core::actions::{ActionFuture, CustomAction};
use adminx_core::prelude::*;
use adminx_core::storage::{CreateOutcome, ListPage, QueryOptions, StorageError};
use actix_web::{test, App};
use async_trait::async_trait;
use serde_json::{json, Map, Value};

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

fn setup() {
    configure_auth(AuthConfig {
        jwt_secret: "test-secret-key-that-is-long-enough-32b".into(),
        token_ttl_secs: 3600,
        admin_table: "adminx_users".into(),
        secure_cookie: false,
    });
    set_storage(Box::new(AuthMock));
}

fn cookie_named(resp: &actix_web::dev::ServiceResponse, name: &str) -> Option<String> {
    resp.response()
        .headers()
        .get_all(actix_web::http::header::SET_COOKIE)
        .filter_map(|v| v.to_str().ok())
        .find_map(|c| c.strip_prefix(&format!("{name}=")))
        .and_then(|v| v.split(';').next())
        .map(str::to_string)
}

#[actix_web::test]
async fn browser_login_succeeds_and_forged_post_is_rejected() {
    setup();
    let app = test::init_service(App::new().service(adminx_actix::scope())).await;

    // --- 1. Fetch the login form. ---
    let req = test::TestRequest::get().uri("/adminx/login").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let token = cookie_named(&resp, "adminx_csrf").expect("GET /login must mint a CSRF cookie");

    let html = String::from_utf8(test::read_body(resp).await.to_vec()).unwrap();
    assert!(
        html.contains(&format!(r#"name="_csrf" value="{token}""#)),
        "the served form must carry the token matching the cookie it set"
    );

    let creds = format!("email=admin%40x.io&password=secret&_csrf={token}");

    // --- 2. Post it back with the cookie: genuine login. ---
    let req = test::TestRequest::post()
        .uri("/adminx/login")
        .insert_header(("content-type", "application/x-www-form-urlencoded"))
        .cookie(actix_web::cookie::Cookie::new("adminx_csrf", token.clone()))
        .set_payload(creds.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 303, "genuine login must succeed");
    assert!(
        cookie_named(&resp, "adminx_token").is_some(),
        "genuine login must set the auth cookie"
    );

    // --- 3. Forged post: no CSRF cookie (SameSite=Strict withholds it). ---
    let req = test::TestRequest::post()
        .uri("/adminx/login")
        .insert_header(("content-type", "application/x-www-form-urlencoded"))
        .set_payload(creds)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "no cookie -> rejected");
    assert!(
        cookie_named(&resp, "adminx_token").is_none(),
        "a forged post must never log anyone in"
    );

    // --- 4. Cookie present, field mismatched. ---
    let req = test::TestRequest::post()
        .uri("/adminx/login")
        .insert_header(("content-type", "application/x-www-form-urlencoded"))
        .cookie(actix_web::cookie::Cookie::new("adminx_csrf", token))
        .set_payload("email=admin%40x.io&password=secret&_csrf=wrong")
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403, "mismatch -> rejected");
    assert!(cookie_named(&resp, "adminx_token").is_none());
}

fn toggle(_ctx: ReqCtx, id: String, _body: Value) -> ActionFuture {
    Box::pin(async move { ApiResponse::ok(json!({ "toggled": id })) })
}

/// A resource whose create, delete, and action forms all do real work.
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

/// Mirror of the Axum `resource_forms_require_csrf`. Especially load-bearing on
/// Actix, whose action route changed from reading a JSON body to a form body —
/// the very seam this drives.
#[actix_web::test]
async fn resource_forms_require_csrf() {
    setup();
    register_resource(Box::new(Widget));
    let app = test::init_service(App::new().service(adminx_actix::scope())).await;

    // Log in for an auth cookie.
    let form = test::call_service(
        &app,
        test::TestRequest::get().uri("/adminx/login").to_request(),
    )
    .await;
    let login_csrf = cookie_named(&form, "adminx_csrf").unwrap();
    let login = test::call_service(
        &app,
        test::TestRequest::post()
            .uri("/adminx/login")
            .insert_header(("content-type", "application/x-www-form-urlencoded"))
            .cookie(actix_web::cookie::Cookie::new("adminx_csrf", login_csrf.clone()))
            .set_payload(format!("email=admin%40x.io&password=secret&_csrf={login_csrf}"))
            .to_request(),
    )
    .await;
    let auth = cookie_named(&login, "adminx_token").expect("login sets auth cookie");

    // Open the new-widget form as the admin; capture its CSRF token.
    let new_page = test::call_service(
        &app,
        test::TestRequest::get()
            .uri("/adminx/widgets/new")
            .cookie(actix_web::cookie::Cookie::new("adminx_token", auth.clone()))
            .to_request(),
    )
    .await;
    assert_eq!(new_page.status(), 200, "admin can open the new form");
    let csrf = cookie_named(&new_page, "adminx_csrf").expect("form page mints a CSRF cookie");
    let html = String::from_utf8(test::read_body(new_page).await.to_vec()).unwrap();
    assert!(html.contains(&format!(r#"name="_csrf" value="{csrf}""#)));

    let cases = [
        ("create", "/adminx/widgets/create", 303),
        ("delete", "/adminx/widgets/1/delete", 303),
        ("action", "/adminx/widgets/1/action/toggle", 200),
    ];

    for (label, uri, ok_status) in cases {
        // Both cookies + a matching token -> succeeds.
        let good = test::call_service(
            &app,
            test::TestRequest::post()
                .uri(uri)
                .insert_header(("content-type", "application/x-www-form-urlencoded"))
                .cookie(actix_web::cookie::Cookie::new("adminx_token", auth.clone()))
                .cookie(actix_web::cookie::Cookie::new("adminx_csrf", csrf.clone()))
                .set_payload(format!("name=Ada&_csrf={csrf}"))
                .to_request(),
        )
        .await;
        assert_eq!(good.status(), ok_status, "{label}: valid token should pass");

        // No token in the body -> 403.
        let bad = test::call_service(
            &app,
            test::TestRequest::post()
                .uri(uri)
                .insert_header(("content-type", "application/x-www-form-urlencoded"))
                .cookie(actix_web::cookie::Cookie::new("adminx_token", auth.clone()))
                .cookie(actix_web::cookie::Cookie::new("adminx_csrf", csrf.clone()))
                .set_payload("name=Ada")
                .to_request(),
        )
        .await;
        assert_eq!(bad.status(), 403, "{label}: no token -> rejected");
    }
}
