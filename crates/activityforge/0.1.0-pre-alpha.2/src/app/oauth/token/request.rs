use serde::{Deserialize, Serialize};

use activitystreams_vocabulary::{field_access, impl_default};

use crate::app::oauth::{NormalizedParameter, OAuthGrantType, Scope, ScopeList};
use crate::db::Iri;

/// Represents a request to retrieve an OAuth-2.0 access token.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TokenRequest {
    grant_type: OAuthGrantType,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    client_secret: Option<String>,
    code: String,
    redirect_uri: Iri,
    #[serde(skip_serializing_if = "Option::is_none")]
    code_verifier: Option<String>,
}

impl TokenRequest {
    /// Creates a new [TokenRequest].
    pub const fn new() -> Self {
        Self {
            grant_type: OAuthGrantType::new(),
            client_id: None,
            client_secret: None,
            code: String::new(),
            redirect_uri: Iri::new(),
            code_verifier: None,
        }
    }

    /// Encodes the [TokenRequest] as the `x-www-form-urlencoded` format.
    pub fn encode_url(&self) -> String {
        NormalizedParameter::from(self).to_string()
    }
}

impl From<TokenRequest> for NormalizedParameter {
    fn from(val: TokenRequest) -> Self {
        (&val).into()
    }
}

impl From<&TokenRequest> for NormalizedParameter {
    fn from(val: &TokenRequest) -> Self {
        let mut params = NormalizedParameter::new();

        params.insert("grant_type", val.grant_type().as_str());
        params.insert("code", val.code());
        params.insert("redirect_uri", val.redirect_uri().as_str());

        if let Some(v) = val.client_id() {
            params.insert("client_id", v);
        }

        if let Some(v) = val.client_secret() {
            params.insert("client_secret", v);
        }

        if let Some(v) = val.code_verifier() {
            params.insert("code_verifier", v);
        }

        params
    }
}

field_access! {
    TokenRequest {
        /// Represents the OAuth-2.0 grant type of the requested token.
        grant_type: OAuthGrantType,
    }
}

field_access! {
    TokenRequest {
        /// Represents the OAuth-2.0 authization code.
        code: as_ref { &str, String },
    }
}

field_access! {
    TokenRequest {
        /// Represents the OAuth-2.0 redirect URI.
        redirect_uri: as_ref { Iri },
    }
}

field_access! {
    TokenRequest {
        /// Represents the OAuth-2.0 client ID requesting the token.
        client_id: option_deref { &str, String },
        /// Represents the OAuth-2.0 client secret (not recommended).
        client_secret: option_deref { &str, String },
        /// Represents the OAuth-2.0 PKCE verifier code.
        code_verifier: option_deref { &str, String },
    }
}

impl_default!(TokenRequest);

impl std::fmt::Display for TokenRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.encode_url())
    }
}

/// Represents a request to refresh an OAuth-2.0 access token.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct RefreshTokenRequest {
    grant_type: OAuthGrantType,
    refresh_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<ScopeList>,
}

impl RefreshTokenRequest {
    /// Creates a new [RefreshTokenRequest].
    pub const fn new() -> Self {
        Self {
            grant_type: OAuthGrantType::RefreshToken,
            refresh_token: String::new(),
            scope: None,
        }
    }

    /// Encodes the [RefreshTokenRequest] as the `x-www-form-urlencoded` format.
    pub fn encode_url(&self) -> String {
        NormalizedParameter::from(self).to_string()
    }
}

impl From<RefreshTokenRequest> for NormalizedParameter {
    fn from(val: RefreshTokenRequest) -> Self {
        (&val).into()
    }
}

impl From<&RefreshTokenRequest> for NormalizedParameter {
    fn from(val: &RefreshTokenRequest) -> Self {
        let mut params = NormalizedParameter::new();

        params.insert("grant_type", val.grant_type().as_str());
        params.insert("refresh_token", val.refresh_token());

        if let Some(v) = val.scope.as_ref().map(|s| s.encode_url()) {
            params.insert("scope", v);
        }

        params
    }
}

field_access! {
    RefreshTokenRequest {
        /// Represents the OAuth-2.0 grant type of the requested token.
        grant_type: OAuthGrantType,
    }
}

field_access! {
    RefreshTokenRequest {
        /// Represents the OAuth-2.0 authization code.
        refresh_token: as_ref { &str, String },
    }
}

field_access! {
    RefreshTokenRequest {
        /// Represents the OAuth-2.0 scope list for the refresh token.
        ///
        /// If present, the scope parameter cannot contain any scope not included in the original token grant.
        scope: option_deref { &[Scope], ScopeList },
    }
}

impl_default!(RefreshTokenRequest);

impl std::fmt::Display for RefreshTokenRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.encode_url())
    }
}
