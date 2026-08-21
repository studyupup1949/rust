//! Request conversion utilities between Actix and `http` types.

use crate::compat::http::{method_to_http, uri_to_http, version_to_http};
use crate::compat::tower::body::{collect_body_limited, ActixRequestBody};
use crate::internal::common::{BoxError, StringError, TowerError};
use actix_web::{
    dev::{Payload, ServiceRequest},
    HttpMessage,
};
use http::{HeaderName, HeaderValue, Request};

use std::cell::RefCell;
use std::collections::HashMap;

thread_local! {
    pub(crate) static REQUEST_REGISTRY: RefCell<HashMap<u64, actix_web::HttpRequest>> = RefCell::new(HashMap::new());
    pub(crate) static RESPONSE_REGISTRY: RefCell<HashMap<u64, actix_web::HttpRequest>> = RefCell::new(HashMap::new());
}

static NEXT_REQUEST_ID: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

/// Default maximum request body size passed through the Tower bridge: 4 MiB.
///
/// Tower middleware receives the entire request body eagerly buffered.
/// This constant protects against OOM from arbitrarily large uploads.
/// Override via [`TowerLayer::with_max_body_bytes`](crate::compat::tower::TowerLayer::with_max_body_bytes).
pub const DEFAULT_MAX_BODY_BYTES: usize = 4 * 1024 * 1024; // 4 MiB

/// Converts an Actix `ServiceRequest` into an `http::Request<ActixRequestBody>`.
///
/// The original `HttpRequest` is stored in a thread-local registry keyed by
/// a generated `req_id`.  A [`RequestRegistryGuard`] placed in the returned
/// request's extensions ensures the registry entry is cleaned up even if the
/// request is cancelled.
pub fn service_request_to_http(sr: ServiceRequest) -> Request<ActixRequestBody> {
    let (http_req, payload) = sr.into_parts();

    let method = method_to_http(http_req.method().clone());
    let uri = uri_to_http(http_req.uri());
    let version = version_to_http(http_req.version());

    let body = ActixRequestBody::new(payload);

    let mut http_request = Request::builder()
        .method(method)
        .uri(uri)
        .version(version)
        .body(body)
        .expect("failed to build http::Request");

    for (name, value) in http_req.headers().iter() {
        if let (Some(n), Some(v)) = (
            HeaderName::from_bytes(name.as_str().as_bytes()).ok(),
            HeaderValue::from_bytes(value.as_bytes()).ok(),
        ) {
            http_request.headers_mut().insert(n, v);
        }
    }

    let req_id = NEXT_REQUEST_ID.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

    // Store req_id in the HttpRequest's extensions so `service_response_to_http`
    // can retrieve it from the response side.
    http_req.extensions_mut().insert(req_id);

    REQUEST_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(req_id, http_req);
    });

    http_request
        .extensions_mut()
        .insert(RequestRegistryGuard { req_id });

    http_request
}

/// Reconstructs a `ServiceRequest` from an `http::Request<B>`.
///
/// # Errors
///
/// Returns an error if:
/// - The [`RequestRegistryGuard`] has been removed from the request extensions
///   (Tower middleware that strips extensions).
/// - The request body exceeds `max_body_bytes`.
/// - The request body stream returns an I/O error.
///
/// On error the corresponding `REQUEST_REGISTRY` entry is cleaned up.
pub async fn http_to_service_request(
    mut http_request: Request<ActixRequestBody>,
    max_body_bytes: usize,
) -> Result<ServiceRequest, actix_web::Error> {
    // --- Retrieve the registry guard -------------------------------------------
    // If missing, Tower middleware removed it (e.g. a sanitization layer that
    // calls extensions_mut().clear()).  Return a typed error; never panic.
    let guard = http_request
        .extensions_mut()
        .remove::<RequestRegistryGuard>()
        .ok_or_else(|| {
            actix_web::Error::from(TowerError(Box::new(StringError(
                "actix_tower: RequestRegistryGuard missing from http::Request extensions. \
                 A Tower middleware likely cleared or replaced request extensions. \
                 Middleware that intercepts extensions must preserve the RequestRegistryGuard."
                    .to_owned(),
            )) as BoxError))
        })?;

    let req_id = guard.req_id;
    // Disarm: we take ownership of cleanup from here.
    std::mem::forget(guard);

    // --- Retrieve the original HttpRequest -------------------------------------
    let http_req = REQUEST_REGISTRY
        .with(|registry| registry.borrow_mut().remove(&req_id))
        .ok_or_else(|| {
            actix_web::Error::from(TowerError(Box::new(StringError(
                "actix_tower: HttpRequest not found in REQUEST_REGISTRY. \
                 This is an internal bug; please file an issue."
                    .to_owned(),
            )) as BoxError))
        })?;

    // --- Buffer the request body with a hard size limit -----------------------
    // The body has already been handed to Tower middleware as a stream.
    // We must re-buffer it to reconstruct the Actix Payload.
    // `collect_body_limited` aborts without allocating more than `max_body_bytes`.
    let body = http_request.into_body();
    let body_bytes = collect_body_limited(body, max_body_bytes)
        .await
        .map_err(|e| {
            // Distinguish body-too-large from generic I/O error.
            let msg = e.to_string();
            if msg.contains("too large") {
                actix_web::error::ErrorPayloadTooLarge(
                    "request body exceeds the Tower bridge body limit",
                )
            } else {
                actix_web::error::ErrorBadRequest(format!("request body read error: {msg}"))
            }
        })?;

    let payload = if body_bytes.is_empty() {
        Payload::None
    } else {
        Payload::from(body_bytes)
    };

    Ok(ServiceRequest::from_parts(http_req, payload))
}

// ---------------------------------------------------------------------------
// Guards — ensure registry cleanup on all exit paths
// ---------------------------------------------------------------------------

/// Placed in `http::Request` extensions to clean up `REQUEST_REGISTRY` on drop.
///
/// `mem::forget`-ted by `http_to_service_request` once the entry is explicitly
/// removed, so cleanup never runs twice.
#[derive(Clone)]
pub struct RequestRegistryGuard {
    pub(crate) req_id: u64,
}

impl Drop for RequestRegistryGuard {
    fn drop(&mut self) {
        let req_opt = REQUEST_REGISTRY.with(|registry| {
            registry.borrow_mut().remove(&self.req_id)
        });
        if let Some(req) = req_opt {
            RESPONSE_REGISTRY.with(|registry| {
                registry.borrow_mut().insert(self.req_id, req);
            });
        }
    }
}

/// Placed in `http::Response` extensions to clean up `RESPONSE_REGISTRY` on drop.
///
/// `mem::forget`-ted by `TowerMiddlewareService::call` once the entry is
/// explicitly removed.
#[derive(Clone)]
pub struct ResponseRegistryGuard {
    pub(crate) req_id: u64,
}

impl Drop for ResponseRegistryGuard {
    fn drop(&mut self) {
        RESPONSE_REGISTRY.with(|registry| {
            registry.borrow_mut().remove(&self.req_id);
        });
    }
}
