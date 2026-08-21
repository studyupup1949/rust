use std::sync::Arc;

use axum::extract::{Request, State};
use axum::middleware::Next;
use axum::response::{IntoResponse, Response};

use http::{Method, StatusCode, header};

use crate::app::oauth::OAuthToken;
use crate::app::{App, AppState};
use crate::crypto::{AlgorithmName, HttpContentDigest, HttpMessageSignature, HttpPublicKey};
use crate::db::{Actor as DbActor, Iri, Key, TableEntry, TableType};
use crate::{Actor, Error, Role};

/// Helper macro to define HTTP middleware layers for `axum` routers.
#[macro_export]
macro_rules! middleware {
    ($ty:ident: $state:expr => { $($middleware_fn:ident$(,)?)+ }) => {
        ::tower::ServiceBuilder::new()
            $(
            .layer(::axum::middleware::from_fn_with_state($state.clone(), $ty::$middleware_fn))
            )+
    };
}

impl App {
    /// Handles HTTP Message Signature (RFC 9421) verification.
    ///
    /// Based on code from [httpsig-hyper](https://docs.rs/httpsig-hyper).
    pub async fn http_signature_handler(
        State(state): State<Arc<AppState>>,
        req: Request,
        next: Next,
    ) -> Response
    where
        Request: Sized,
    {
        log::debug!("httpsig: verifying signature(s) for request: {}", req.uri());

        let Ok(sig_headers) = req.extract_signature_headers_with_name().map_err(|err| {
            log::error!("httpsig: error parsing signature headers: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let mut pubkeys: Vec<(HttpPublicKey, Option<String>)> =
            Vec::with_capacity(sig_headers.len());

        for (_, sig_header) in sig_headers.iter() {
            let sig_name = sig_header.signature_name();

            if let Some(key_id) = sig_header.signature_params().keyid.as_deref() {
                let Ok(key_id) = Iri::try_from(key_id).map_err(|err| {
                    log::warn!("httpsig: {sig_name}: invalid key ID: {err}");
                }) else {
                    continue;
                };

                log::debug!("httpsig: {sig_name}: fetching key for ID: {key_id}");

                let Ok(pubkey) = state.fetch_key(&key_id).await.map_err(|err| {
                    log::error!(
                        "httpsig: {sig_name}: error finding record for key ID: {key_id}, error: {err}"
                    );
                }) else {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                };

                pubkeys.push((pubkey, Some(key_id.to_string())));
            } else {
                log::error!("httpsig: missing key ID for signature: {sig_name}");
                return StatusCode::BAD_REQUEST.into_response();
            }
        }

        match req.verify_message_signatures(pubkeys.iter().map(|(key, id)| (key, id.as_deref()))) {
            Ok(results) => {
                if results.iter().any(|r| r.is_ok()) {
                    if sig_headers
                        .iter()
                        .any(|(_, h)| h.signature_input_header_value().contains("content-digest"))
                    {
                        match req.verify_content_digest().await {
                            Ok(req) => {
                                log::debug!("httpsig: signature verified");
                                next.run(req).await
                            }
                            Err(err) => {
                                log::warn!("httpsig: invalid `content-digest`: {err}");
                                StatusCode::BAD_REQUEST.into_response()
                            }
                        }
                    } else {
                        log::debug!("httpsig: signature verified");
                        next.run(req).await
                    }
                } else {
                    log::error!("httpsig: no valid signature found");
                    log::debug!("httpsig: results: {results:?}");
                    StatusCode::BAD_REQUEST.into_response()
                }
            }
            Err(err) => {
                log::error!("httpsig: error validating signatures: {err}");
                StatusCode::BAD_REQUEST.into_response()
            }
        }
    }

    /// HTTP handler to fetch an actor record after a request's signature has been verified.
    pub async fn fetch_actor_handler(
        State(state): State<Arc<AppState>>,
        req: Request,
        next: Next,
    ) -> Response {
        if req.headers().get(header::AUTHORIZATION).is_some() {
            return next.run(req).await;
        }

        log::debug!("fetch_actor: fetching actor for request: {}", req.uri());

        let Ok(sig_headers) = req.extract_signature_headers_with_name().map_err(|err| {
            log::error!("httpsig: error parsing signature headers: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let mut actor_id: Option<Iri> = None;

        while let Some(key_id) = sig_headers
            .iter()
            .filter_map(|(_, h)| {
                h.signature_params()
                    .keyid
                    .as_deref()
                    .and_then(|k| Iri::try_from(k).ok())
            })
            .next()
            && actor_id.is_none()
        {
            log::debug!("fetch_actor: looking up actor for key ID: {key_id}");

            let actor_res = DbActor::find_by_key_id(&*state.db().await, &key_id).await;
            if let Ok(Some(actor)) = actor_res {
                log::debug!(
                    "fetch_actor: found actor for key ID: {key_id}, actor ID: {}",
                    actor.id()
                );
                // actor record already exists in the database, call next layer
                return next.run(req).await;
            } else if let Ok(Some(key)) = Key::find_by_key_id(&*state.db().await, &key_id).await {
                actor_id = Some(key.actor_id().clone());
            }
        }

        let Some(actor_id) = actor_id else {
            log::warn!("fetch_actor: no actor ID found");
            return StatusCode::NOT_FOUND.into_response();
        };

        log::debug!("fetch_actor: fetch actor for ID: {actor_id}");

        let Ok(res) = state
            .signed_request::<()>(Method::GET, &actor_id, None)
            .await
            .map_err(|err| {
                log::error!("router: error fetching actor for actor ID: {actor_id}, error: {err}");
            })
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(res) = res.text().await.map_err(|err| {
            log::error!("router: error fetch_actor response body: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(actor) = serde_json::from_str::<Actor>(&res).map_err(|err| {
            log::error!(
                "router: error parsing actor: {err}, request: {actor_id}, response body: {res}"
            );
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(actor) = DbActor::try_from_vocab(&*state.db().await, &actor)
            .await
            .map_err(|err| {
                log::error!("router: error storing actor for actor ID: {actor_id}, error: {err}");
            })
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        if let Err(err) = state
            .create_grant(&actor, &[Role::Visit, Role::Write], actor.table_entry())
            .await
        {
            log::error!("router: error creating actor grant: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        if let Err(err) = state
            .create_grant(
                &actor,
                &[Role::Visit, Role::Write],
                TableEntry::create(TableType::Inbox, actor.inbox()),
            )
            .await
        {
            log::error!("router: error creating actor inbox grant: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        if let Err(err) = state
            .create_grant(
                &actor,
                &[Role::Visit, Role::Write],
                TableEntry::create(TableType::Outbox, actor.outbox()),
            )
            .await
        {
            log::error!("router: error creating actor outbox grant: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        let app_actor = DbActor::application(state.app().clone());
        if let Err(err) = state
            .create_grant(
                &app_actor,
                &[Role::Visit, Role::Write, Role::Admin],
                actor.table_entry(),
            )
            .await
        {
            log::error!("router: error creating actor grant: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        log::debug!(
            "router: successfully stored actor record for actor ID: {actor_id} at UUID: {}",
            actor.uuid()
        );
        next.run(req).await
    }

    /// Handles OAuth-2.0 verification.
    ///
    /// Useful for endpoints consumed by federation clients using the C2S protocol.
    pub async fn oauth_verification_handler(
        State(state): State<Arc<AppState>>,
        req: Request,
        next: Next,
    ) -> Response
    where
        Request: Sized,
    {
        let Ok(pubkey) = state
            .signing_key(AlgorithmName::Ed25519)
            .await
            .map(|k| k.public_key())
            .map_err(|err| {
                log::error!("oauth: error retrieving verification key: {err}");
            })
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(claims) = req
            .headers()
            .get(header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .and_then(|h| h.strip_prefix("Bearer "))
            .ok_or(Error::http("oauth: invalid/missing bearer token"))
            .and_then(|t| OAuthToken::verify_token(pubkey.key(), t))
            .map_err(|err| {
                log::error!("oauth: error verifying token: {err}");
            })
        else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        if claims.is_expired() {
            log::error!("oauth: claim expired: {claims}");
            return StatusCode::UNAUTHORIZED.into_response();
        }

        next.run(req).await
    }

    /// Handles OAuth-2.0 or HTTP Message Signature (RFC 9421) verification.
    ///
    /// Useful for endpoints that may have client or server users.
    ///
    /// For example, `outbox` endpoints can be read by both clients and servers.
    /// Clients will submit OAuth-2.0 JWT tokens in the `Authorization` header.
    /// Servers will submit HTTP Message Signatures in the `Signature` + `Signature-Input` headers.
    ///
    /// The `Authorization` and `Signature` headers **SHOULD** never appear in the same request.
    pub async fn oauth_or_httpsig_handler(
        State(state): State<Arc<AppState>>,
        req: Request,
        next: Next,
    ) -> Response
    where
        Request: Sized,
    {
        log::debug!("middleware: request headers: {:?}", req.headers());

        if req.headers().get(header::AUTHORIZATION).is_some() {
            Self::oauth_verification_handler(State(state), req, next).await
        } else {
            Self::http_signature_handler(State(state), req, next).await
        }
    }
}
