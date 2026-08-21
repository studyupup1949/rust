//! Common internal utilities.

use std::fmt;

/// A type-erased, heap-allocated error that is `Send + Sync`.
pub type BoxError = Box<dyn std::error::Error + Send + Sync + 'static>;

/// A simple string-based error used when we need to convert a non-`Error`
/// display value into an `Error`.
#[derive(Debug)]
pub struct StringError(pub String);

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for StringError {}

// ---------------------------------------------------------------------------
// TowerError — bridges Tower `Service::Error` into Actix
// ---------------------------------------------------------------------------

/// Wraps a Tower middleware error so it can be returned as an Actix response.
///
/// # HTTP Status
///
/// `TowerError` always responds with **502 Bad Gateway** rather than 500.
/// This is semantically correct: a Tower `Service::Error` represents an
/// upstream or middleware infrastructure failure, not a server programming
/// error.  500 is reserved for bugs in your own server code.
///
/// # Tower middleware that rejects requests via `Service::Error`
///
/// Some Tower middleware (e.g. `tower::limit::RateLimitLayer`) signal
/// rejection through `Err` rather than through an OK response with a
/// non-2xx status code.  This loses HTTP status information.  The
/// recommended pattern for HTTP-aware rejection is to return:
///
/// ```text
/// Ok(http::Response::builder().status(429).body(body).unwrap())
/// ```
///
/// rather than `Err(...)`.  If you need rate-limiting with proper 429
/// responses, use `tower_governor` or Actix-native `RateLimit` middleware.
///
/// # Inspecting the original error
///
/// The original error is preserved in the `source()` chain.  Use
/// `error.source()` to retrieve it.
#[derive(Debug)]
pub struct TowerError(pub BoxError);

impl fmt::Display for TowerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "tower middleware error: {}", self.0)
    }
}

impl std::error::Error for TowerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        // The original Tower error is accessible here, fully typed.
        Some(self.0.as_ref())
    }
}

impl actix_web::ResponseError for TowerError {
    fn status_code(&self) -> actix_web::http::StatusCode {
        // 502 Bad Gateway is correct for upstream/middleware infrastructure
        // errors.  500 implies a bug in *this* server's code.
        actix_web::http::StatusCode::BAD_GATEWAY
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        actix_web::HttpResponse::build(self.status_code()).body(self.to_string())
    }
}

impl From<BoxError> for TowerError {
    fn from(e: BoxError) -> Self {
        TowerError(e)
    }
}
