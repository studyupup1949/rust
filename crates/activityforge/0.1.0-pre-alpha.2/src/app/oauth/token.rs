use std::time::Duration;

use chrono::Utc;
use oauth::primitives::issuer::{IssuedToken, RefreshedToken, TokenType};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use activitystreams_vocabulary::{field_access, impl_default, impl_display};

use crate::app::oauth::{OAuthClaims, Scope, ScopeList};
use crate::crypto::{KeyType, PrivateKey, PublicKey};
use crate::db::{
    DateTime, Db, OptionalDateTime, OptionalI64, OptionalScopeList, OptionalString, Person,
    Transaction, Uuid,
};
use crate::{Error, Result, impl_sql_record, util};

mod request;

pub use request::*;

/// Represents the JWT token type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "oauth_token_type", rename_all = "lowercase")]
pub enum OAuthTokenType {
    /// Represents an initially issued access bearer token.
    Access,
    /// Represents a refreshed access bearer token.
    Refresh,
    /// Represents a client registration bearer token.
    Register,
    /// Represents a generic bearer token.
    Bearer,
}

impl OAuthTokenType {
    /// String representation of the [Access](Self::Access) variant.
    pub const ACCESS: &str = "access";
    /// DB table column of the [Access](Self::Access) variant.
    pub const ACCESS_COLUMN: &str = "token";
    /// String representation of the [Refresh](Self::Refresh) variant.
    pub const REFRESH: &str = "refresh";
    /// DB table column of the [Refresh](Self::Refresh) variant.
    pub const REFRESH_COLUMN: &str = "refresh_token";
    /// String representation of the [Register](Self::Register) variant.
    pub const REGISTER: &str = "register";
    /// DB table column of the [Register](Self::Register) variant.
    pub const REGISTER_COLUMN: &str = "token";
    /// String representation of the [Bearer](Self::Bearer) variant.
    pub const BEARER: &str = "bearer";
    /// DB table column of the [Bearer](Self::Bearer) variant.
    pub const BEARER_COLUMN: &str = "token";

    /// Creates a new [OAuthTokenType].
    #[inline]
    pub const fn new() -> Self {
        Self::Access
    }

    /// Gets the [OAuthTokenType] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Access => Self::ACCESS,
            Self::Refresh => Self::REFRESH,
            Self::Register => Self::REGISTER,
            Self::Bearer => Self::BEARER,
        }
    }

    /// Gets the database column for the [OAuthTokenType].
    #[inline]
    pub const fn db_column(&self) -> &'static str {
        match self {
            Self::Access => Self::ACCESS_COLUMN,
            Self::Refresh => Self::REFRESH_COLUMN,
            Self::Register => Self::REGISTER_COLUMN,
            Self::Bearer => Self::BEARER_COLUMN,
        }
    }
}

impl_default!(OAuthTokenType);
impl_display!(OAuthTokenType, str);

/// Represents an OAuth-2.0 token for scoped access to a resource.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "oauth_token")]
pub struct OAuthToken {
    #[serde(skip)]
    uuid: Uuid,
    #[serde(rename = "access_token")]
    token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    refresh_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    until: Option<DateTime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in: Option<i64>,
    token_type: OAuthTokenType,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<ScopeList>,
    #[serde(skip)]
    grant_id: Uuid,
}

