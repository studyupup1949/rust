use std::borrow::Cow;
use std::sync::Arc;

use async_trait::async_trait;
use oauth::primitives::registrar::{
    BoundClient, ClientUrl, ExactUrl, PasswordPolicy, PreGrant, RegisteredUrl, RegistrarError,
};
use oauth::primitives::scope::Scope as OAuthScope;
use oauth_async::primitives::Registrar;
use tokio::sync::{Mutex, MutexGuard};

use crate::app::oauth::{OAuthClient, ScopeList};
use crate::crypto::Password;
use crate::db::{Db, Iri, Uuid};

/// Represents an [OAuth-2.0 Registrar](Registrar) implementation.
#[derive(Clone)]
pub struct OAuthRegistrar {
    db: Arc<Mutex<Db>>,
    uri: Iri,
}

impl OAuthRegistrar {
    /// Creates a new [OAuthRegistrar].
    pub const fn new(db: Arc<Mutex<Db>>, uri: Iri) -> Self {
        Self { db, uri }
    }

    /// Gets a guarded reference to the database.
    pub async fn db(&self) -> MutexGuard<'_, Db> {
        self.db.lock().await
    }

    /// Gets a reference to the [OAuthRegistrar] URI.
    pub const fn uri(&self) -> &Iri {
        &self.uri
    }
}

impl PasswordPolicy for OAuthRegistrar {
    fn store(&self, client_id: &str, passphrase: &[u8]) -> Vec<u8> {
        match Password::derive(client_id, passphrase) {
            Ok(password) => password.into_bytes(),
            Err(err) => {
                log::error!("oauth: error deriving password hash: {err}");
                Vec::new()
            }
        }
    }

    fn check(
        &self,
        client_id: &str,
        passphrase: &[u8],
        stored: &[u8],
    ) -> Result<(), RegistrarError> {
        Password::from_slice(stored)
            .map_err(|_| RegistrarError::PrimitiveError)
            .and_then(|p| {
                p.verify(client_id, passphrase)
                    .map_err(|_| RegistrarError::Unspecified)
            })
    }
}

#[async_trait]
impl Registrar for OAuthRegistrar {
    async fn bound_redirect<'a>(
        &self,
        bound: ClientUrl<'a>,
    ) -> Result<BoundClient<'a>, RegistrarError> {
        let client_id = bound.client_id.parse::<Uuid>().map_err(|err| {
            log::error!("oauth: registrar: invalid client ID: {err}");
            RegistrarError::Unspecified
        })?;

        let db_client = OAuthClient::get(&*self.db().await, &client_id)
            .await
            .map_err(|err| {
                log::error!(
                    "oauth: registrar: error finding client, ID: {client_id}, error: {err}"
                );
                RegistrarError::Unspecified
            })?;

        let redirect_uri = db_client
            .redirect_uris()
            .first()
            .map(|u| u.to_string())
            .unwrap_or(format!("{}/oauth/callback", self.uri()));

        ExactUrl::new(redirect_uri)
            .map(|uri| BoundClient {
                client_id: bound.client_id,
                redirect_uri: Cow::Owned(RegisteredUrl::Exact(uri)),
            })
            .map_err(|err| {
                log::error!("oauth: registrar: {err}");
                RegistrarError::Unspecified
            })
    }

    async fn negotiate<'a>(
        &self,
        client: BoundClient<'a>,
        scope: Option<OAuthScope>,
    ) -> Result<PreGrant, RegistrarError> {
        let client_id = client.client_id.as_ref().parse::<Uuid>().map_err(|err| {
            log::error!("oauth: registrar: invalid client ID: {err}");
            RegistrarError::Unspecified
        })?;

        let db_client = OAuthClient::get(&*self.db().await, &client_id)
            .await
            .map_err(|err| {
                log::error!("oauth: registrar: error fetching client: {err}");
                RegistrarError::Unspecified
            })?;

        let scope_list = if let Some(req_scopes) = scope {
            let req_list = ScopeList::try_from(req_scopes).map_err(|err| {
                log::error!("oauth: registrar: error parsing requested scopes: {err}");
                RegistrarError::Unspecified
            })?;
            let list = req_list
                .into_iter()
                .filter(|s| db_client.scopes().contains(s))
                .collect::<Vec<_>>();
            if list.is_empty() {
                ScopeList::from(db_client.scopes())
            } else {
                ScopeList::from(list)
            }
        } else {
            ScopeList::from(db_client.scopes())
        };

        OAuthScope::try_from(scope_list)
            .map_err(|err| {
                log::error!("oauth: registrar: error converting negotiated scopes: {err}");
                RegistrarError::Unspecified
            })
            .map(|scope| PreGrant {
                client_id: client.client_id.into(),
                redirect_uri: client.redirect_uri.into_owned(),
                scope,
            })
    }

    async fn check(
        &self,
        client_id: &str,
        passphrase: Option<&[u8]>,
    ) -> Result<(), RegistrarError> {
        let client_uuid = client_id.parse::<Uuid>().map_err(|err| {
            log::error!("oauth: registrar: invalid client ID: {err}");
            RegistrarError::Unspecified
        })?;

        let client = OAuthClient::get(&*self.db().await, &client_uuid)
            .await
            .map_err(|err| {
                log::error!("oauth: registrar: error fetching client: {err}");
                RegistrarError::Unspecified
            })?;

        match passphrase {
            Some(passphrase) => client
                .password()
                .verify(client_id, passphrase)
                .map_err(|err| {
                    log::error!("oauth: registrar: invalid password: {err}");
                    RegistrarError::Unspecified
                }),
            None => {
                log::error!("oauth: registrar: missing expected passphrase");
                Err(RegistrarError::Unspecified)
            }
        }
    }
}
