// adminx-axum/src/lib.rs
//
// Axum adapter. Translates the neutral `ApiResponse` into an Axum response,
// extracts the auth cookie into a `ReqCtx`, and wires each registered resource's
// JSON API and HTML UI (plus login/logout) onto an `axum::Router`. All behavior
// lives in `adminx_core`; this is translation only.

use std::collections::HashMap;
use std::sync::Arc;

use adminx_core::auth;
use adminx_core::registry::all_resources;
use adminx_core::request::ReqCtx;
use adminx_core::resource::Resource;
use adminx_core::response::{ApiBody, ApiResponse};
use adminx_core::storage::storage;
use adminx_core::ui;

use axum::body::Body;
use axum::extract::{Form, Path, RawQuery};
use axum::http::{header::COOKIE, HeaderMap, StatusCode};
use axum::response::Response;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::{json, Value};

/// Prefix the adminx router is expected to be nested at. Used for UI links.
const MOUNT: &str = "/adminx";

#[derive(Deserialize)]
struct LoginForm {
    email: String,
    password: String,
}

#[derive(Deserialize)]
struct CodeForm {
    code: String,
}

/// Convert a neutral response into an Axum response.
fn into_response(api: ApiResponse) -> Response {
    let status = StatusCode::from_u16(api.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut builder = Response::builder().status(status);
    for (k, v) in &api.headers {
        builder = builder.header(k, v);
    }
    match api.body {
        ApiBody::Json(v) => builder
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_vec(&v).unwrap_or_default()))
            .unwrap(),
        ApiBody::Bytes { content_type, data } => builder
            .header("content-type", content_type)
            .body(Body::from(data))
            .unwrap(),
        ApiBody::Empty => builder.body(Body::empty()).unwrap(),
    }
}

/// Pull the adminx auth token out of the `Cookie` request header.
fn token_from_headers(headers: &HeaderMap) -> Option<String> {
    let cookie = headers.get(COOKIE)?.to_str().ok()?;
    let prefix = format!("{}=", auth::COOKIE_NAME);
    cookie
        .split(';')
        .map(|p| p.trim())
        .find_map(|p| p.strip_prefix(&prefix).map(|v| v.to_string()))
}

fn make_ctx(headers: &HeaderMap, query: &str) -> ReqCtx {
    auth::build_ctx(MOUNT, query, token_from_headers(headers).as_deref())
}

/// Build the JSON API + HTML UI sub-router for a single resource.
fn resource_router(resource: Box<dyn Resource>) -> Router {
    let r: Arc<dyn Resource> = Arc::from(resource);

    // ---------- JSON API ----------
    let api = Router::new()
        .route(
            "/api",
            get({
                let r = r.clone();
                move |headers: HeaderMap, RawQuery(q): RawQuery| {
                    let r = r.clone();
                    async move {
                        let ctx = make_ctx(&headers, &q.unwrap_or_default());
                        into_response(r.list(&ctx).await)
                    }
                }
            })
            .post({
                let r = r.clone();
                move |headers: HeaderMap, Json(body): Json<Value>| {
                    let r = r.clone();
                    async move { into_response(r.create(&make_ctx(&headers, ""), body).await) }
                }
            }),
        )
        .route(
            "/api/{id}",
            get({
                let r = r.clone();
                move |headers: HeaderMap, Path(id): Path<String>| {
                    let r = r.clone();
                    async move { into_response(r.get(&make_ctx(&headers, ""), &id).await) }
                }
            })
            .put({
                let r = r.clone();
                move |headers: HeaderMap, Path(id): Path<String>, Json(body): Json<Value>| {
                    let r = r.clone();
                    async move { into_response(r.update(&make_ctx(&headers, ""), &id, body).await) }
                }
            })
            .delete({
                let r = r.clone();
                move |headers: HeaderMap, Path(id): Path<String>| {
                    let r = r.clone();
                    async move { into_response(r.delete(&make_ctx(&headers, ""), &id).await) }
                }
            }),
        );

    // ---------- HTML UI ----------
    let list = get({
        let r = r.clone();
        move |headers: HeaderMap, RawQuery(q): RawQuery| {
            let r = r.clone();
            async move {
                let ctx = make_ctx(&headers, &q.unwrap_or_default());
                into_response(r.list_page(&ctx).await)
            }
        }
    });
    let new = get({
        let r = r.clone();
        move |headers: HeaderMap| {
            let r = r.clone();
            async move { into_response(r.new_page(&make_ctx(&headers, "")).await) }
        }
    });
    let view = get({
        let r = r.clone();
        move |headers: HeaderMap, Path(id): Path<String>| {
            let r = r.clone();
            async move { into_response(r.view_page(&make_ctx(&headers, ""), &id).await) }
        }
    });
    let edit = get({
        let r = r.clone();
        move |headers: HeaderMap, Path(id): Path<String>| {
            let r = r.clone();
            async move { into_response(r.edit_page(&make_ctx(&headers, ""), &id).await) }
        }
    });
    let create = post({
        let r = r.clone();
        move |headers: HeaderMap, Form(form): Form<HashMap<String, String>>| {
            let r = r.clone();
            async move { into_response(r.create_form(&make_ctx(&headers, ""), form).await) }
        }
    });
    let update = post({
        let r = r.clone();
        move |headers: HeaderMap, Path(id): Path<String>, Form(form): Form<HashMap<String, String>>| {
            let r = r.clone();
            async move { into_response(r.update_form(&make_ctx(&headers, ""), &id, form).await) }
        }
    });
    let delete = post({
        let r = r.clone();
        move |headers: HeaderMap, Path(id): Path<String>| {
            let r = r.clone();
            async move { into_response(r.delete_form(&make_ctx(&headers, ""), &id).await) }
        }
    });
    let action = post({
        let r = r.clone();
        move |headers: HeaderMap, Path((id, name)): Path<(String, String)>, bytes: axum::body::Bytes| {
            let r = r.clone();
            async move {
                let payload: Value = serde_json::from_slice(&bytes).unwrap_or_else(|_| json!({}));
                into_response(r.run_action(&make_ctx(&headers, ""), &name, id, payload).await)
            }
        }
    });

    let ui_routes = Router::new()
        .route("/list", list)
        .route("/new", new)
        .route("/view/{id}", view)
        .route("/edit/{id}", edit)
        .route("/create", create)
        .route("/update/{id}", update)
        .route("/{id}/delete", delete)
        .route("/{id}/action/{name}", action);

    api.merge(ui_routes)
}

