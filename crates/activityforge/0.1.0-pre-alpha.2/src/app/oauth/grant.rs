use base64::Encoding;
use hmac::Mac;
use oauth::primitives::grant::Grant;
use oauth::primitives::scope::Scope as OAuthScope;
use serde::{Deserialize, Serialize};
use sha3::{Digest, Sha3_256};
use sqlx::FromRow;

use activitystreams_vocabulary::{field_access, impl_default, impl_display};

use crate::app::oauth::{OAuthTokenType, Scope, ScopeList};
use crate::crypto::{HmacSha3_256, Nonce};
use crate::db::{DateTime, Db, Iri, Transaction, Uuid};
use crate::{Error, Result, impl_sql_find_or_create, impl_sql_list_field, impl_sql_record, util};

/// Represents the types of OAuth-2.0 client grants.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(rename_all = "snake_case", type_name = "oauth_grant_type")]
pub enum OAuthGrantType {
    AuthorizationCode,
    RefreshToken,
}

impl OAuthGrantType {
    /// String reprsentation of the [AuthorizationCode](Self::AuthorizationCode) variant.
    pub const AUTHORIZATION_CODE: &str = "authorization_code";
    /// String reprsentation of the [RefreshToken](Self::RefreshToken) variant.
    pub const REFRESH_TOKEN: &str = "refresh_token";

    /// Creates a new [OAuthGrantType].
    #[inline]
    pub const fn new() -> Self {
        Self::AuthorizationCode
    }

    /// Gets the [OAuthGrantType] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::AuthorizationCode => Self::AUTHORIZATION_CODE,
            Self::RefreshToken => Self::REFRESH_TOKEN,
        }
    }
}

impl_default!(OAuthGrantType);
impl_display!(OAuthGrantType, str);

/// Represents an OAuth-2.0 grant for scoped access to a resource.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "oauth_grant")]
pub struct OAuthGrant {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    owner_id: Iri,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    client_id: Uuid,
    #[serde(skip_serializing_if = "ScopeList::is_empty")]
    scopes: ScopeList,
    redirect_uri: Iri,
    until: DateTime,
    pkce: String,
    tag: String,
}

