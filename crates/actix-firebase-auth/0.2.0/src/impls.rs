use actix_web::error::InternalError;
use actix_web::http::{StatusCode, header};
use actix_web::{FromRequest, HttpRequest, dev, http::header::Header, web};
use actix_web::{HttpResponse, ResponseError};
use actix_web_httpauth::headers::authorization::{Authorization, Bearer};
use futures::future::{Ready, err, ok};

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

impl FromRequest for FirebaseUser {
    type Error = actix_web::Error;
    type Future = Ready<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut dev::Payload) -> Self::Future {
        let firebase_auth = req
            .app_data::<web::Data<FirebaseAuth>>()
            .expect("FirebaseAuth should be initialized in application data");

        let bearer = match Authorization::<Bearer>::parse(req) {
            Ok(header) => header.into_scheme(),
            Err(_) => {
                // Per RFC 7235, a 401 Unauthorized response MUST be returned when the
                // Authorization header is missing, malformed, or uses an unsupported scheme.
                //
                // Actix defaults to 400 Bad Request for parsing failures, which is incorrect
                // in the context of authentication. We explicitly return 401 and include a
                // WWW-Authenticate header to guide the client on how to authenticate.
                return err(missing_or_malformed_auth_header());
            }
        };

        let id_token = bearer.token();

        match firebase_auth.verify(id_token) {
            Ok(user) => ok(user),
            Err(crate::Error::VerificationError(VerificationError::CannotDecodePublicKeys)) => {
                err(internal_token_verification_error())
            }
            Err(other) => err(invalid_token_error(&other)),
        }
    }
}

fn internal_token_verification_error() -> actix_web::Error {
    let response =
        HttpResponse::InternalServerError().body("Internal error during token verification");

    InternalError::from_response("token_verification_failure", response).into()
}

fn missing_or_malformed_auth_header() -> actix_web::Error {
    unauthorized_with_www_authenticate(
        "invalid_request",
        "Authorization header missing or not using Bearer scheme",
        "Authorization header is missing or malformed",
    )
}

fn invalid_token_error(err: &crate::Error) -> actix_web::Error {
    unauthorized_with_www_authenticate(
        "invalid_token",
        &err.to_string(),
        format!("Failed to verify Firebase ID token: {}", err),
    )
}

/// Constructs a generic `actix_web::Error` with a `WWW-Authenticate` header if needed.
fn unauthorized_with_www_authenticate(
    www_error_code: &str,
    www_error_description: &str,
    body: impl Into<String>,
) -> actix_web::Error {
    let header_value = format!(
        r#"Bearer realm="firebase", error="{}", error_description="{}""#,
        www_error_code, www_error_description
    );

    let response = HttpResponse::Unauthorized()
        .insert_header((header::WWW_AUTHENTICATE, header_value))
        .body(body.into());

    InternalError::from_response("auth_error", response).into()
}
