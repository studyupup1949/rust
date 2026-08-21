use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use chrono::Utc;
use oauth::primitives::scope::Scope as OAuthScope;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::app::oauth::{OAuthGrantType, Scope, ScopeList};
use crate::crypto::{Password, SymmetricKey};
use crate::db::{DateTime, Db, Iri, IriList, Key, OAuthGrantTypeList, Transaction, Uuid, UuidList};
use crate::{Error, Result, impl_sql_list_field, impl_sql_record, util};

mod auth_method;
mod response;
mod secret;

pub use auth_method::*;
pub use response::*;
pub use secret::*;

/// Represents whether an OAuth-2.0 client is private or public.
///
/// Public clients are generally federated clients with accounts on other instances.
/// They will have reduced privileges related to repositories, issues, PRs, etc.
///
/// Private clients are used to authenticate local accounts.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "oauth_client_type", rename_all = "lowercase")]
pub enum OAuthClientType {
    Private,
    Public,
}

impl OAuthClientType {
    /// String representation of the [Private](Self::Private) variant.
    pub const PRIVATE: &str = "private";
    /// String representation of the [Public](Self::Public) variant.
    pub const PUBLIC: &str = "public";

    /// Creates a new [OAuthClientType].
    #[inline]
    pub const fn new() -> Self {
        Self::Private
    }

    /// Gets the [OAuthClientType] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Private => Self::PRIVATE,
            Self::Public => Self::PUBLIC,
        }
    }
}

impl_default!(OAuthClientType);
impl_display!(OAuthClientType, str);

/// Represents an OAuth-2.0 dynamic client registration request.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct OAuthClientRegister {
    #[serde(skip_serializing_if = "Option::is_none")]
    redirect_uris: Option<Vec<Iri>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope: Option<OAuthScope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    jwks: Option<jwt::jwk::JwkSet>,
}

impl OAuthClientRegister {
    /// Creates a new [OAuthClientRegister].
    #[inline]
    pub const fn new() -> Self {
        Self {
            redirect_uris: None,
            scope: None,
            jwks: None,
        }
    }
}

field_access! {
    OAuthClientRegister {
        /// Represents the OAuth-2.0 client redirect URIs.
        redirect_uris: option_deref { &[Iri], Vec<Iri> },
    }
}

field_access! {
    OAuthClientRegister {
        /// Represents the OAuth-2.0 client scope.
        scope: option_ref { OAuthScope },
        /// Represents the OAuth-2.0 client JWT key set.
        jwks: option_ref { jwt::jwk::JwkSet },
    }
}

impl_default!(OAuthClientRegister);
impl_display!(OAuthClientRegister, json);

impl TryFrom<OAuthClientRegister> for OAuthClient {
    type Error = Error;

    fn try_from(val: OAuthClientRegister) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&OAuthClientRegister> for OAuthClient {
    type Error = Error;

    fn try_from(val: &OAuthClientRegister) -> Result<Self> {
        let mut client = Self::new().with_uuid(util::rand_uuid());

        if let Some(uris) = val.redirect_uris() {
            client.set_redirect_uris(uris)?;
        }

        if let Some(scope) = val.scope() {
            ScopeList::try_from(scope).and_then(|scopes| client.with_scopes(scopes))
        } else {
            Ok(client)
        }
    }
}

/// Represents an OAuth-2.0 client database record.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, FromRow)]
#[sqlx(type_name = "oauth_client")]
pub struct OAuthClient {
    #[serde(
        rename = "client_id",
        serialize_with = "util::ser_uuid",
        deserialize_with = "util::de_uuid"
    )]
    uuid: Uuid,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    owner_id: Uuid,
    client_type: OAuthClientType,
    scopes: ScopeList,
    #[serde(skip)]
    password: Password,
    issued_at: DateTime,
    redirect_uris: Vec<Iri>,
    grant_types: Vec<OAuthGrantType>,
    #[serde(
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    key_ids: Vec<Uuid>,
}