async fn dashboard(headers: HeaderMap) -> Response {
    let ctx = make_ctx(&headers, "");
    if let Some(deny) = auth::guard_ui(&ctx) {
        return into_response(deny);
    }
    into_response(ui::dashboard(&ctx))
}

async fn login_form(headers: HeaderMap) -> Response {
    into_response(auth::login_page(&make_ctx(&headers, ""), None))
}

async fn login_submit(headers: HeaderMap, Form(form): Form<LoginForm>) -> Response {
    let ctx = make_ctx(&headers, "");
    into_response(auth::handle_login(&ctx, &form.email, &form.password).await)
}

async fn logout(headers: HeaderMap) -> Response {
    into_response(auth::handle_logout(&make_ctx(&headers, "")))
}

async fn mfa_setup_get(headers: HeaderMap) -> Response {
    into_response(auth::mfa_setup_page(&make_ctx(&headers, ""), None).await)
}

async fn mfa_enable_post(headers: HeaderMap, Form(form): Form<CodeForm>) -> Response {
    into_response(auth::handle_mfa_enable(&make_ctx(&headers, ""), &form.code).await)
}

async fn mfa_verify_get(headers: HeaderMap) -> Response {
    into_response(auth::mfa_verify_page(&make_ctx(&headers, ""), None).await)
}

async fn mfa_verify_post(headers: HeaderMap, Form(form): Form<CodeForm>) -> Response {
    into_response(auth::handle_mfa_verify(&make_ctx(&headers, ""), &form.code).await)
}

async fn health() -> Response {
    let ok = storage().health().await;
    let body = json!({ "status": if ok { "healthy" } else { "unhealthy" } });
    into_response(ApiResponse::json(if ok { 200 } else { 503 }, body))
}

/// Build the full adminx router. Nest it at `/adminx`:
/// `Router::new().nest("/adminx", adminx_axum::router())`.
pub fn router() -> Router {
    let mut router = Router::new()
        .route("/", get(dashboard))
        .route("/health", get(health))
        .route("/login", get(login_form).post(login_submit))
        .route("/logout", get(logout))
        .route("/mfa/setup", get(mfa_setup_get))
        .route("/mfa/enable", post(mfa_enable_post))
        .route("/mfa/verify", get(mfa_verify_get).post(mfa_verify_post));

    let resources = all_resources();
    tracing::info!("adminx-axum: mounting {} resource(s)", resources.len());

    for resource in resources {
        let base = format!("/{}", resource.base_path());
        tracing::info!(
            "adminx-axum: mounting '{}' at '{}{}'",
            resource.resource_name(),
            MOUNT,
            base
        );
        router = router.nest(&base, resource_router(resource));
    }

    router
}