impl OAuthToken {
    /// Creates a new [OAuthToken].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            token: String::new(),
            refresh_token: None,
            until: Some((Utc::now() + Duration::from_hours(1)).into()),
            expires_in: Some(Duration::from_hours(1).as_secs() as i64),
            token_type: OAuthTokenType::new(),
            scope: None,
            grant_id: Uuid::nil(),
        }
    }

    /// Performs checks on record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.token.is_empty() {
            Err(Error::sql("oauth: token: empty token"))
        } else if let Some(t) = self.refresh_token.as_ref()
            && t.is_empty()
        {
            Err(Error::sql("oauth: token: empty refresh token"))
        } else if let Some(until) = self.until
            && until < DateTime::from(Utc::now())
        {
            Err(Error::sql("oauth: token: token is expired"))
        } else {
            Ok(())
        }
    }

    /// Convenience function to create the `Authorization` header value.
    pub fn token_header(&self, token_type: OAuthTokenType) -> Result<String> {
        match token_type {
            OAuthTokenType::Access | OAuthTokenType::Bearer | OAuthTokenType::Register => {
                Ok(format!("Bearer {}", self.token()))
            }
            OAuthTokenType::Refresh => self
                .refresh_token()
                .ok_or(Error::http("oauth: token: missing refresh token"))
                .map(|t| format!("Bearer {t}")),
        }
    }

    /// Attempts to find an [OAuthToken] with the associated access token.
    pub async fn find_by_token(
        db: &Db,
        token: &str,
        token_type: OAuthTokenType,
    ) -> Result<Option<Self>> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let token = Self::find_by_token_tx(&mut dbtx, token, token_type).await?;

        dbtx.commit()
            .await
            .map(|_| token)
            .map_err(|err| Error::db(format!("oauth: token: {err}")))
    }

    /// Attempts to find an [OAuthToken] with the associated access token using a transaction.
    pub async fn find_by_token_tx(
        dbtx: &mut Transaction<'_>,
        token: &str,
        token_type: OAuthTokenType,
    ) -> Result<Option<Self>> {
        let table = Self::TABLE;
        let token_col = token_type.db_column();

        sqlx::query(format!("SELECT * FROM {table} WHERE {token_col} = $1").as_str())
            .bind(token)
            .fetch_optional(&mut **dbtx)
            .await
            .map_err(|err| Error::db(format!("oauth: token: {err}")))
            .and_then(|row| {
                if let Some(row) = row {
                    Self::from_row(&row)
                        .map(Some)
                        .map_err(|err| Error::db(format!("oauth: token: {err}")))
                } else {
                    Ok(None)
                }
            })
    }

    /// Attempts to find an [OAuthToken]'s owner with the associated access token.
    pub async fn find_owner_by_token(
        db: &Db,
        token: &str,
        token_type: OAuthTokenType,
    ) -> Result<Option<Person>> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let owner = Self::find_owner_by_token_tx(&mut dbtx, token, token_type).await?;

        dbtx.commit()
            .await
            .map(|_| owner)
            .map_err(|err| Error::db(format!("oauth: token: {err}")))
    }

    /// Attempts to find an [OAuthToken]'s owner with the associated access token using a transaction.
    pub async fn find_owner_by_token_tx(
        dbtx: &mut Transaction<'_>,
        token: &str,
        token_type: OAuthTokenType,
    ) -> Result<Option<Person>> {
        let table = Self::TABLE;
        let token_col = token_type.db_column();

        sqlx::query(
            format!(
                "SELECT p.* FROM person as p, {table} as t, oauth_grant as g, oauth_client as c
                 WHERE t.{token_col} = $1
                     AND g.uuid = t.grant_id
                     AND c.uuid = g.client_id
                     AND p.uuid = c.owner_id
                "
            )
            .as_str(),
        )
        .bind(token)
        .fetch_optional(&mut **dbtx)
        .await
        .map_err(|err| Error::db(format!("oauth: token: {err}")))
        .and_then(|row| {
            if let Some(row) = row {
                Person::from_row(&row)
                    .map(Some)
                    .map_err(|err| Error::db(format!("oauth: token: {err}")))
            } else {
                Ok(None)
            }
        })
    }

    /// Attempts to get the JWT encoding key for signing a token.
    pub fn encoding_key(key: &PrivateKey) -> Result<jwt::EncodingKey> {
        let pem = key.to_pem()?;

        match key.algorithm() {
            KeyType::Ed25519 => jwt::EncodingKey::from_ed_pem(pem.as_bytes())
                .map_err(|err| Error::http(format!("oauth: token: error parsing key pem: {err}"))),
            KeyType::Ecdsa256 | KeyType::Ecdsa384 => jwt::EncodingKey::from_ec_pem(pem.as_bytes())
                .map_err(|err| Error::http(format!("oauth: token: error parsing key pem: {err}"))),
            KeyType::Rsa2048 => jwt::EncodingKey::from_rsa_pem(pem.as_bytes())
                .map_err(|err| Error::http(format!("oauth: token: error parsing key pem: {err}"))),
            algo => Err(Error::http(format!(
                "oauth: token: unsupported encoding key algorithm: {algo}"
            ))),
        }
    }

    /// Attempts to get the JWT decoding key for verifying a signed token.
    pub fn decoding_key(key: &PublicKey) -> Result<jwt::DecodingKey> {
        let pem = key.to_pem()?;

        match key.algorithm() {
            KeyType::Ed25519 => jwt::DecodingKey::from_ed_pem(pem.as_bytes())
                .map_err(|err| Error::http(format!("oauth: token: error parsing key pem: {err}"))),
            KeyType::Ecdsa256 | KeyType::Ecdsa384 => jwt::DecodingKey::from_ec_pem(pem.as_bytes())
                .map_err(|err| Error::http(format!("oauth: token: error parsing key pem: {err}"))),
            KeyType::Rsa2048 => jwt::DecodingKey::from_rsa_pem(pem.as_bytes())
                .map_err(|err| Error::http(format!("oauth: token: error parsing key pem: {err}"))),
            algo => Err(Error::http(format!(
                "oauth: token: unsupported decoding key algorithm: {algo}"
            ))),
        }
    }

    /// Creates a signed JWT bearer token.
    pub fn sign_token(key: &PrivateKey, claims: &OAuthClaims) -> Result<String> {
        let algo: jwt::Algorithm = key.algorithm().try_into()?;
        let key = Self::encoding_key(key)?;

        let header = jwt::Header::new(algo);

        jwt::encode(&header, claims, &key)
            .map_err(|err| Error::http(format!("oauth: token: error signing token: {err}")))
    }

    /// Verifies a signed JWT bearer token, and extracts the claims.
    pub fn verify_token(key: &PublicKey, token: &str) -> Result<OAuthClaims> {
        let algo: jwt::Algorithm = key.algorithm().try_into()?;
        let key = Self::decoding_key(key)?;

        let val = jwt::Validation::new(algo);

        let data = jwt::decode::<OAuthClaims>(&token, &key, &val)
            .map_err(|err| Error::http(format!("oauth: token: error verifying token: {err}")))?;

        let claims = data.claims;
        if claims.is_expired() {
            Err(Error::http(format!(
                "oauth: token: token expired: {claims}"
            )))
        } else {
            Ok(claims)
        }
    }
}

