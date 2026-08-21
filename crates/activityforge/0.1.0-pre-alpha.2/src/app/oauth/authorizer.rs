use std::sync::Arc;

use async_trait::async_trait;

use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use oauth::primitives::grant::Grant;
use oauth_async::primitives::Authorizer;
use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, MutexGuard};

use crate::Error;
use crate::app::oauth::{OAuthGrant, OAuthOpaqueError, opaque_error};
use crate::db::Db;

mod request;

pub use request::*;

/// Represents an OAuth-2.0 authorization code response.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct AuthorizationCode {
    code: String,
}

impl AuthorizationCode {
    /// Creates a new [AuthorizationCode].
    pub const fn new() -> Self {
        Self {
            code: String::new(),
        }
    }
}

field_access! {
    AuthorizationCode {
        code: as_ref { &str, String },
    }
}

impl_default!(AuthorizationCode);
impl_display!(AuthorizationCode, json);

/// Represents the OAuth-2.0 authorizer used to verify a resource grant request.
///
/// Issues an authentication code that can be used to redeem an access bearer token.
#[derive(Clone)]
pub struct OAuthAuthorizer {
    db: Arc<Mutex<Db>>,
}

impl OAuthAuthorizer {
    /// Creates a new [OAuthAuthorizer].
    pub const fn new(db: Arc<Mutex<Db>>) -> Self {
        Self { db }
    }

    /// Gets a guarded reference to the database.
    pub async fn db(&self) -> MutexGuard<'_, Db> {
        self.db.lock().await
    }

    /// Creates a signature tag for the provided `grant`.
    pub async fn authorize_grant(&mut self, grant: Grant) -> Result<String, Error> {
        let db = self.db().await;
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let key = db.key()?;

        let mut oauth_grant = OAuthGrant::try_from(grant)?;
        let tag = OAuthGrant::create_tag(key.as_ref())?;
        oauth_grant.set_tag(tag);
        oauth_grant.insert_tx(&mut dbtx).await?;

        dbtx.commit()
            .await
            .map(|_| oauth_grant.tag().to_string())
            .map_err(|err| Error::http(format!("oauth: authz: error storing grant {err}")))
    }

    /// Attempts to find a [OAuthGrant] associated with the `token`.
    pub async fn extract_grant(&mut self, token: &str) -> Result<Option<Grant>, Error> {
        let db = self.db().await;
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let db_grant = OAuthGrant::find_by_tag_tx(&mut dbtx, token).await?;

        let grant = if let Some(db_grant) = db_grant {
            let grant = Grant::try_from(&db_grant)?;
            db_grant.delete_tx(&mut dbtx).await?;
            Some(grant)
        } else {
            None
        };

        dbtx.commit()
            .await
            .map(|_| grant)
            .map_err(|err| Error::db(format!("oauth: authz: {err}")))
    }
}

#[async_trait]
impl Authorizer for OAuthAuthorizer {
    async fn authorize(&mut self, grant: Grant) -> Result<String, OAuthOpaqueError> {
        let res = self.authorize_grant(grant).await;

        res.map_err(|err| {
            log::error!("{err}");
            opaque_error()
        })
    }

    async fn extract(&mut self, token: &str) -> Result<Option<Grant>, OAuthOpaqueError> {
        let res = self.extract_grant(token).await;

        res.map_err(|err| {
            log::error!("{err}");
            opaque_error()
        })
    }
}
