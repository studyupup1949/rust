//! Response conversion utilities between Actix and `http` types.
//!
//! # Optimisation notes
//!
//! ## Optimised header copy
//!
//! The original implementation iterated headers and called
//! `HeaderName::from_bytes` + `HeaderValue::from_bytes` on each entry,
//! performing O(n) full byte-scan validation at every bridge crossing.
//!
//! Because `actix-web` uses `http v0.2` and `tower`/`hyper` use `http v1`,
//! the two `HeaderMap` types cannot be pointer-cast between each other.
//! However, the byte-content of header values is already validated (they
//! come from a live HTTP connection). The `header_bridge` module replaces
//! `HeaderValue::from_bytes` with `from_maybe_shared_unchecked`, which skips
//! the byte-scan while preserving correctness.
//!
//! ## Inlining
//!
//! Both functions are `#[inline(always)]` so LLVM can fold them into their
//! callers and eliminate function-call overhead.

use crate::compat::http::{status_from_http, status_to_http, version_to_http};
use crate::compat::tower::body::{ActixResponseBody, TowerBodyStream};
use crate::compat::tower::header_bridge::{copy_actix_headers_to_http, copy_http_headers_to_actix};
use actix_web::{
    body::{BodyStream, BoxBody},
    dev::ServiceResponse,
    HttpResponse,
};
use futures_util::TryStreamExt;
use http::Response;

use crate::compat::tower::request::RESPONSE_REGISTRY;
use actix_web::HttpMessage;

/// Converts an Actix `ServiceResponse` into an `http::Response<ActixResponseBody<BoxBody>>`.
///
/// Header values are copied using `from_maybe_shared_unchecked`, skipping
/// byte-scan validation since the bytes are already valid from the HTTP stack.
#[inline(always)]
pub fn service_response_to_http(sr: ServiceResponse) -> Response<ActixResponseBody<BoxBody>> {
    let (req, res) = sr.into_parts();

    let req_id = *req
        .extensions()
        .get::<u64>()
        .expect("req_id not found in request extensions");

    let status = status_to_http(res.status());
    let version = version_to_http(res.head().version);

    // Optimised header copy — skip byte-scan validation on values.
    let headers = copy_actix_headers_to_http(res.headers());

    RESPONSE_REGISTRY.with(|registry| {
        registry.borrow_mut().insert(req_id, req);
    });

    let body = res.into_body();
    let tower_body = ActixResponseBody::from_box_body(body);

    let mut response = Response::builder()
        .status(status)
        .version(version)
        .body(tower_body)
        .expect("failed to build http::Response");

    *response.headers_mut() = headers;

    response
}

/// Converts an `http::Response<B>` into an Actix `ServiceResponse`.
///
/// Header values are copied using `from_maybe_shared_unchecked`, skipping
/// byte-scan validation since the bytes are already valid from the HTTP stack.
#[inline(always)]
pub fn http_to_service_response<B>(
    http_response: Response<B>,
    req: actix_web::HttpRequest,
) -> ServiceResponse
where
    B: http_body::Body<Data = actix_web::web::Bytes> + 'static,
    B::Error: std::fmt::Display + 'static,
{
    let (parts, body) = http_response.into_parts();

    let mut builder = HttpResponse::build(status_from_http(parts.status));

    // Optimised header copy — skip byte-scan validation on values.
    let actix_headers = copy_http_headers_to_actix(&parts.headers);
    for (name, value) in actix_headers.iter() {
        builder.append_header((name.clone(), value.clone()));
    }

    let body_stream = TowerBodyStream::new(body).map_err(|e| {
        let boxed: crate::internal::common::BoxError =
            Box::new(crate::internal::common::StringError(e.0.to_string()));
        crate::internal::common::TowerError(boxed)
    });
    let actix_body = BodyStream::new(body_stream);

    let response = builder.body(actix_body);

    ServiceResponse::new(req, response)
}