impl OAuthGrant {
    /// Creates a new [OAuthGrant].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            owner_id: Iri::new(),
            client_id: Uuid::nil(),
            scopes: ScopeList::new(),
            redirect_uri: Iri::new(),
            until: chrono::Utc::now().into(),
            pkce: String::new(),
            tag: String::new(),
        }
    }

    /// Creates a cryptographic tag for the [OAuthGrant].
    pub fn create_tag(key: &[u8]) -> Result<String> {
        let nonce = Nonce::random();

        let key = Sha3_256::digest(key);

        let mut hmac = HmacSha3_256::new_from_slice(key.as_slice())
            .map_err(|err| Error::crypto(format!("oauth: grant: error deriving tag key: {err}")))?;

        hmac.update(nonce.as_ref());
        let sig = hmac.finalize();

        Ok(base64::Base64UrlUnpadded::encode_string(
            sig.into_bytes().as_ref(),
        ))
    }

    /// Performs checks on record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.owner_id.is_empty() {
            Err(Error::sql("oauth: grant: empty owner ID"))
        } else if self.client_id.is_nil() {
            Err(Error::sql("oauth: grant: empty client ID"))
        } else if self.scopes.is_empty() {
            Err(Error::sql("oauth: grant: empty scopes"))
        } else if self.redirect_uri.is_empty() {
            Err(Error::sql("oauth: grant: empty redirect URI"))
        } else {
            Ok(())
        }
    }

    /// Attempts to find an [OAuthGrant] by owner ID.
    pub async fn find_by_owner_id(db: &Db, id: &Iri) -> Result<Option<Self>> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let actor = Self::find_by_owner_id_tx(&mut dbtx, id).await?;

        dbtx.commit()
            .await
            .map(|_| actor)
            .map_err(|err| Error::db(format!("oauth: grant: {err}")))
    }

    /// Attempts to find an [OAuthGrant] by owner ID using a transaction.
    pub async fn find_by_owner_id_tx(dbtx: &mut Transaction<'_>, id: &Iri) -> Result<Option<Self>> {
        let table = Self::TABLE;

        sqlx::query(format!("SELECT * FROM {table} WHERE owner_id = $1").as_str())
            .bind(id)
            .fetch_optional(&mut **dbtx)
            .await
            .map_err(|err| Error::db(format!("oauth: grant: {err}")))
            .and_then(|row| {
                if let Some(row) = row {
                    Self::from_row(&row)
                        .map(Some)
                        .map_err(|err| Error::db(format!("oauth: grant: {err}")))
                } else {
                    Ok(None)
                }
            })
    }

    /// Attempts to find an [OAuthGrant] by client ID.
    pub async fn find_by_client_id(db: &Db, id: &Uuid) -> Result<Option<Self>> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let actor = Self::find_by_client_id_tx(&mut dbtx, id).await?;

        dbtx.commit()
            .await
            .map(|_| actor)
            .map_err(|err| Error::db(format!("oauth: grant: {err}")))
    }

    /// Attempts to find an [OAuthGrant] by client ID using a transaction.
    pub async fn find_by_client_id_tx(
        dbtx: &mut Transaction<'_>,
        id: &Uuid,
    ) -> Result<Option<Self>> {
        let table = Self::TABLE;

        sqlx::query(format!("SELECT * FROM {table} WHERE client_id = $1").as_str())
            .bind(id)
            .fetch_optional(&mut **dbtx)
            .await
            .map_err(|err| Error::db(format!("oauth: grant: {err}")))
            .and_then(|row| {
                if let Some(row) = row {
                    Self::from_row(&row)
                        .map(Some)
                        .map_err(|err| Error::db(format!("oauth: grant: {err}")))
                } else {
                    Ok(None)
                }
            })
    }

    /// Attempts to find an [OAuthGrant] by its associated `tag`.
    pub async fn find_by_tag(db: &Db, tag: &str) -> Result<Option<Self>> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let grant = Self::find_by_tag_tx(&mut dbtx, tag).await?;

        dbtx.commit()
            .await
            .map(|_| grant)
            .map_err(|err| Error::db(format!("oauth: grant: {err}")))
    }

    /// Attempts to find an [OAuthGrant] by its associated `tag` using a transaction.
    pub async fn find_by_tag_tx(dbtx: &mut Transaction<'_>, tag: &str) -> Result<Option<Self>> {
        let table = Self::TABLE;

        sqlx::query(format!("SELECT * FROM {table} WHERE tag = $1").as_str())
            .bind(tag)
            .fetch_optional(&mut **dbtx)
            .await
            .map_err(|err| Error::db(format!("oauth: grant: {err}")))
            .and_then(|row| {
                if let Some(row) = row {
                    Self::from_row(&row)
                        .map(Some)
                        .map_err(|err| Error::db(format!("oauth: grant: {err}")))
                } else {
                    Ok(None)
                }
            })
    }

    /// Attempts to find an [OAuthGrant] by its associated `token`.
    pub async fn find_by_token(
        db: &Db,
        token: &str,
        token_type: OAuthTokenType,
    ) -> Result<Option<Self>> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let grant = Self::find_by_token_tx(&mut dbtx, token, token_type).await?;

        dbtx.commit()
            .await
            .map(|_| grant)
            .map_err(|err| Error::db(format!("oauth: grant: {err}")))
    }

    /// Attempts to find an [OAuthGrant] by its associated `token` using a transaction.
    pub async fn find_by_token_tx(
        dbtx: &mut Transaction<'_>,
        token: &str,
        token_type: OAuthTokenType,
    ) -> Result<Option<Self>> {
        let table = Self::TABLE;
        let token_col = token_type.db_column();

        sqlx::query(
            format!(
                "SELECT g.* FROM {table} as g, oauth_token as t
                 WHERE t.{token_col} = $1 AND g.uuid = t.grant_id"
            )
            .as_str(),
        )
        .bind(token)
        .fetch_optional(&mut **dbtx)
        .await
        .map_err(|err| Error::db(format!("oauth: grant: {err}")))
        .and_then(|row| {
            if let Some(row) = row {
                Self::from_row(&row)
                    .map(Some)
                    .map_err(|err| Error::db(format!("oauth: grant: {err}")))
            } else {
                Ok(None)
            }
        })
    }
}

