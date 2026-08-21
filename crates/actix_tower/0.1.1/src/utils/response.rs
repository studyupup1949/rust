//! Typed response utilities.

use actix_web::{HttpResponse, Responder};
use serde::Serialize;

/// A typed API response wrapper.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct User { id: u32, name: String }
///
/// async fn get_user() -> impl Responder {
///     ApiResponse::ok(User { id: 1, name: "Alice".into() })
/// }
/// ```
#[derive(Debug, Serialize)]
pub struct ApiResponse<T: Serialize> {
    /// Whether the request was successful.
    pub success: bool,
    /// The response data (present if success is true).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error message (present if success is false).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl<T: Serialize> ApiResponse<T> {
    /// Create a successful response.
    pub fn ok(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    /// Create an error response.
    pub fn error(msg: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(msg.into()),
        }
    }
}

impl<T: Serialize> Responder for ApiResponse<T> {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        if self.success {
            HttpResponse::Ok().json(self)
        } else {
            HttpResponse::InternalServerError().json(self)
        }
    }
}

/// A typed response with a configurable status code.
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use actix_web::http::StatusCode;
/// use serde::Serialize;
///
/// #[derive(Serialize)]
/// struct User { id: u32, name: String }
///
/// async fn create_user() -> impl Responder {
///     TypedResponse::new(StatusCode::CREATED, User { id: 1, name: "Alice".into() })
/// }
/// ```
#[derive(Debug)]
pub struct TypedResponse<T: Serialize> {
    status: actix_web::http::StatusCode,
    data: T,
}

impl<T: Serialize> TypedResponse<T> {
    /// Create a new typed response with a custom status code.
    pub fn new(status: actix_web::http::StatusCode, data: T) -> Self {
        Self { status, data }
    }

    /// Create a 200 OK response.
    pub fn ok(data: T) -> Self {
        Self::new(actix_web::http::StatusCode::OK, data)
    }

    /// Create a 201 Created response.
    pub fn created(data: T) -> Self {
        Self::new(actix_web::http::StatusCode::CREATED, data)
    }
}

impl<T: Serialize> Responder for TypedResponse<T> {
    type Body = actix_web::body::BoxBody;

    fn respond_to(self, _req: &actix_web::HttpRequest) -> HttpResponse<Self::Body> {
        HttpResponse::build(self.status).json(self.data)
    }
}
