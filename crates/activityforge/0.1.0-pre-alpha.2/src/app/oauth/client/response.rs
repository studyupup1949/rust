use serde::{Deserialize, Serialize};

use activitystreams_vocabulary::{field_access, impl_default, impl_display};

use crate::app::oauth::{ClientSecret, EndpointAuthMethod, OAuthGrantType};
use crate::db::{Iri, Uuid};
use crate::util;

/// Represents an OAuth-2.0 "Client Information Response".
///
/// See [RFC 7581, Section 3.2.1](https://www.rfc-editor.org/rfc/rfc7591) for details.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OAuthClientResponse {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    client_id: Uuid,
    client_secret: ClientSecret,
    client_id_issued_at: u64,
    client_secret_expires_at: u64,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    redirect_uris: Vec<Iri>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    grant_types: Vec<OAuthGrantType>,
    token_endpoint_auth_method: EndpointAuthMethod,
}

impl OAuthClientResponse {
    /// Creates a new [OAuthClientResponse].
    #[inline]
    pub const fn new() -> Self {
        Self {
            client_id: Uuid::nil(),
            client_secret: ClientSecret::new(),
            client_id_issued_at: 0,
            client_secret_expires_at: 0,
            redirect_uris: Vec::new(),
            grant_types: Vec::new(),
            token_endpoint_auth_method: EndpointAuthMethod::new(),
        }
    }
}

field_access! {
    OAuthClientResponse {
        /// Represents the OAuth-2.0 `client_id` issued by the server.
        client_id: Uuid,
        /// Represents the timestamp when the `client_id` was issued.
        client_id_issued_at: u64,
        /// Represents the expiration timestamp of the `client_secret`.
        ///
        /// **NOTE**: will be `0` if the `client_secret` does not expire.
        client_secret_expires_at: u64,
        /// Represents the OAuth-2.0 client authentication method.
        token_endpoint_auth_method: EndpointAuthMethod,
    }
}

field_access! {
    OAuthClientResponse {
        /// Represents the `client_secret` used to authenticate the client.
        client_secret: as_ref { ClientSecret },
    }
}

field_access! {
    OAuthClientResponse {
        /// Represents the list of `redirect_uri` URIs for OAuth-2.0 callbacks.
        redirect_uris: as_ref { &[Iri], Vec<Iri> },
        /// Represents the list of allowable grant types.
        grant_types: as_ref { &[OAuthGrantType], Vec<OAuthGrantType> },
    }
}

impl_default!(OAuthClientResponse);
impl_display!(OAuthClientResponse, json);