impl OAuthClient {
    /// Creates a new [OAuthClient].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            owner_id: Uuid::nil(),
            client_type: OAuthClientType::new(),
            scopes: ScopeList::new(),
            password: Password::new(),
            issued_at: Utc::now().into(),
            redirect_uris: Vec::new(),
            grant_types: [
                OAuthGrantType::AuthorizationCode,
                OAuthGrantType::RefreshToken,
            ]
            .into(),
            key_ids: Vec::new(),
        }
    }

    /// Gets the [OAuthClient] `client_id`.
    #[inline]
    pub const fn client_id(&self) -> Uuid {
        self.uuid
    }

    /// Gets the OAuth-2.0 serialized scope paramter.
    pub fn oauth_scope(&self) -> Result<OAuthScope> {
        (&self.scopes)
            .try_into()
            .map_err(|err| Error::http(format!("oauth: client: {err}")))
    }

    /// Performs checks on record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.password.is_empty() {
            Err(Error::db("oauth: client: empty password"))
        } else if self.scopes.is_empty() {
            Err(Error::db("oauth: client: empty scopes"))
        } else {
            Ok(())
        }
    }

    /// Attempts to convert an [OAuthClientRegister] request into an [OAuthClient].
    pub async fn try_from_register(
        db: &Db,
        uri: &Iri,
        owner_id: Uuid,
        client_secret: &ClientSecret,
        val: &OAuthClientRegister,
    ) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let client =
            Self::try_from_register_tx(&mut dbtx, &db_key, uri, owner_id, client_secret, val)
                .await?;

        dbtx.commit()
            .await
            .map(|_| client)
            .map_err(|err| Error::db(format!("oauth: client: {err}")))
    }

    /// Attempts to convert an [OAuthClientRegister] request into an [OAuthClient] using a transaction.
    pub async fn try_from_register_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        uri: &Iri,
        owner_id: Uuid,
        client_secret: &ClientSecret,
        val: &OAuthClientRegister,
    ) -> Result<Self> {
        let mut client = Self::try_from(val).map(|c| c.with_owner_id(owner_id))?;

        let password = Password::derive(
            &client.uuid.to_string(),
            client_secret.to_string().as_bytes(),
        )?;
        client.set_password(password.clone());

        let client_id = Self::TABLE.id_from_uuid(uri, client.uuid)?;
        let client_entry = client.table_entry();

        let mut key_ids = Vec::new();

        if let Some(jwks) = val.jwks() {
            for mut key in jwks.keys.iter().filter_map(|jwk| {
                Key::try_from(jwk)
                    .map(|k| k.with_actor_id(client_id.clone()).with_actor(client_entry))
                    .map_err(|err| log::warn!("oauth: client: error parsing JWK: {err}"))
                    .ok()
            }) {
                let key_uuid = util::rand_uuid();
                let key_id = Key::TABLE.id_from_uuid(uri, key_uuid)?;

                key.set_id(key_id);

                let key_uuid = key
                    .insert_tx(dbtx, db_key)
                    .await
                    .map_err(|err| Error::db(format!("oauth: client: {err}")))?;

                key_ids.push(key_uuid);
            }
        }

        client.set_key_ids(key_ids)?;

        client.insert_tx(dbtx).await.map(|_| client)
    }

    /// Attempts to find all [OAuthClient] key entries.
    pub async fn keys(&self, db: &Db) -> Result<Vec<Key>> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let client = self.keys_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| client)
            .map_err(|err| Error::db(format!("oauth: client: {err}")))
    }

    /// Attempts to find all [OAuthClient] key entries using a transaction.
    pub async fn keys_tx(
        &self,
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
    ) -> Result<Vec<Key>> {
        Key::find_by_actor_tx(dbtx, db_key, self.table_entry()).await
    }
}

field_access! {
    OAuthClient {
        /// Represents the [Uuid] primary key of the table entry.
        uuid: Uuid,
        /// Represents the [Uuid] of the [owner](crate::db::Person) of the client.
        owner_id: Uuid,
        /// Represents the OAuth-2.0 client type.
        client_type: OAuthClientType,
        /// Represents when the OAuth-2.0 client record was issued.
        issued_at: DateTime,
    }
}

field_access! {
    OAuthClient {
        /// Represents the password used for [Private](OAuthClientType::Private) clients.
        password: as_ref { Password },
    }
}

impl_sql_record! {
    OAuthClient {
        owner_id: { "owner_id" Uuid },
        client_type: { "client_type" OAuthClientType },
        scopes: { "scopes" ScopeList },
        password: { "password" Password },
        issued_at: { "issued_at" DateTime },
        redirect_uris: { "redirect_uris" IriList },
        grant_types: { "grant_types" OAuthGrantTypeList },
        key_ids: { "key_ids" UuidList },
    }
}

impl_sql_list_field! {
    OAuthClient {
        /// List of [Scope] OAuth-2.0 grants given to the [OAuthClient].
        scope, scopes: { "scopes" Scope },
        /// List of OAuth-2.0 redirect URIs used by the [OAuthClient].
        redirect_uri, redirect_uris: { "redirect_uris" Iri },
        /// List of OAuth-2.0 grant types permitted for the [OAuthClient].
        grant_type, grant_types: { "grant_types" OAuthGrantType },
        /// List of UUIDs referencing OAuth-2.0 JWK public keys for the [OAuthClient].
        key_id, key_ids: { "key_ids" Uuid },
    }
}

impl_default!(OAuthClient);
impl_display!(OAuthClient, json);
