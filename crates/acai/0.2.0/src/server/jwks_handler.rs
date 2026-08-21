use std::sync::Arc;

use bytes::Bytes;
use http_body_util::combinators::BoxBody;
use hyper::{Method, Request, Response, StatusCode};
use tower::BoxError;

use crate::server::full_body;
use crate::server::jwks::JwksManager;

/// Handle JWKS endpoint requests
pub struct JwksHandler {
    jwks_manager: Arc<JwksManager>,
}

impl JwksHandler {
    /// Create a new JWKS handler with the given JWKS manager
    pub fn new(jwks_manager: Arc<JwksManager>) -> Self {
        Self { jwks_manager }
    }

    /// Handle a request to the JWKS endpoint
    pub async fn handle_request(
        &self,
        req: Request<hyper::body::Incoming>,
    ) -> Result<Response<BoxBody<Bytes, BoxError>>, BoxError> {
        // Only GET requests are supported for JWKS endpoint
        if req.method() != Method::GET {
            return Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header("Content-Type", "application/json")
                .body(full_body(r#"{"error":"Method not allowed"}"#))
                .map_err(|e| {
                    BoxError::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to build method not allowed response: {}", e),
                    ))
                });
        }

        // Get the JWKS response
        match self.jwks_manager.get_jwks() {
            Ok(jwks_json) => Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(full_body(jwks_json))
                .map_err(|e| {
                    BoxError::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to build JWKS response: {}", e),
                    ))
                }),
            Err(e) => Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .header("Content-Type", "application/json")
                .body(full_body(format!(
                    r#"{{"error":"Failed to generate JWKS: {e}"}}"#
                )))
                .map_err(|e| {
                    BoxError::from(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Failed to build error response: {}", e),
                    ))
                }),
        }
    }
}
