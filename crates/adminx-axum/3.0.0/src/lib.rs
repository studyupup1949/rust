// adminx-axum/src/lib.rs
//
// Axum adapter. Translates the neutral `ApiResponse` into an Axum response,
// extracts the auth cookie into a `ReqCtx`, and wires each registered resource's
// JSON API and HTML UI (plus login/logout) onto an `axum::Router`. All behavior
// lives in `adminx_core`; this is translation only.

use std::collections::HashMap;
use std::sync::Arc;

use adminx_core::auth;
use adminx_core::csrf;
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
    /// Hidden CSRF field; absent on a forged post, hence `Option`.
    #[serde(rename = "_csrf")]
    csrf: Option<String>,
}

#[derive(Deserialize)]
struct CodeForm {
    code: String,
    #[serde(rename = "_csrf")]
    csrf: Option<String>,
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

/// Pull a named cookie out of the `Cookie` request header.
fn cookie_from_headers(headers: &HeaderMap, name: &str) -> Option<String> {
    let cookie = headers.get(COOKIE)?.to_str().ok()?;
    let prefix = format!("{name}=");
    cookie
        .split(';')
        .map(|p| p.trim())
        .find_map(|p| p.strip_prefix(&prefix).map(|v| v.to_string()))
}

fn make_ctx(headers: &HeaderMap, query: &str, mount: &str) -> ReqCtx {
    auth::build_ctx(
        mount,
        query,
        cookie_from_headers(headers, auth::COOKIE_NAME).as_deref(),
        cookie_from_headers(headers, csrf::COOKIE_NAME).as_deref(),
    )
}

/// Build the JSON API + HTML UI sub-router for a single resource.
fn resource_router(resource: Box<dyn Resource>, mount: &'static str) -> Router {
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
                        let ctx = make_ctx(&headers, &q.unwrap_or_default(), mount);
                        into_response(r.list(&ctx).await)
                    }
                }
            })
            .post({
                let r = r.clone();
                move |headers: HeaderMap, Json(body): Json<Value>| {
                    let r = r.clone();
                    async move { into_response(r.create(&make_ctx(&headers, "", mount), body).await) }
                }
            }),
        )
        .route(
            "/api/{id}",
            get({
                let r = r.clone();
                move |headers: HeaderMap, Path(id): Path<String>| {
                    let r = r.clone();
                    async move { into_response(r.get(&make_ctx(&headers, "", mount), &id).await) }
                }
            })
            .put({
                let r = r.clone();
                move |headers: HeaderMap, Path(id): Path<String>, Json(body): Json<Value>| {
                    let r = r.clone();
                    async move { into_response(r.update(&make_ctx(&headers, "", mount), &id, body).await) }
                }
            })
            .delete({
                let r = r.clone();
                move |headers: HeaderMap, Path(id): Path<String>| {
                    let r = r.clone();
                    async move { into_response(r.delete(&make_ctx(&headers, "", mount), &id).await) }
                }
            }),
        );

    // ---------- HTML UI ----------
    let list = get({
        let r = r.clone();
        move |headers: HeaderMap, RawQuery(q): RawQuery| {
            let r = r.clone();
            async move {
                let ctx = make_ctx(&headers, &q.unwrap_or_default(), mount);
                into_response(r.list_page(&ctx).await)
            }
        }
    });
    let new = get({
        let r = r.clone();
        move |headers: HeaderMap| {
            let r = r.clone();
            async move { into_response(r.new_page(&make_ctx(&headers, "", mount)).await) }
        }
    });
    let view = get({
        let r = r.clone();
        move |headers: HeaderMap, Path(id): Path<String>| {
            let r = r.clone();
            async move { into_response(r.view_page(&make_ctx(&headers, "", mount), &id).await) }
        }
    });
    let edit = get({
        let r = r.clone();
        move |headers: HeaderMap, Path(id): Path<String>| {
            let r = r.clone();
            async move { into_response(r.edit_page(&make_ctx(&headers, "", mount), &id).await) }
        }
    });
    let create = post({
        let r = r.clone();
        move |headers: HeaderMap, Form(form): Form<HashMap<String, String>>| {
            let r = r.clone();
            async move { into_response(r.create_form(&make_ctx(&headers, "", mount), form).await) }
        }
    });
    let update = post({
        let r = r.clone();
        move |headers: HeaderMap, Path(id): Path<String>, Form(form): Form<HashMap<String, String>>| {
            let r = r.clone();
            async move { into_response(r.update_form(&make_ctx(&headers, "", mount), &id, form).await) }
        }
    });
    let delete = post({
        let r = r.clone();
        move |headers: HeaderMap, Path(id): Path<String>, Form(mut form): Form<HashMap<String, String>>| {
            let r = r.clone();
            let csrf = form.remove(csrf::FIELD_NAME);
            async move { into_response(r.delete_form(&make_ctx(&headers, "", mount), &id, csrf).await) }
        }
    });
    let action = post({
        let r = r.clone();
        // The action button submits a url-encoded form, so parse it as one (the
        // remaining fields become the action's JSON body) and lift out `_csrf`.
        move |headers: HeaderMap, Path((id, name)): Path<(String, String)>, Form(mut form): Form<HashMap<String, String>>| {
            let r = r.clone();
            let csrf = form.remove(csrf::FIELD_NAME);
            let payload = ui::form_to_json(form);
            async move {
                into_response(r.run_action(&make_ctx(&headers, "", mount), &name, id, payload, csrf).await)
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

async fn dashboard(headers: HeaderMap, mount: &'static str) -> Response {
    let ctx = make_ctx(&headers, "", mount);
    if let Some(deny) = auth::guard_ui(&ctx) {
        return into_response(deny);
    }
    into_response(ui::dashboard(&ctx))
}

async fn login_form(headers: HeaderMap, mount: &'static str) -> Response {
    into_response(auth::login_page(&make_ctx(&headers, "", mount), None))
}

async fn login_submit(headers: HeaderMap, mount: &'static str, Form(form): Form<LoginForm>) -> Response {
    let ctx = make_ctx(&headers, "", mount);
    into_response(auth::handle_login(&ctx, &form.email, &form.password, form.csrf.as_deref()).await)
}

async fn logout(headers: HeaderMap, mount: &'static str) -> Response {
    into_response(auth::handle_logout(&make_ctx(&headers, "", mount)))
}

async fn mfa_setup_get(headers: HeaderMap, mount: &'static str) -> Response {
    into_response(auth::mfa_setup_page(&make_ctx(&headers, "", mount), None).await)
}

async fn mfa_enable_post(headers: HeaderMap, mount: &'static str, Form(form): Form<CodeForm>) -> Response {
    let ctx = make_ctx(&headers, "", mount);
    into_response(auth::handle_mfa_enable(&ctx, &form.code, form.csrf.as_deref()).await)
}

async fn mfa_verify_get(headers: HeaderMap, mount: &'static str) -> Response {
    into_response(auth::mfa_verify_page(&make_ctx(&headers, "", mount), None).await)
}

async fn mfa_verify_post(headers: HeaderMap, mount: &'static str, Form(form): Form<CodeForm>) -> Response {
    let ctx = make_ctx(&headers, "", mount);
    into_response(auth::handle_mfa_verify(&ctx, &form.code, form.csrf.as_deref()).await)
}

async fn health() -> Response {
    let ok = storage().health().await;
    let body = json!({ "status": if ok { "healthy" } else { "unhealthy" } });
    into_response(ApiResponse::json(if ok { 200 } else { 503 }, body))
}

/// Build the full adminx router for the default `/adminx` mount:
/// `Router::new().nest("/adminx", adminx_axum::router())`.
pub fn router() -> Router {
    router_at(MOUNT)
}

/// Build the adminx router for a custom mount prefix. Pass the **same** path you
/// nest it at, so in-page links, redirects, and form actions resolve correctly:
///
/// ```ignore
/// Router::new().nest("/admin", adminx_axum::router_at("/admin"));
/// ```
///
/// The prefix must be a `&'static str` (it is captured by the route handlers).
pub fn router_at(mount: &'static str) -> Router {
    let mut router = Router::new()
        .route("/", get(move |h: HeaderMap| dashboard(h, mount)))
        .route("/health", get(health))
        .route(
            "/login",
            get(move |h: HeaderMap| login_form(h, mount))
                .post(move |h: HeaderMap, f: Form<LoginForm>| login_submit(h, mount, f)),
        )
        .route("/logout", get(move |h: HeaderMap| logout(h, mount)))
        .route("/mfa/setup", get(move |h: HeaderMap| mfa_setup_get(h, mount)))
        .route(
            "/mfa/enable",
            post(move |h: HeaderMap, f: Form<CodeForm>| mfa_enable_post(h, mount, f)),
        )
        .route(
            "/mfa/verify",
            get(move |h: HeaderMap| mfa_verify_get(h, mount))
                .post(move |h: HeaderMap, f: Form<CodeForm>| mfa_verify_post(h, mount, f)),
        );

    let resources = all_resources();
    tracing::info!("adminx-axum: mounting {} resource(s) at '{}'", resources.len(), mount);

    for resource in resources {
        let base = format!("/{}", resource.base_path());
        tracing::info!(
            "adminx-axum: mounting '{}' at '{}{}'",
            resource.resource_name(),
            mount,
            base
        );
        router = router.nest(&base, resource_router(resource, mount));
    }

    router
}
