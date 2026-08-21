use std::sync::Arc;

use tokio::sync::Mutex;

use oauth::endpoint::{OAuthError, Scopes, Template, WebRequest};
use oauth::frontends::simple::endpoint::{ResponseCreator, Vacant};
use oauth_async::endpoint::{Endpoint, Extension, OwnerSolicitor};
use oauth_async::primitives::{Authorizer, Issuer, Registrar};

use crate::app::oauth::{
    EndpointScopes, OAuthAuthorizer, OAuthIssuer, OAuthOwnerSolicitor, OAuthRegistrar, Pkce,
};
use crate::db::{Db, Iri};
use crate::{Error, Result};

/// Represents an [OAuth-2.0 Endpoint](Endpoint) implementation.
pub struct OAuthEndpoint {
    registrar: OAuthRegistrar,
    authorizer: OAuthAuthorizer,
    issuer: OAuthIssuer,
    owner_solicitor: OAuthOwnerSolicitor,
    pkce: Pkce,
    scopes: EndpointScopes,
    response: Vacant,
}

impl OAuthEndpoint {
    /// Creates a new [OAuthEndpoint].
    ///
    /// # Parameters
    ///
    /// - `db`: wrapped reference to the database connection.
    /// - `uri`: the application URI
    /// - `key_id`: the key ID used to fetch the application's signing key.
    pub fn new(db: &Arc<Mutex<Db>>, uri: Iri, key_id: Iri) -> Self {
        let registrar = OAuthRegistrar::new(Arc::clone(db), uri.clone());
        let authorizer = OAuthAuthorizer::new(Arc::clone(db));
        let issuer = OAuthIssuer::new(Arc::clone(db), key_id);
        let owner_solicitor = OAuthOwnerSolicitor::new(Arc::clone(db), uri);
        let pkce = Pkce::new();
        let scopes = EndpointScopes::new();

        Self {
            registrar,
            authorizer,
            issuer,
            owner_solicitor,
            pkce,
            scopes,
            response: Vacant,
        }
    }
}

impl<Request> Endpoint<Request> for OAuthEndpoint
where
    Request: WebRequest + Send,
    Request::Error: std::fmt::Display,
    Request::Response: Default,
{
    type Error = Error;

    fn registrar(&self) -> Option<&(dyn Registrar + Sync)> {
        Some(&self.registrar)
    }

    fn authorizer_mut(&mut self) -> Option<&mut (dyn Authorizer + Send)> {
        Some(&mut self.authorizer)
    }

    fn issuer_mut(&mut self) -> Option<&mut (dyn Issuer + Send)> {
        Some(&mut self.issuer)
    }

    fn owner_solicitor(&mut self) -> Option<&mut (dyn OwnerSolicitor<Request> + Send)> {
        Some(&mut self.owner_solicitor)
    }

    fn scopes(&mut self) -> Option<&mut dyn Scopes<Request>> {
        Some(&mut self.scopes)
    }

    fn response(&mut self, request: &mut Request, kind: Template) -> Result<Request::Response> {
        Ok(self.response.create(request, kind))
    }

    fn error(&mut self, err: OAuthError) -> Self::Error {
        Error::http(format!("oauth: endpoint: error: {err}"))
    }

    fn web_error(&mut self, err: Request::Error) -> Self::Error {
        Error::http(format!("oauth: endpoint: web error: {err}"))
    }

    fn extension(&mut self) -> Option<&mut (dyn Extension + Send)> {
        Some(&mut self.pkce)
    }
}
