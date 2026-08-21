//! Common error types for API responses.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use serde::Serialize;
use thiserror::Error;

/// Error code enum for structured API errors.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    /// Bad request (400).
    BadRequest,
    /// Unauthorized (401).
    Unauthorized,
    /// Forbidden (403).
    Forbidden,
    /// Not found (404).
    NotFound,
    /// Conflict (409).
    Conflict,
    /// Validation error (422).
    ValidationError,
    /// Internal server error (500).
    InternalError,
    /// Service unavailable (503).
    ServiceUnavailable,
    /// Custom error code.
    Custom(String),
}

impl std::fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ErrorCode::Custom(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

/// Structured API error response body.
#[derive(Debug, Serialize)]
pub struct ApiErrorResponse {
    /// The error code.
    pub code: ErrorCode,
    /// Human-readable error message.
    pub message: String,
    /// Optional additional details.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

/// A common error type for API applications.
#[derive(Debug, Error)]
pub struct ApiError {
    code: ErrorCode,
    message: String,
    status: StatusCode,
    details: Option<serde_json::Value>,
}

impl ApiError {
    /// Create a new API error.
    pub fn new(code: ErrorCode, message: impl Into<String>, status: StatusCode) -> Self {
        Self {
            code,
            message: message.into(),
            status,
            details: None,
        }
    }

    /// Add details to the error.
    pub fn with_details(mut self, details: serde_json::Value) -> Self {
        self.details = Some(details);
        self
    }

    /// Bad request error (400).
    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::BadRequest, msg, StatusCode::BAD_REQUEST)
    }

    /// Unauthorized error (401).
    pub fn unauthorized(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Unauthorized, msg, StatusCode::UNAUTHORIZED)
    }

    /// Forbidden error (403).
    pub fn forbidden(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::Forbidden, msg, StatusCode::FORBIDDEN)
    }

    /// Not found error (404).
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::new(ErrorCode::NotFound, msg, StatusCode::NOT_FOUND)
    }

    /// Internal error (500).
    pub fn internal(msg: impl Into<String>) -> Self {
        Self::new(
            ErrorCode::InternalError,
            msg,
            StatusCode::INTERNAL_SERVER_ERROR,
        )
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)
    }
}

impl ResponseError for ApiError {
    fn status_code(&self) -> StatusCode {
        self.status
    }

    fn error_response(&self) -> HttpResponse<actix_web::body::BoxBody> {
        let body = ApiErrorResponse {
            code: self.code.clone(),
            message: self.message.clone(),
            details: self.details.clone(),
        };

        HttpResponse::build(self.status).json(body)
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        Self::internal(e.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self::bad_request(e.to_string())
    }
}
