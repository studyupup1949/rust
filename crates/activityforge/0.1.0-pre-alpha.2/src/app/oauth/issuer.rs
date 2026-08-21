use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use chrono::Utc;
use oauth::primitives::grant::Grant;
use oauth::primitives::issuer::{IssuedToken, RefreshedToken};
use oauth_async::primitives::Issuer;
use tokio::sync::{Mutex, MutexGuard};

use crate::app::oauth::{
    OAuthClaims, OAuthGrant, OAuthOpaqueError, OAuthToken, OAuthTokenType, opaque_error,
};
use crate::crypto::{Nonce, PrivateKey, SymmetricKey};
use crate::db::{Db, Iri, Key, Transaction};
use crate::{Error, Result, util};

/// Represents the OAuth-2.0 issuer used to verify a resource grant request.
#[derive(Clone)]
pub struct OAuthIssuer {
    db: Arc<Mutex<Db>>,
    key_id: Iri,
}

impl OAuthIssuer {
    /// Creates a new [OAuthIssuer].
    pub const fn new(db: Arc<Mutex<Db>>, key_id: Iri) -> Self {
        Self { db, key_id }
    }

    /// Gets a guarded reference to the database.
    pub async fn db(&self) -> MutexGuard<'_, Db> {
        self.db.lock().await
    }

    /// Gets a reference to the private signing key ID.
    pub const fn key_id(&self) -> &Iri {
        &self.key_id
    }

    /// Attempts to retrieve the signing key for the [OAuthIssuer].
    pub async fn key(&self) -> Result<PrivateKey> {
        let db = self.db().await;
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let key = self.key_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| key)
            .map_err(|err| Error::db(format!("oauth: issuer: {err}")))
    }

    /// Attempts to retrieve the signing key for the [OAuthIssuer] using a transaction.
    pub async fn key_tx(
        &self,
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
    ) -> Result<PrivateKey> {
        let key_id = &self.key_id;

        Key::find_by_key_id_tx(dbtx, key_id, db_key)
            .await
            .and_then(|k| k.ok_or(Error::db("oauth: issuer: missing key for: {key_id}")))
            .and_then(PrivateKey::try_from)
    }

    /// Issues a JWT access bearer token.
    pub async fn create_token(
        &self,
        grant: &Grant,
        refresh: Option<&str>,
        token_type: OAuthTokenType,
    ) -> Result<OAuthToken> {
        let claims: OAuthClaims = grant.clone().into();

        if claims.is_expired() {
            return Err(Error::http(format!(
                "oauth: issuer: expired grant for: {}",
                claims.subject()
            )));
        }

        let db = self.db().await;

        let pool = db
            .pool()
            .map_err(|err| Error::db(format!("oauth: issuer: {err}")))?;

        let db_key = db
            .key()
            .map_err(|err| Error::db(format!("oauth: issuer: {err}")))?;

        let mut dbtx = pool
            .begin()
            .await
            .map_err(|err| Error::db(format!("oauth: issuer: {err}")))?;

        let mut db_grant = OAuthGrant::try_from(grant)?;

        let tag = OAuthGrant::create_tag(db_key.as_ref())
            .map_err(|err| Error::http(format!("oauth: issuer: {err}")))?;

        db_grant.set_tag(tag);

        let db_key = self.key_tx(&mut dbtx, &db_key).await?;

        db_grant.find_or_create_tx(&mut dbtx).await?;

        if let Some(refresh) = refresh {
            OAuthToken::verify_token(&db_key.public_key(), refresh)?;

            OAuthToken::find_by_token_tx(&mut dbtx, refresh, OAuthTokenType::Refresh)
                .await
                .and_then(|k| {
                    k.ok_or(Error::db(format!(
                        "oauth: issuer: no token found for refresh: {refresh}"
                    )))
                })?;
        }

        let expires = Utc::now() + Duration::from_hours(4);

        let refresh_claims = claims
            .clone()
            .with_expires(expires.timestamp() as u64)
            .with_nonce(Nonce::random());

        let token = OAuthToken::sign_token(&db_key, &claims)?;
        let refresh_token = OAuthToken::sign_token(&db_key, &refresh_claims)?;

        let uuid = util::rand_uuid();

        let mut oauth_token = OAuthToken::new()
            .with_uuid(uuid)
            .with_token(token)
            .with_refresh_token(refresh_token)
            .with_until(expires)
            .with_token_type(token_type)
            .with_scope(db_grant.scopes())
            .with_grant_id(db_grant.uuid());

        oauth_token
            .insert_or_update_tx(&mut dbtx)
            .await
            .map_err(|err| Error::http(format!("oauth: issuer: {err}")))?;

        dbtx.commit()
            .await
            .map(|_| oauth_token)
            .map_err(|err| Error::db(format!("oauth: issuer: {err}")))
    }

    /// Verifies and extracts a JWT access bearer token.
    pub async fn extract_token(&self, token: &str, token_type: OAuthTokenType) -> Result<Grant> {
        let db = self.db().await;
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let db_key = self.key_tx(&mut dbtx, &db_key).await?;

        OAuthToken::verify_token(&db_key.public_key(), token)?;

        let db_grant = OAuthGrant::find_by_token_tx(&mut dbtx, token, token_type)
            .await
            .and_then(|g| {
                g.ok_or(Error::db(format!(
                    "oauth: issuer: missing grant for: {token}"
                )))
            })?;

        dbtx.commit()
            .await
            .map_err(|err| Error::db(format!("oauth: issuer: {err}")))
            .and_then(|_| db_grant.try_into())
    }
}

#[async_trait]
impl Issuer for OAuthIssuer {
    async fn issue(&mut self, grant: Grant) -> core::result::Result<IssuedToken, OAuthOpaqueError> {
        let oauth_token = self
            .create_token(&grant, None, OAuthTokenType::Access)
            .await
            .map_err(|err| {
                log::error!("{err}");
                opaque_error()
            })?;

        oauth_token.try_into().map_err(|err| {
            log::error!("oauth: issuer: {err}");
            opaque_error()
        })
    }

    async fn refresh(
        &mut self,
        refresh: &str,
        grant: Grant,
    ) -> core::result::Result<RefreshedToken, OAuthOpaqueError> {
        let oauth_token = self
            .create_token(&grant, Some(refresh), OAuthTokenType::Refresh)
            .await
            .map_err(|err| {
                log::error!("{err}");
                opaque_error()
            })?;

        oauth_token.try_into().map_err(|err| {
            log::error!("oauth: issuer: {err}");
            opaque_error()
        })
    }

    async fn recover_token(
        &mut self,
        token: &str,
    ) -> core::result::Result<Option<Grant>, OAuthOpaqueError> {
        self.extract_token(token, OAuthTokenType::Access)
            .await
            .map(Some)
            .map_err(|err| {
                log::error!("{err}");
                opaque_error()
            })
    }

    async fn recover_refresh(
        &mut self,
        token: &str,
    ) -> core::result::Result<Option<Grant>, OAuthOpaqueError> {
        self.extract_token(token, OAuthTokenType::Refresh)
            .await
            .map(Some)
            .map_err(|err| {
                log::error!("{err}");
                opaque_error()
            })
    }
}