impl_default!(OAuthGrant);
impl_display!(OAuthGrant, json);

field_access! {
    OAuthGrant {
        /// Represents the [Uuid] primary key of the table entry.
        uuid: Uuid,
        /// Represents the [Uuid] of the {OAuthClient](super::OAuthClient] given the [OAuthGrant].
        client_id: Uuid,
        /// Represents the time the [OAuthGrant] is valid until.
        until: DateTime,
    }
}

field_access! {
    OAuthGrant {
        /// Represents the IRI of the [OAuthGrant] owner ID.
        owner_id: as_ref { Iri },
        /// Represents the IRI of the [OAuthGrant] redirects the client to after a completed flow.
        redirect_uri: as_ref { Iri },
    }
}

field_access! {
    OAuthGrant {
        /// Represents the PKCE code used during OAuth-2.0 authorization.
        pkce: as_ref { &str, String },
        /// Represents the cryptographic tag used during OAuth-2.0 authorization.
        tag: as_ref { &str, String },
    }
}

impl_sql_record! {
    OAuthGrant {
        owner_id: { "owner_id" Iri },
        client_id: { "client_id" Uuid },
        scopes: { "scopes" ScopeList },
        redirect_uri: { "redirect_uri" Iri },
        until: { "until" DateTime },
        pkce: { "pkce" String },
        tag: { "tag" String },
    }
}

impl_sql_list_field! {
    OAuthGrant {
        /// List of [Scope] OAuth-2.0 grants given to the [OAuthGrant].
        scope, scopes: { "scopes" Scope },
    }
}

impl_sql_find_or_create!(OAuthGrant: client_id);

impl TryFrom<Grant> for OAuthGrant {
    type Error = Error;

    fn try_from(val: Grant) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Grant> for OAuthGrant {
    type Error = Error;

    fn try_from(val: &Grant) -> Result<Self> {
        let uuid = util::rand_uuid();

        let owner_id = Iri::try_from(val.owner_id.as_str())?;

        log::debug!("oauth: grant: owner ID: {owner_id}");

        let client_id = val
            .client_id
            .as_str()
            .parse::<Uuid>()
            .map_err(|err| Error::http(format!("oauth: grant: invalid client ID: {err}")))?;

        let scopes = ScopeList::try_from(&val.scope)?;
        let redirect_uri = Iri::try_from(val.redirect_uri.as_str())?;

        let pkce = val
            .extensions
            .private()
            .into_iter()
            .find(|(k, _)| *k == "pkce")
            .and_then(|(_, v)| v)
            .map(|v| v.to_string())
            .unwrap_or_default();

        Ok(Self {
            uuid,
            owner_id,
            client_id,
            scopes,
            redirect_uri,
            until: val.until.into(),
            pkce,
            tag: String::new(),
        })
    }
}

impl TryFrom<OAuthGrant> for Grant {
    type Error = Error;

    fn try_from(val: OAuthGrant) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&OAuthGrant> for Grant {
    type Error = Error;

    fn try_from(val: &OAuthGrant) -> Result<Self> {
        let owner_id = val.owner_id().as_str().to_string();
        let client_id = val.client_id().to_string();

        let scope: OAuthScope = (&val.scopes)
            .try_into()
            .map_err(|err| Error::http(format!("oauth: grant: {err}")))?;

        let redirect_uri = val
            .redirect_uri()
            .as_str()
            .parse()
            .map_err(|err| Error::http(format!("oauth: grant: {err}")))?;

        let mut extensions = oauth::primitives::grant::Extensions::default();

        let pkce = val.pkce();
        if !pkce.is_empty() {
            extensions.set_raw(
                "pkce".to_string(),
                oauth::primitives::grant::Value::private(Some(pkce.into())),
            );
        }

        Ok(Self {
            owner_id,
            client_id,
            scope,
            redirect_uri,
            until: val.until.into(),
            extensions,
        })
    }
}
