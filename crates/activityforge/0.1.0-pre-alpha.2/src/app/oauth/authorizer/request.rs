use serde::{Deserialize, Serialize};

use activitystreams_vocabulary::{field_access, impl_default, impl_display};

use crate::app::oauth::{CodeChallenge, NormalizedParameter};
use crate::db::{Iri, Uuid};
use crate::{Error, Result, util};

/// Represents OAuth-2.0 authorization response types.
///
/// See [RFC 6749, Sections 3.1.1](https://www.rfc-editor.org/rfc/rfc6749) for details.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum ResponseType {
    Code,
    Token,
}

impl ResponseType {
    /// String representation of the [Code](Self::Code) variant.
    pub const CODE: &str = "code";
    /// String representation of the [Token](Self::Token) variant.
    pub const TOKEN: &str = "token";

    /// Creates a new [ResponseType].
    #[inline]
    pub const fn new() -> Self {
        Self::Code
    }

    /// Gets the [ResponseType] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Code => Self::CODE,
            Self::Token => Self::TOKEN,
        }
    }
}

impl_default!(ResponseType);
impl_display!(ResponseType, str);

impl From<ResponseType> for &'static str {
    fn from(val: ResponseType) -> Self {
        (&val).into()
    }
}

impl From<&ResponseType> for &'static str {
    fn from(val: &ResponseType) -> Self {
        val.as_str()
    }
}

impl TryFrom<&str> for ResponseType {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        match val {
            Self::CODE => Ok(Self::Code),
            Self::TOKEN => Ok(Self::Token),
            _ => Err(Error::http(format!(
                "oauth: authz: invalid response type: {val}"
            ))),
        }
    }
}

impl std::str::FromStr for ResponseType {
    type Err = Error;

    fn from_str(val: &str) -> Result<Self> {
        val.try_into()
    }
}

/// Represents a OAuth-2.0 authorization request parameters.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct AuthorizationRequest {
    response_type: ResponseType,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    client_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_uri: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    code_challenge: Option<CodeChallenge>,
    #[serde(skip_serializing_if = "Option::is_none")]
    state: Option<String>,
}

impl AuthorizationRequest {
    /// Creates a new [AuthorizationRequest].
    #[inline]
    pub const fn new() -> Self {
        Self {
            response_type: ResponseType::new(),
            client_id: Uuid::nil(),
            redirect_uri: None,
            code_challenge: None,
            state: None,
        }
    }

    /// Encodes the [AuthorizationRequest] as the `x-www-form-urlencoded` format.
    pub fn encode_url(&self) -> String {
        let mut params = NormalizedParameter::new();

        params.insert("response_type", self.response_type().as_str());
        params.insert("client_id", self.client_id().to_string());

        if let Some(uri) = self.redirect_uri() {
            params.insert("redirect_uri", uri.as_str());
        }

        if let Some(code) = self.code_challenge() {
            params.insert("code_challenge", code.code_str());
            params.insert("code_challenge_method", code.method().as_str());
        }

        if let Some(state) = self.state() {
            params.insert("state", state);
        }

        params.to_string()
    }
}

field_access! {
    AuthorizationRequest {
        /// Represents the `response_type` parameter value.
        response_type: ResponseType,
        /// Represents the `client_id` parameter value.
        client_id: Uuid,
    }
}

field_access! {
    AuthorizationRequest {
        /// Represents the optional redirect URI parameter value.
        redirect_uri: option_ref { Iri },
        /// Represents the optional PKCE code challenge parameters.
        code_challenge: option_ref { CodeChallenge },
    }
}

field_access! {
    AuthorizationRequest {
        /// Represents the optional (recommended) `state` parameter value.
        state: option_deref { &str, String },
    }
}

impl_default!(AuthorizationRequest);

impl std::fmt::Display for AuthorizationRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.encode_url())
    }
}
