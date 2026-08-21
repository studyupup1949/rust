//! Response conversion utilities between Actix and `http` types.

use crate::compat::http::{status_from_http, status_to_http, version_to_http};
use crate::compat::tower::body::{ActixResponseBody, TowerBodyStream};
use actix_web::{
    body::{BodyStream, BoxBody},
    dev::ServiceResponse,
    HttpResponse,
};
use futures_util::TryStreamExt;
use http::{HeaderName, HeaderValue, Response};

use crate::compat::tower::request::RESPONSE_REGISTRY;
use actix_web::HttpMessage;
/// Converts an Actix `ServiceResponse` into an `http::Response<ActixResponseBody<BoxBody>>`.
pub fn service_response_to_http(sr: ServiceResponse) -> Response<ActixResponseBody<BoxBody>> {
    let (req, res) = sr.into_parts();

    let req_id = *req
        .extensions()
        .get::<u64>()
        .expect("req_id not found in request extensions");

    let status = status_to_http(res.status());
    let version = version_to_http(res.head().version);

    // Extract headers before consuming the body
    let mut headers = http::HeaderMap::new();
    for (name, value) in res.headers().iter() {
        if let (Some(n), Some(v)) = (
            HeaderName::from_bytes(name.as_str().as_bytes()).ok(),
            HeaderValue::from_bytes(value.as_bytes()).ok(),
        ) {
            headers.insert(n, v);
        }
    }

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
        .extensions_mut()
        .insert(crate::compat::tower::request::ResponseRegistryGuard { req_id });

    response
}

/// Converts an `http::Response<B>` into an Actix `ServiceResponse`.
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

    for (name, value) in parts.headers.iter() {
        if let (Ok(actix_name), Ok(actix_value)) = (
            actix_web::http::header::HeaderName::from_bytes(name.as_str().as_bytes()),
            actix_web::http::header::HeaderValue::from_bytes(value.as_bytes()),
        ) {
            builder.append_header((actix_name, actix_value));
        }
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
