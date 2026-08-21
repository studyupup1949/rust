use std::str::FromStr;

use http::status::StatusCode;
use serde::{Deserialize, Serialize};

use activitystreams_vocabulary::{impl_default, impl_display};

use crate::{Error, Result};

/// Represents the OAuth-2.0 error response codes.
///
/// See [RFC 6749, Sections 4.1.2.1 + 4.2.2.1](https://www.rfc-editor.org/rfc/rfc6749) for details.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuthError {
    InvalidRequest,
    UnauthorizedClient,
    AccessDenied,
    UnsupportedResponseType,
    InvalidScope,
    ServerError,
    TemporarilyUnavailable,
}

impl OAuthError {
    /// String representation of the [InvalidRequest](Self::InvalidRequest) variant.
    pub const INVALID_REQUEST: &str = "invalid_request";
    /// String representation of the [UnauthorizedClient](Self::UnauthorizedClient) variant.
    pub const UNAUTHORIZED_CLIENT: &str = "unauthorized_client";
    /// String representation of the [AccessDenied](Self::AccessDenied) variant.
    pub const ACCESS_DENIED: &str = "access_denied";
    /// String representation of the [UnsupportedResponseType](Self::UnsupportedResponseType) variant.
    pub const UNSUPPORTED_RESPONSE_TYPE: &str = "unsupported_response_type";
    /// String representation of the [InvalidScope](Self::InvalidScope) variant.
    pub const INVALID_SCOPE: &str = "invalid_scope";
    /// String representation of the [ServerError](Self::ServerError) variant.
    pub const SERVER_ERROR: &str = "server_error";
    /// String representation of the [TemporarilyUnavailable](Self::TemporarilyUnavailable) variant.
    pub const TEMPORARILY_UNAVAILABLE: &str = "temporarily_unavailable";

    /// Creates a new [OAuthError].
    #[inline]
    pub const fn new() -> Self {
        Self::InvalidRequest
    }

    /// Gets the [OAuthError] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::InvalidRequest => Self::INVALID_REQUEST,
            Self::UnauthorizedClient => Self::UNAUTHORIZED_CLIENT,
            Self::AccessDenied => Self::ACCESS_DENIED,
            Self::UnsupportedResponseType => Self::UNSUPPORTED_RESPONSE_TYPE,
            Self::InvalidScope => Self::INVALID_SCOPE,
            Self::ServerError => Self::SERVER_ERROR,
            Self::TemporarilyUnavailable => Self::TEMPORARILY_UNAVAILABLE,
        }
    }

    /// Converts the [OAuthError] to a HTTP status code.
    #[inline]
    pub const fn status(&self) -> StatusCode {
        match self {
            Self::InvalidRequest => StatusCode::BAD_REQUEST,
            Self::UnauthorizedClient => StatusCode::UNAUTHORIZED,
            Self::AccessDenied => StatusCode::FORBIDDEN,
            Self::UnsupportedResponseType => StatusCode::NOT_IMPLEMENTED,
            Self::InvalidScope => StatusCode::BAD_REQUEST,
            Self::ServerError => StatusCode::INTERNAL_SERVER_ERROR,
            Self::TemporarilyUnavailable => StatusCode::SERVICE_UNAVAILABLE,
        }
    }
}

impl_default!(OAuthError);
impl_display!(OAuthError, str);

impl From<OAuthError> for &'static str {
    fn from(val: OAuthError) -> Self {
        (&val).into()
    }
}

impl From<&OAuthError> for &'static str {
    fn from(val: &OAuthError) -> Self {
        val.as_str()
    }
}

impl TryFrom<&str> for OAuthError {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        match val {
            Self::INVALID_REQUEST => Ok(Self::InvalidRequest),
            Self::UNAUTHORIZED_CLIENT => Ok(Self::UnauthorizedClient),
            Self::ACCESS_DENIED => Ok(Self::AccessDenied),
            Self::UNSUPPORTED_RESPONSE_TYPE => Ok(Self::UnsupportedResponseType),
            Self::INVALID_SCOPE => Ok(Self::InvalidScope),
            Self::SERVER_ERROR => Ok(Self::ServerError),
            Self::TEMPORARILY_UNAVAILABLE => Ok(Self::TemporarilyUnavailable),
            _ => Err(Error::http("oauth: invalid error code: {val}")),
        }
    }
}

impl FromStr for OAuthError {
    type Err = Error;

    fn from_str(val: &str) -> Result<Self> {
        val.try_into()
    }
}

impl From<OAuthError> for StatusCode {
    fn from(val: OAuthError) -> Self {
        (&val).into()
    }
}

impl From<&OAuthError> for StatusCode {
    fn from(val: &OAuthError) -> Self {
        val.status()
    }
}
