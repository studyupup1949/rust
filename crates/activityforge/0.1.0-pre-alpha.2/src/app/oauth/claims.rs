use std::time::Duration;

use chrono::Utc;
use oauth::primitives::grant::Grant;
use serde::{Deserialize, Serialize};

use activitystreams_vocabulary::{field_access, impl_default, impl_display};

use crate::crypto::Nonce;
use crate::db::DateTime;
use crate::{Error, Result};

/// Represents the OAuth-2.0 JWT claims.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OAuthClaims {
    #[serde(rename = "iss")]
    issuer: String,
    #[serde(rename = "sub")]
    subject: String,
    #[serde(rename = "exp")]
    expires: u64,
    nonce: Nonce,
}

impl OAuthClaims {
    /// Represents the default [OAuthClaims] issuer.
    pub const ISSUER: &str = "activityforge";
    /// Represents the [OAuthClaims] authentication subject prefix.
    pub const AUTHENTICATE_SUBJECT_PREFIX: &str = "authenticate:";

    /// Creates a new [OAuthClaims].
    pub fn new() -> Self {
        Self {
            issuer: Self::ISSUER.into(),
            subject: String::new(),
            expires: (Utc::now() + Duration::from_hours(1)).timestamp() as u64,
            nonce: Nonce::random(),
        }
    }

    /// Gets whether the [OAuthClaims] has expired.
    pub fn is_expired(&self) -> bool {
        chrono::DateTime::from_timestamp(self.expires as i64, 0).unwrap_or_default()
            < DateTime::from(Utc::now())
    }

    /// Gets the authentication subject.
    pub fn authenticate_subject(&self) -> Result<&str> {
        self.subject
            .strip_prefix(Self::AUTHENTICATE_SUBJECT_PREFIX)
            .ok_or(Error::http("oauth: claims: invalid authenticate subject"))
    }

    /// Sets the authentication subject.
    pub fn set_authenticate_subject<S: core::fmt::Display>(&mut self, subject: S) {
        self.subject = format!("{}{subject}", Self::AUTHENTICATE_SUBJECT_PREFIX);
    }

    /// Builder function that sets the authentication subject.
    pub fn with_authenticate_subject<S: core::fmt::Display>(self, subject: S) -> Self {
        Self {
            subject: format!("{}{subject}", Self::AUTHENTICATE_SUBJECT_PREFIX),
            ..self
        }
    }
}

field_access! {
    OAuthClaims {
        /// Represents the issuer of the JWT token, typically `activityforge`.
        issuer: as_ref { &str, String },
        /// Represents the subject (user) of the JWT token.
        subject: as_ref { &str, String },
    }
}

field_access! {
    OAuthClaims {
        /// Represents the nonce used to sign the JWT token.
        nonce: as_ref { Nonce },
    }
}

field_access! {
    OAuthClaims {
        /// Represents the JWT expiration time.
        expires: u64,
    }
}

impl_default!(OAuthClaims);
impl_display!(OAuthClaims, json);

impl From<Grant> for OAuthClaims {
    fn from(val: Grant) -> Self {
        (&val).into()
    }
}

impl From<&Grant> for OAuthClaims {
    fn from(val: &Grant) -> Self {
        Self::new()
            .with_subject(val.owner_id.as_str())
            .with_expires(val.until.timestamp() as u64)
    }
}
