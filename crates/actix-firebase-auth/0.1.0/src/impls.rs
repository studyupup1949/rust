use actix_web::error::ErrorUnauthorized;
use actix_web::http::StatusCode;
use actix_web::{FromRequest, HttpRequest, dev, http::header::Header, web};
use actix_web::{HttpResponse, ResponseError};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use futures::future::{Ready, err, ok};
use tracing::debug;

use crate::jwk::{PublicKeysError, VerificationError};
use crate::{Error, FirebaseAuth, FirebaseUser};

fn status_code_from_http_err(err: &reqwest::Error) -> StatusCode {
    let code = err.status().map(|s| s.as_u16()).unwrap_or_else(|| 500);
    StatusCode::from_u16(code).unwrap_or(StatusCode::BAD_GATEWAY) // Use BAD_GATEWAY for upstream fetch failures
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match self {
            Error::PublicKeysError(err) => match err {
                PublicKeysError::FetchPublicKeys(http_err)
                | PublicKeysError::PublicKeyParseError(http_err) => {
                    status_code_from_http_err(http_err)
                }

                PublicKeysError::MissingCacheControlHeader
                | PublicKeysError::MissingMaxAgeDirective
                | PublicKeysError::EmptyMaxAgeDirective
                | PublicKeysError::InvalidMaxAgeValue => {
                    // Indicates a misconfigured or invalid response from the identity provider
                    StatusCode::INTERNAL_SERVER_ERROR
                }
            },

            Error::VerificationError(err) => match err {
                VerificationError::InvalidSignature => {
                    // Token is invalid or tampered with
                    StatusCode::UNAUTHORIZED
                }
                VerificationError::InvalidKeyAlgorithm => {
                    // Token uses unsupported algorithm – client bug or attacker
                    StatusCode::BAD_REQUEST
                }
                VerificationError::InvalidToken => {
                    // Token is malformed or structurally invalid
                    StatusCode::BAD_REQUEST
                }
                VerificationError::NoKidHeader => {
                    // Token doesn't specify which key was used to sign – malformed
                    StatusCode::BAD_REQUEST
                }
                VerificationError::NoMatchingKid => {
                    // Token specifies a `kid` for which we have no key – likely expired key
                    StatusCode::UNAUTHORIZED
                }
                VerificationError::CannotDecodePublicKeys => {
                    // Server failed to decode key set
                    StatusCode::INTERNAL_SERVER_ERROR
                }
                VerificationError::CannotDecodeJwt(_) => {
                    // Server failed to decode key set
                    StatusCode::UNAUTHORIZED
                }
            },
        }
    }
}

fn get_bearer_token(header: &str) -> Option<String> {
    let prefix_len = "Bearer ".len();

    match header.len() {
        l if l < prefix_len => None,
        _ => Some(header[prefix_len..].to_string()),
    }
}

impl FromRequest for FirebaseUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let firebase_auth = req
            .app_data::<web::Data<FirebaseAuth>>()
            .expect("must initialize FirebaseAuth in Application Data");

        let bearer = match Authorization::<Bearer>::parse(req) {
            Err(e) => return err(e.into()),
            Ok(v) => get_bearer_token(&v.to_string()).unwrap_or_default(),
        };

        debug!("Got bearer token {}", bearer);

        match firebase_auth.verify(&bearer) {
            Err(e) => err(ErrorUnauthorized(format!("Failed to verify Token {}", e))),
            Ok(user) => ok(user),
        }
    }
}
