// adminx-actix/src/lib.rs
//
// Actix Web adapter. Translates the neutral `ApiResponse` into an
// `actix_web::HttpResponse`, extracts the auth cookie into a `ReqCtx`, and wires
// each registered resource's JSON API and HTML UI (plus login/logout) onto an
// Actix `Scope`. All behavior lives in `adminx_core`; this is translation only.

use std::collections::HashMap;
use std::sync::Arc;

use adminx_core::auth;
use adminx_core::registry::all_resources;
use adminx_core::request::ReqCtx;
use adminx_core::resource::Resource;
use adminx_core::response::{ApiBody, ApiResponse};
use adminx_core::storage::storage;
use adminx_core::ui;

use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse, Scope};
use serde::Deserialize;
use serde_json::{json, Value};

/// Prefix the adminx scope is mounted at. Used for building UI links.
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

/// Convert a neutral response into an Actix response.
fn into_response(api: ApiResponse) -> HttpResponse {
    let status = StatusCode::from_u16(api.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut builder = HttpResponse::build(status);
    for (k, v) in &api.headers {
        if let (Ok(name), Ok(val)) = (
            HeaderName::from_bytes(k.as_bytes()),
            HeaderValue::from_str(v),
        ) {
            builder.append_header((name, val));
        }
    }
    match api.body {
        ApiBody::Json(v) => builder.json(v),
        ApiBody::Bytes { content_type, data } => builder.content_type(content_type).body(data),
        ApiBody::Empty => builder.finish(),
    }
}

/// Build a request context, verifying the auth cookie into `claims`.
fn make_ctx(req: &HttpRequest) -> ReqCtx {
    let token = req.cookie(auth::COOKIE_NAME).map(|c| c.value().to_string());
    auth::build_ctx(MOUNT, req.query_string(), token.as_deref())
}

/// Build the JSON API + HTML UI routes for a single resource.
fn resource_routes(resource: Box<dyn Resource>) -> Scope {
    let r: Arc<dyn Resource> = Arc::from(resource);
    let mut scope = web::scope("");

    // ---------- JSON API ----------
    scope = scope.route("/api", {
        let r = r.clone();
        web::get().to(move |req: HttpRequest| {
            let r = r.clone();
            async move { into_response(r.list(&make_ctx(&req)).await) }
        })
    });
    scope = scope.route("/api", {
        let r = r.clone();
        web::post().to(move |req: HttpRequest, body: web::Json<Value>| {
            let r = r.clone();
            async move { into_response(r.create(&make_ctx(&req), body.into_inner()).await) }
        })
    });
    scope = scope.route("/api/{id}", {
        let r = r.clone();
        web::get().to(move |req: HttpRequest, path: web::Path<String>| {
            let r = r.clone();
            async move { into_response(r.get(&make_ctx(&req), &path.into_inner()).await) }
        })
    });
    scope = scope.route("/api/{id}", {
        let r = r.clone();
        web::put().to(
            move |req: HttpRequest, path: web::Path<String>, body: web::Json<Value>| {
                let r = r.clone();
                async move {
                    into_response(
                        r.update(&make_ctx(&req), &path.into_inner(), body.into_inner())
                            .await,
                    )
                }
            },
        )
    });
    scope = scope.route("/api/{id}", {
        let r = r.clone();
        web::delete().to(move |req: HttpRequest, path: web::Path<String>| {
            let r = r.clone();
            async move { into_response(r.delete(&make_ctx(&req), &path.into_inner()).await) }
        })
    });

    // ---------- HTML UI ----------
    scope = scope.route("/list", {
        let r = r.clone();
        web::get().to(move |req: HttpRequest| {
            let r = r.clone();
            async move { into_response(r.list_page(&make_ctx(&req)).await) }
        })
    });
    scope = scope.route("/new", {
        let r = r.clone();
        web::get().to(move |req: HttpRequest| {
            let r = r.clone();
            async move { into_response(r.new_page(&make_ctx(&req)).await) }
        })
    });
    scope = scope.route("/view/{id}", {
        let r = r.clone();
        web::get().to(move |req: HttpRequest, path: web::Path<String>| {
            let r = r.clone();
            async move { into_response(r.view_page(&make_ctx(&req), &path.into_inner()).await) }
        })
    });
    scope = scope.route("/edit/{id}", {
        let r = r.clone();
        web::get().to(move |req: HttpRequest, path: web::Path<String>| {
            let r = r.clone();
            async move { into_response(r.edit_page(&make_ctx(&req), &path.into_inner()).await) }
        })
    });
    scope = scope.route("/create", {
        let r = r.clone();
        web::post().to(
            move |req: HttpRequest, form: web::Form<HashMap<String, String>>| {
                let r = r.clone();
                async move {
                    into_response(r.create_form(&make_ctx(&req), form.into_inner()).await)
                }
            },
        )
    });
    scope = scope.route("/update/{id}", {
        let r = r.clone();
        web::post().to(
            move |req: HttpRequest,
                  path: web::Path<String>,
                  form: web::Form<HashMap<String, String>>| {
                let r = r.clone();
                async move {
                    into_response(
                        r.update_form(&make_ctx(&req), &path.into_inner(), form.into_inner())
                            .await,
                    )
                }
            },
        )
    });
    scope = scope.route("/{id}/delete", {
        let r = r.clone();
        web::post().to(move |req: HttpRequest, path: web::Path<String>| {
            let r = r.clone();
            async move { into_response(r.delete_form(&make_ctx(&req), &path.into_inner()).await) }
        })
    });

    // ---------- Custom actions ----------
    scope = scope.route("/{id}/action/{name}", {
        let r = r.clone();
        web::post().to(
            move |req: HttpRequest, path: web::Path<(String, String)>, body: web::Bytes| {
                let r = r.clone();
                async move {
                    let (id, name) = path.into_inner();
                    let payload: Value = serde_json::from_slice(&body).unwrap_or_else(|_| json!({}));
                    into_response(r.run_action(&make_ctx(&req), &name, id, payload).await)
                }
            },
        )
    });

    scope
}

async fn dashboard(req: HttpRequest) -> HttpResponse {
    let ctx = make_ctx(&req);
    if let Some(deny) = auth::guard_ui(&ctx) {
        return into_response(deny);
    }
    into_response(ui::dashboard(&ctx))
}

async fn login_form(req: HttpRequest) -> HttpResponse {
    into_response(auth::login_page(&make_ctx(&req), None))
}

async fn login_submit(req: HttpRequest, form: web::Form<LoginForm>) -> HttpResponse {
    let ctx = make_ctx(&req);
    into_response(auth::handle_login(&ctx, &form.email, &form.password).await)
}

async fn logout(req: HttpRequest) -> HttpResponse {
    into_response(auth::handle_logout(&make_ctx(&req)))
}

async fn mfa_setup_get(req: HttpRequest) -> HttpResponse {
    into_response(auth::mfa_setup_page(&make_ctx(&req), None).await)
}

async fn mfa_enable_post(req: HttpRequest, form: web::Form<CodeForm>) -> HttpResponse {
    into_response(auth::handle_mfa_enable(&make_ctx(&req), &form.code).await)
}

async fn mfa_verify_get(req: HttpRequest) -> HttpResponse {
    into_response(auth::mfa_verify_page(&make_ctx(&req), None).await)
}

async fn mfa_verify_post(req: HttpRequest, form: web::Form<CodeForm>) -> HttpResponse {
    into_response(auth::handle_mfa_verify(&make_ctx(&req), &form.code).await)
}

async fn health() -> HttpResponse {
    let ok = storage().health().await;
    let body = json!({ "status": if ok { "healthy" } else { "unhealthy" } });
    into_response(ApiResponse::json(if ok { 200 } else { 503 }, body))
}

/// Build the full adminx Actix scope, mounted at `/adminx`.
/// `App::new().service(adminx_actix::scope())`.
pub fn scope() -> Scope {
    let mut scope = web::scope(MOUNT)
        .route("", web::get().to(dashboard))
        .route("/", web::get().to(dashboard))
        .route("/health", web::get().to(health))
        .route("/login", web::get().to(login_form))
        .route("/login", web::post().to(login_submit))
        .route("/logout", web::get().to(logout))
        .route("/mfa/setup", web::get().to(mfa_setup_get))
        .route("/mfa/enable", web::post().to(mfa_enable_post))
        .route("/mfa/verify", web::get().to(mfa_verify_get))
        .route("/mfa/verify", web::post().to(mfa_verify_post));

    let resources = all_resources();
    tracing::info!("adminx-actix: mounting {} resource(s)", resources.len());

    for resource in resources {
        let base = format!("/{}", resource.base_path());
        tracing::info!(
            "adminx-actix: mounting '{}' at '{}{}'",
            resource.resource_name(),
            MOUNT,
            base
        );
        scope = scope.service(web::scope(&base).service(resource_routes(resource)));
    }

    scope
}