field_access! {
    OAuthToken {
        /// Represents the [Uuid] primary key of the table entry.
        uuid: Uuid,
        /// Represents whether the [OAuthToken] is a refresh token.
        token_type: OAuthTokenType,
        /// References the [OAuthGrant](crate::app::oauth::Grant) for the [OAuthToken].
        grant_id: Uuid,
    }
}

field_access! {
    OAuthToken {
        /// Represents the [OAuthToken] JWT token (encoded).
        token: as_ref { &str, String },
    }
}

field_access! {
    OAuthToken {
        /// Represents the [OAuthToken] JWT refresh token (encoded).
        refresh_token: option_deref { &str, String },
        /// Represents the [OAuthToken] JWT list of scopes.
        scope: option_deref { &[Scope], ScopeList },
    }
}

field_access! {
    OAuthToken {
        /// Represents the time the [OAuthToken] is valid until.
        until: option { DateTime },
        /// Represents the time (in seconds) until the token expires.
        expires_in: option { i64 },
    }
}

impl_sql_record! {
    OAuthToken {
        token: { "token" String },
        refresh_token: { "refresh_token" OptionalString },
        until: { "until" OptionalDateTime },
        expires_in: { "expires_in" OptionalI64 },
        token_type: { "token_type" OAuthTokenType },
        scope: { "scope" OptionalScopeList },
        grant_id: { "grant_id" Uuid },
    }
}

impl_default!(OAuthToken);
impl_display!(OAuthToken, json);

impl TryFrom<OAuthToken> for IssuedToken {
    type Error = Error;

    fn try_from(val: OAuthToken) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&OAuthToken> for IssuedToken {
    type Error = Error;

    fn try_from(val: &OAuthToken) -> Result<Self> {
        match val.token_type() {
            OAuthTokenType::Access => Ok(Self {
                token: val.token().into(),
                refresh: val.refresh_token().map(|s| s.to_owned()),
                until: val.until().map(|d| d.into()).unwrap_or_default(),
                token_type: TokenType::Bearer,
            }),
            token_ty => Err(Error::http(format!(
                "oauth: token: invalid token type: {token_ty}"
            ))),
        }
    }
}

impl TryFrom<OAuthToken> for RefreshedToken {
    type Error = Error;

    fn try_from(val: OAuthToken) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&OAuthToken> for RefreshedToken {
    type Error = Error;

    fn try_from(val: &OAuthToken) -> Result<Self> {
        match val.token_type() {
            OAuthTokenType::Refresh => Ok(Self {
                token: val.token().into(),
                refresh: val.refresh_token().map(|s| s.to_owned()),
                until: val.until().map(|d| d.into()).unwrap_or_default(),
                token_type: TokenType::Bearer,
            }),
            token_ty => Err(Error::http(format!(
                "oauth: token: invalid token type: {token_ty}"
            ))),
        }
    }
}
