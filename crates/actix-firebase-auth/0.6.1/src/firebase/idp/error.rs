//! Error types related to Firebase Identity Provider (`IdP`) claim extraction.

use actix_web::{http::StatusCode, HttpResponse, ResponseError};
use thiserror::Error;

/// Represents an error encountered when working with Firebase Identity Provider (`IdP`) claims.
///
/// This error type is typically used when extracting provider-specific identity information
/// (e.g., a Google user ID from the `firebase.identities` map in a Firebase ID token).
#[derive(Debug, Error)]
pub enum FirebaseIdpError {
    /// Occurs when the expected identity provider claims are missing from the Firebase ID token.
    ///
    /// For example, if a token was issued via "google.com" login, but no corresponding
    /// Google user ID is found in the `firebase.identities` field, this variant is returned.
    ///
    /// # Example
    /// A decoded token's `firebase.identities` might look like:
    /// ```json
    /// {
    ///     "google.com": ["123456789012345678901"]
    /// }
    /// ```
    /// If the "google.com" key is missing or contains an invalid value, this error is returned.
    ///
    /// # Fields
    /// - `provider`: The expected identity provider (e.g., `"google.com"`, `"github.com"`).
    #[error(
        "Firebase ID token is missing `{provider}` identity provider claims"
    )]
    MissingIdpClaims {
        /// The Firebase identity provider that was expected but not found in the token.
        provider: &'static str,
    },
}

impl ResponseError for FirebaseIdpError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            FirebaseIdpError::MissingIdpClaims { .. } => {
                // Server failed to parse Firebase IdP claims
                StatusCode::UNAUTHORIZED
            }
        }
    }
}
