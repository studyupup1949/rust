// adminx-core/src/response.rs
//
// A neutral HTTP response. Adapters translate it into `actix_web::HttpResponse`
// or `axum::response::Response`. This is the boundary that lets one Resource
// implementation serve any framework.

use crate::error::CoreError;
use serde_json::{json, Value};

#[derive(Debug, Clone)]
pub enum ApiBody {
    Json(Value),
    Bytes { content_type: String, data: Vec<u8> },
    Empty,
}

#[derive(Debug, Clone)]
pub struct ApiResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: ApiBody,
}

impl ApiResponse {
    pub fn new(status: u16, body: ApiBody) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body,
        }
    }

    pub fn json(status: u16, value: Value) -> Self {
        Self::new(status, ApiBody::Json(value))
    }

    pub fn ok(value: Value) -> Self {
        Self::json(200, value)
    }

    pub fn created(value: Value) -> Self {
        Self::json(201, value)
    }

    pub fn error(err: CoreError) -> Self {
        Self::json(err.status(), json!({ "error": err.message() }))
    }

    /// HTML body with a `text/html` content type.
    pub fn html(status: u16, markup: String) -> Self {
        Self::new(
            status,
            ApiBody::Bytes {
                content_type: "text/html; charset=utf-8".to_string(),
                data: markup.into_bytes(),
            },
        )
    }

    /// See-other redirect (303) to `location`.
    pub fn redirect(location: impl Into<String>) -> Self {
        Self::new(303, ApiBody::Empty).with_header("Location", location)
    }

    pub fn with_header(mut self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.push((name.into(), value.into()));
        self
    }
}

impl From<CoreError> for ApiResponse {
    fn from(err: CoreError) -> Self {
        ApiResponse::error(err)
    }
}
