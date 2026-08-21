use std::sync::Arc;

use axum::Router;
use axum::http::Method;
use axum::routing::{get, post};

use activitystreams_vocabulary::{Item, Items};

use crate::crypto::HttpPrivateKey;
use crate::db::{DbConfig, Iri, Name};
use crate::{Error, Result};

mod credentials;
mod factory;
mod middleware;
mod person;
mod repository;
mod state;

pub mod oauth;

pub use credentials::*;
pub use state::*;

use crate::middleware;

/// Represents an application instance.
pub struct App {
    state: Arc<AppState>,
}

impl App {
    /// Creates a new [App] instance from the provided database configuration.
    pub async fn create(config: DbConfig, uri: Iri, name: Name) -> Result<Self> {
        AppState::create(config, uri, name)
            .await
            .map(|s| Self { state: Arc::new(s) })
    }

    /// Gets a reference to the application shared state.
    pub fn state(&self) -> &AppState {
        self.state.as_ref()
    }

    /// Clones the application shared state.
    pub fn clone_state(&self) -> Arc<AppState> {
        Arc::clone(&self.state)
    }

    /// Gets a reference to the application URI.
    #[inline]
    pub fn uri(&self) -> &Iri {
        self.state.uri()
    }

    /// Gets the default OAuth-2.0 callback endpoint for the provided URI.
    #[inline]
    pub fn oauth_callback_uri(uri: &Iri) -> Result<Iri> {
        uri.base_iri()
            .and_then(|base| Iri::try_from(format!("{base}/oauth/callback")))
    }

    /// Creates a signed request to a remote host using the provided keys.
    ///
    /// Uses HTTP Message Signatures (RFC 9421) to create a signed request.
    pub async fn signed_request_with_keys<S: serde::ser::Serialize>(
        keys: &[HttpPrivateKey],
        method: Method,
        uri: &Iri,
        body: Option<&S>,
    ) -> Result<reqwest::Response> {
        AppState::signed_request_with_keys(keys, method, uri, body).await
    }

    /// Creates the main application router handler.
    pub async fn router(&self) -> Result<Router> {
        Ok(Router::new()
            .route("/api/v1/factories/{uuid}/outbox", get(Self::factory_outbox_read))
            .route("/api/v1/factories/{uuid}/outbox", post(Self::factory_outbox_write))
            .route("/api/v1/factories/{uuid}", get(Self::get_factory))
            .route("/api/v1/persons/{uuid}/inbox", get(Self::person_inbox_read))
            .route("/api/v1/persons/{uuid}/inbox", post(Self::person_inbox_write))
            .route("/api/v1/persons/{uuid}/outbox", get(Self::person_outbox_read))
            .route("/api/v1/persons/{uuid}/outbox", post(Self::person_outbox_write))
            .route("/api/v1/persons/{uuid}", get(Self::get_person))
            .route("/api/v1/repositories/{uuid}/inbox", get(Self::repository_inbox_read))
            .route("/api/v1/repositories/{uuid}/inbox", post(Self::repository_inbox_write))
            .route("/api/v1/repositories/{uuid}/outbox", get(Self::repository_outbox_read))
            .route("/api/v1/repositories/{uuid}/outbox", post(Self::repository_outbox_write))
            .route("/api/v1/repositories/{uuid}", get(Self::get_repository))
            .route_layer(
                middleware!(Self: self.clone_state() => { oauth_or_httpsig_handler, fetch_actor_handler }),
            )
            .route("/oauth/authenticate", post(Self::oauth_authenticate))
            .route("/oauth/register", post(Self::oauth_register))
            .route("/oauth/authorize", get(Self::oauth_authorize))
            .route("/oauth/callback", get(Self::oauth_callback))
            .route("/oauth/token", post(Self::oauth_token))
            .route("/oauth/refresh", post(Self::oauth_refresh))
            .with_state(self.clone_state()))
    }

    /// Creates a test application router handler.
    #[cfg(feature = "e2e-tests")]
    pub async fn test_router(&self) -> Result<Router> {
        use axum::extract::State;

        Ok(Router::new()
            .route("/fetch-actor", get(|_: State<Arc<AppState>>| async { "Test HTTP Message Signature + fetch actor handler" }))
            .route_layer(
                middleware!(Self: self.clone_state() => { http_signature_handler, fetch_actor_handler }),
            )
            .route("/signature", get(|_: State<Arc<AppState>>| async { "Test HTTP Message Signature handler" }))
            .route_layer(
                middleware!(Self: self.clone_state() => { http_signature_handler }),
            )
            .route("/", get(|_: State<Arc<AppState>>| async { "Test ROOT handler" }))
            .with_state(self.clone_state()))
    }

    /// Checks that an `Activity` `actor` has an ID matching the `actor_id` derived from the RFC 9421 signature.
    fn check_activity_actor_id(context: &str, actor_id: &Iri, vocab_actor: &Items) -> Result<()> {
        match vocab_actor {
            Items::Single(Item::Iri(id)) => {
                let activity_actor_id = id.as_ref().into();
                if actor_id == &activity_actor_id {
                    Ok(())
                } else {
                    Err(Error::http(format!(
                        "{context}: mismatch of Activity actor ID: {activity_actor_id} and signer ID: {actor_id}"
                    )))
                }
            }
            Items::Single(Item::Object(obj)) => {
                let activity_actor_id = obj.id().map(|i| i.into()).unwrap_or_default();
                if actor_id == &activity_actor_id {
                    Ok(())
                } else {
                    Err(Error::http(format!(
                        "{context}: mismatch of Activity actor ID: {activity_actor_id} and signer ID: {actor_id}"
                    )))
                }
            }
            Items::Single(Item::Link(link)) => {
                let activity_actor_id = link.href().into();
                if actor_id == &activity_actor_id {
                    Ok(())
                } else {
                    Err(Error::http(format!(
                        "{context}: mismatch of Activity actor ID: {activity_actor_id} and signer ID: {actor_id}"
                    )))
                }
            }
            Items::List(list) => {
                if list.iter().any(|i| match i {
                    Item::Iri(id) => actor_id.as_str() == id.as_str(),
                    Item::Object(obj) => obj
                        .id()
                        .map(|i| actor_id.as_str() == i.as_str())
                        .unwrap_or_default(),
                    Item::Link(link) => link.href().as_str() == actor_id.as_str(),
                }) {
                    Ok(())
                } else {
                    Err(Error::http(format!(
                        "{context}: no matching Activity actor ID for ID: {actor_id}",
                    )))
                }
            }
        }
    }
}
