use std::sync::Arc;

use oauth::endpoint::{OwnerConsent, OwnerSolicitor, Solicitation, WebRequest};

use tokio::sync::{Mutex, MutexGuard};

use crate::app::App;
use crate::db::{Db, Iri};

/// Represents the OAuth-2.0 owner-solicitor used to verify a user is authenticated, and accepts the
/// presents grant(s).
///
/// Issues an authentication code that can be used to redeem an access bearer token.
#[derive(Clone)]
pub struct OAuthOwnerSolicitor {
    db: Arc<Mutex<Db>>,
    uri: Iri,
}

impl OAuthOwnerSolicitor {
    /// Creates a new [OAuthOwnerSolicitor].
    pub const fn new(db: Arc<Mutex<Db>>, uri: Iri) -> Self {
        Self { db, uri }
    }

    /// Gets a guarded reference to the database connection.
    pub async fn db(&self) -> MutexGuard<'_, Db> {
        self.db.lock().await
    }
}

impl<Request: WebRequest + Send> OwnerSolicitor<Request> for OAuthOwnerSolicitor {
    fn check_consent(
        &mut self,
        _req: &mut Request,
        solicitation: Solicitation<'_>,
    ) -> OwnerConsent<Request::Response> {
        let Ok(redirect_uri) = App::oauth_callback_uri(&self.uri)
            .map(|u| u.to_string())
            .map_err(|err| log::error!("oauth: solicitor: {err}"))
        else {
            return OwnerConsent::Denied;
        };

        let req_redirect_uri = solicitation.pre_grant().redirect_uri.to_string();

        if req_redirect_uri == redirect_uri {
            OwnerConsent::Authorized(solicitation.pre_grant().client_id.clone())
        } else {
            log::error!(
                "oauth: solicitor: invalid redirect URL, have: {req_redirect_uri}, expected: {redirect_uri}"
            );
            log::warn!(
                "oauth: solicitor: interactive consent with resource owners not yet supported"
            );

            OwnerConsent::Denied
        }
    }
}
