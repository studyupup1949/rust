// Proves `router_at(mount)` threads a custom mount through to the rendered page:
// a router nested at "/admin" must build its form actions and links against
// "/admin", not the hardcoded "/adminx". This is the regression guard for the
// mount-parameterization fix.

use axum::body::Body;
use axum::http::{Request, StatusCode};
use axum::Router;
use tower::util::ServiceExt;

#[tokio::test]
async fn custom_mount_is_reflected_in_rendered_urls() {
    // No resources registered and auth left unconfigured: the login page renders
    // on its own, and its form action is built from the mount in `ReqCtx`.
    let app = Router::new().nest("/admin", adminx_axum::router_at("/admin"));

    let resp = app
        .oneshot(Request::builder().uri("/admin/login").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();

    assert!(
        html.contains(r#"action="/admin/login""#),
        "the login form must post to the custom mount, got HTML without /admin/login"
    );
    assert!(
        !html.contains(r#"action="/adminx/login""#),
        "must not fall back to the hardcoded /adminx"
    );
}

#[tokio::test]
async fn default_router_still_uses_adminx() {
    let app = Router::new().nest("/adminx", adminx_axum::router());
    let resp = app
        .oneshot(Request::builder().uri("/adminx/login").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let bytes = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let html = String::from_utf8(bytes.to_vec()).unwrap();
    assert!(html.contains(r#"action="/adminx/login""#));
}
