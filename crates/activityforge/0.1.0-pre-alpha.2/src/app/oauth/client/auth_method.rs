use serde::{Deserialize, Serialize};

use activitystreams_vocabulary::{impl_default, impl_display};

/// Represents the OAuth-2.0 authentication methods.
///
/// See [RFC 7591, Section 2](https://www.rfc-editor.org/rfc/rfc7591) for details.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointAuthMethod {
    None,
    ClientSecretPost,
    ClientSecretBasic,
}

impl EndpointAuthMethod {
    /// String representation of the [None](Self::None) variant.
    pub const NONE: &str = "none";
    /// String representation of the [ClientSecretPost](Self::ClientSecretPost) variant.
    pub const CLIENT_SECRET_POST: &str = "client_secret_post";
    /// String representation of the [ClientSecretBasic](Self::ClientSecretBasic) variant.
    pub const CLIENT_SECRET_BASIC: &str = "client_secret_basic";

    /// Creates a new [EndpointAuthMethod].
    #[inline]
    pub const fn new() -> Self {
        Self::ClientSecretBasic
    }

    /// Gets the [EndpointAuthMethod] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => Self::NONE,
            Self::ClientSecretPost => Self::CLIENT_SECRET_POST,
            Self::ClientSecretBasic => Self::CLIENT_SECRET_BASIC,
        }
    }
}

impl_default!(EndpointAuthMethod);
impl_display!(EndpointAuthMethod, str);
