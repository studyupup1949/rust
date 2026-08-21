//! Provides an OAuth-2.0 server implementation.

use std::sync::Arc;
use std::time::Duration;

use activitystreams_vocabulary::MimeType;
use axum::body::Body;
use axum::extract::{FromRequest, Request, State};
use axum::response::{IntoResponse, Response};
use chrono::Utc;
use http::StatusCode;
use oauth_async::endpoint::access_token::AccessTokenFlow;
use oauth_async::endpoint::authorization::AuthorizationFlow;
use oauth_async::endpoint::refresh::RefreshFlow;
use oauth_axum::OAuthRequest;

use crate::Error;
use crate::app::oauth::{OAuthEndpoint, OAuthError};
use crate::app::{App, AppState, Credentials};
use crate::crypto::{AlgorithmName, Nonce};
use crate::db::{Iri, Name, Person};

mod authorizer;
mod claims;
mod client;
mod endpoint;
mod error;
mod grant;
mod issuer;
mod owner_solicitor;
mod parameters;
mod pkce;
mod registrar;
mod scope;
mod token;

pub use authorizer::*;
pub use claims::*;
pub use client::*;
pub use endpoint::*;
pub use error::*;
pub use grant::*;
pub use issuer::*;
pub use owner_solicitor::*;
pub use parameters::*;
pub use pkce::*;
pub use registrar::*;
pub use scope::*;
pub use token::*;

/// Convenience alias for an opaque error from the `oxide-auth` crate.
///
/// NOTE: placeholder useful for a transition to an unreleased crate version where this becomes a new-type.
pub type OAuthOpaqueError = ();

/// Convenience function for returning an opaque error from the `oxide-auth` crate.
///
/// NOTE: placeholder useful for a transition to an unreleased crate version where this becomes a new-type.
#[inline]
pub const fn opaque_error() -> OAuthOpaqueError {}

impl App {
    /// Performs OAuth-2.0 authentication of local user accounts.
    ///
    /// This endpoint is used to issue a JWT bearer token for registering a new OAuth-2.0 client.
    ///
    /// Future work could also implement remote authentication flows, TBD.
    pub async fn oauth_authenticate(State(state): State<Arc<AppState>>, req: Request) -> Response {
        let Ok(creds) = req
            .headers()
            .get("Authorization")
            .ok_or(Error::http(
                "oauth: authenticate: missing Authorization header",
            ))
            .and_then(|v| {
                v.to_str()
                    .map_err(|err| {
                        Error::http(format!("oauth: authenticate: invalid basic-auth: {err}"))
                    })
                    .and_then(Credentials::from_basic_auth)
            })
            .map_err(|err| {
                log::error!("oauth: authenticate: {err}");
            })
        else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Ok(username) = Name::try_from(creds.username()).map_err(|err| {
            log::error!("oauth: authenticate: invalid username: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let db = state.db().await;
        let Ok(pool) = db.pool().map_err(|err| {
            log::error!("oauth: authenticate: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let Ok(db_key) = db.key().map_err(|err| {
            log::error!("oauth: authenticate: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let Ok(mut dbtx) = pool.begin().await.map_err(|err| {
            log::error!("oauth: authenticate: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(Some(person)) = Person::find_by_name_tx(&mut dbtx, &username)
            .await
            .map_err(|err| {
                log::error!("oauth: authenticate: user not found: {username}, error: {err}");
            })
        else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        let Some(password) = person.password() else {
            log::error!("oauth: authenticate: missing password for: {username}");
            return StatusCode::BAD_REQUEST.into_response();
        };

        if let Err(err) = password.verify(username.as_str(), creds.password().as_bytes()) {
            log::error!("oauth: authenticate: invalid credentials: {err}");
            return StatusCode::UNAUTHORIZED.into_response();
        }

        let duration = Duration::from_secs(600);
        let expires = Utc::now() + duration;
        let claims = OAuthClaims::new()
            .with_issuer("activityforge")
            .with_authenticate_subject(username)
            .with_expires(expires.timestamp() as u64)
            .with_nonce(Nonce::random());

        let Ok(key) = state
            .signing_key_tx(&mut dbtx, &db_key, AlgorithmName::Ed25519)
            .await
            .map_err(|err| {
                log::error!("oauth: authenticate: {err}");
            })
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(token) = OAuthToken::sign_token(key.key(), &claims).map_err(|err| {
            log::error!("oauth: authenticate: error signing JWT token: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(redirect_uri) =
            Iri::try_from(format!("{}/oauth/register", state.uri())).map_err(|err| {
                log::error!("oauth: authenticate: invalid grant redirect URI: {err}");
            })
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(tag) = OAuthGrant::create_tag(db_key.as_ref()).map_err(|err| {
            log::error!("oauth: authenticate: error creating grant tag: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(mut oauth_grant) = OAuthGrant::new()
            .with_owner_id(person.id())
            .with_client_id(db.rand_uuid())
            .with_redirect_uri(redirect_uri)
            .with_until(expires)
            .with_tag(tag)
            .with_scopes([Scope::Register])
            .map_err(|err| {
                log::error!("oauth: authenticate: error creating grant: {err}");
            })
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(grant_id) = oauth_grant.insert_tx(&mut dbtx).await.map_err(|err| {
            log::error!("oauth: authenticate: error storing registration grant: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let mut oauth_token = OAuthToken::new()
            .with_token(token)
            .with_until(expires)
            .with_token_type(OAuthTokenType::Register)
            .with_grant_id(grant_id);

        if let Err(err) = oauth_token.insert_tx(&mut dbtx).await {
            log::error!("oauth: authenticate: error storing registration token: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        if let Err(err) = dbtx.commit().await {
            log::error!("oauth: authenticate: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        let token_json = oauth_token.to_string();

        Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", MimeType::ApplicationJson.as_str())
            .header("content-length", token_json.len())
            .body(Body::new(token_json))
            .unwrap_or_else(|err| {
                log::error!("oauth: authenticate: error creating response: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }

    /// Performs [OAuth 2.0 Dynamic Client Registration](https://www.rfc-editor.org/rfc/rfc7591).
    ///
    /// Currently, used to register a new OAuth-2.0 client for local user accounts.
    ///
    /// Endpoint users **MUST** first call the [oauth_authenticate](Self::oauth_authenticate)
    /// endpoint to authenticate the user, and receive a valid JWT bearer token.
    pub async fn oauth_register(State(state): State<Arc<AppState>>, req: Request) -> Response {
        let Ok(token) = req
            .headers()
            .get("Authorization")
            .ok_or(Error::http("oauth: register: missing authorization header"))
            .and_then(|h| {
                h.to_str().map_err(|err| {
                    Error::http(format!(
                        "oauth: authenticate: invalid header encoding: {err}"
                    ))
                })
            })
            .and_then(|h| {
                h.strip_prefix("Bearer ")
                    .ok_or(Error::http("oauth: authenticate: invalid bearer token"))
            })
            .map_err(|err| {
                log::error!("{err}");
            })
        else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let db = state.db().await;
        let Ok(pool) = db.pool().map_err(|err| {
            log::error!("oauth: register: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let Ok(db_key) = db.key().map_err(|err| {
            log::error!("oauth: register: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };
        let Ok(mut dbtx) = pool.begin().await.map_err(|err| {
            log::error!("oauth: register: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(key) = state
            .signing_key_tx(&mut dbtx, &db_key, AlgorithmName::Ed25519)
            .await
            .map(|k| k.public_key())
            .map_err(|err| {
                log::error!("oauth: authenticate: {err}");
            })
        else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let Ok(claims) = OAuthToken::verify_token(key.key(), token).map_err(|err| {
            log::error!("oauth: register: invalid token: {err}");
        }) else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        if claims.is_expired() {
            log::error!("oauth: register: authentication token expired");
            return StatusCode::UNAUTHORIZED.into_response();
        }

        let Ok(auth_subject) = claims.authenticate_subject().map_err(|err| {
            log::error!("oauth: register: {err}");
        }) else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        let Ok(Some(db_token)) =
            OAuthToken::find_by_token_tx(&mut dbtx, token, OAuthTokenType::Register)
                .await
                .map_err(|err| {
                    log::error!("oauth: register: error finding token: {err}");
                })
        else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        let Ok(db_grant) = OAuthGrant::get_tx(&mut dbtx, &db_token.grant_id())
            .await
            .map_err(|err| {
                log::error!("oauth: register: error finding token: {err}");
            })
        else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        let Ok(Some(db_person)) = Person::find_by_id_tx(&mut dbtx, db_grant.owner_id())
            .await
            .map_err(|err| {
                log::error!("oauth: register: error finding grant owner: {err}");
            })
        else {
            return StatusCode::UNAUTHORIZED.into_response();
        };

        let person_name = db_person.name().as_str();
        if db_person.name().as_str() != auth_subject {
            log::error!(
                "oauth: register: invalid subject: {person_name}, expected: {auth_subject}"
            );
            return StatusCode::UNAUTHORIZED.into_response();
        }

        if let Err(err) = db_grant.delete_tx(&mut dbtx).await {
            log::error!("oauth: register: error deleting grant and token: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        let Ok(body) = axum::body::to_bytes(req.into_body(), 1024 * 1024)
            .await
            .map_err(|err| {
                log::error!("oauth: register: error reading body: {err}");
            })
        else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Ok(body_str) = str::from_utf8(body.as_ref()).map_err(|err| {
            log::error!("oauth: register: invalid body encoding: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let Ok(client_req) = serde_json::from_str::<OAuthClientRegister>(body_str).map_err(|err| {
            log::error!("oauth: register: invalid client request: {err}");
        }) else {
            return StatusCode::BAD_REQUEST.into_response();
        };

        let client_secret = ClientSecret::random();

        let Ok(db_client) = OAuthClient::try_from_register_tx(
            &mut dbtx,
            &db_key,
            state.uri(),
            db_person.uuid(),
            &client_secret,
            &client_req,
        )
        .await
        .map_err(|err| {
            log::error!("oauth: register: error parsing request: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        if let Err(err) = dbtx.commit().await {
            log::error!("oauth: register: {err}");
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }

        let Ok(redirect_uri) = state.oauth_callback_uri().map_err(|err| {
            log::error!("oauth: register: {err}");
        }) else {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        };

        let client_json = OAuthClientResponse::new()
            .with_client_id(db_client.uuid())
            .with_client_id_issued_at(db_client.issued_at().timestamp() as u64)
            .with_client_secret(client_secret)
            .with_client_secret_expires_at(0u64)
            .with_redirect_uris([redirect_uri])
            .with_grant_types([
                OAuthGrantType::AuthorizationCode,
                OAuthGrantType::RefreshToken,
            ])
            .with_token_endpoint_auth_method(EndpointAuthMethod::ClientSecretBasic)
            .to_string();

        Response::builder()
            .status(StatusCode::CREATED)
            .header("Content-Type", MimeType::ApplicationJson.as_str())
            .header("Content-Length", client_json.len())
            .body(Body::new(client_json))
            .unwrap_or_else(|err| {
                log::error!("oauth: register: error creating response: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }

    /// Performs [OAuth 2.0 Authorization](https://www.rfc-editor.org/rfc/rfc6749).
    ///
    /// Endpoint users **MUST** first call (once-per-client):
    ///
    /// - the [oauth_authenticate](Self::oauth_authenticate) endpoint to authenticate the user
    /// - the [oauth_register](Self::oauth_register) endpoint to register the OAuth-2.0 client
    ///
    /// Once a client is registered, [oauth_authorize](Self::oauth_authorize) can be called directly.
    pub async fn oauth_authorize(State(state): State<Arc<AppState>>, req: Request) -> Response {
        let Ok(key_id) = state
            .signing_key(AlgorithmName::Ed25519)
            .await
            .and_then(|k| Iri::try_from(k.key_id()))
            .map_err(|err| {
                log::error!("oauth: authz: {err}");
            })
        else {
            return Self::oauth_error(OAuthError::ServerError);
        };

        let endpoint = OAuthEndpoint::new(&state.clone_db(), state.uri().clone(), key_id);

        let Ok(mut authz_flow) = AuthorizationFlow::prepare(endpoint).map_err(|err| {
            log::error!("oauth: authz: {err}");
        }) else {
            return Self::oauth_error(OAuthError::ServerError);
        };

        let Ok(oauth_req) = OAuthRequest::from_request(req, &state)
            .await
            .map_err(|err| {
                log::error!("oauth: authz: {err}");
            })
        else {
            return Self::oauth_error(OAuthError::InvalidRequest);
        };

        let Ok(res) = authz_flow.execute(oauth_req).await.map_err(|err| {
            log::error!("oauth: authz: {err}");
        }) else {
            return Self::oauth_error(OAuthError::UnauthorizedClient);
        };

        res.into_response()
    }

    /// Performs [OAuth 2.0 Authorization callback](https://www.rfc-editor.org/rfc/rfc6749).
    ///
    /// Automatically redirects from the [oauth_authorize](Self::oauth_authorize) endpoint for
    /// clients that do not have a `redirect_uri` set, typically "confidential/private" clients.
    pub async fn oauth_callback(State(state): State<Arc<AppState>>, req: Request) -> Response {
        let app_uri = state.uri();
        let req_uri = format!("{app_uri}{}", req.uri());

        let Ok(url) = reqwest::Url::parse(&req_uri).map_err(|err| {
            log::error!("oauth: token: error parsing redirect URI: {req_uri}, error: {err}");
        }) else {
            return Self::oauth_error(OAuthError::InvalidRequest);
        };

        let Some(code) = url
            .query_pairs()
            .find(|(k, _)| k.as_ref() == "code")
            .map(|(_, v)| v)
        else {
            log::error!("oauth: token: missing authz code");
            return Self::oauth_error(OAuthError::InvalidRequest);
        };

        let body = AuthorizationCode::new().with_code(code).to_string();

        match Response::builder()
            .status(StatusCode::OK)
            .header("Content-Type", MimeType::ApplicationJson.as_str())
            .header("Content-Length", body.len())
            .body(Body::new(body))
        {
            Ok(res) => res,
            Err(err) => {
                log::error!("oauth: token: error building response: {err}");
                Self::oauth_error(OAuthError::ServerError)
            }
        }
    }

    /// Performs [OAuth 2.0 Token Issuing](https://www.rfc-editor.org/rfc/rfc6749).
    ///
    /// Endpoint users **MUST** first call (once-per-client):
    ///
    /// - the [oauth_authenticate](Self::oauth_authenticate) endpoint to authenticate the user
    /// - the [oauth_register](Self::oauth_register) endpoint to register the OAuth-2.0 client
    /// - the [oauth_authorize](Self::oauth_authorize) endpoint to authorize the OAuth-2.0 client
    ///
    /// Once a client is registered, [oauth_authorize](Self::oauth_authorize) can be called directly.
    pub async fn oauth_token(State(state): State<Arc<AppState>>, req: Request) -> Response {
        let Ok(key_id) = state
            .signing_key(AlgorithmName::Ed25519)
            .await
            .and_then(|k| Iri::try_from(k.key_id()))
            .map_err(|err| {
                log::error!("oauth: token: {err}");
            })
        else {
            return Self::oauth_error(OAuthError::ServerError);
        };

        let endpoint = OAuthEndpoint::new(&state.clone_db(), state.uri().clone(), key_id);
        let Ok(mut token_flow) = AccessTokenFlow::prepare(endpoint).map_err(|err| {
            log::error!("oauth: token: {err}");
        }) else {
            return Self::oauth_error(OAuthError::ServerError);
        };

        let Ok(oauth_req) = OAuthRequest::from_request(req, &state)
            .await
            .map_err(|err| {
                log::error!("oauth: token: {err}");
            })
        else {
            return Self::oauth_error(OAuthError::InvalidRequest);
        };

        log::debug!("oauth: token: executing token flow");

        let Ok(res) = token_flow.execute(oauth_req).await.map_err(|err| {
            log::error!("oauth: token: {err}");
        }) else {
            return Self::oauth_error(OAuthError::UnauthorizedClient);
        };

        res.into_response()
    }

    /// Performs [OAuth 2.0 Token Refresh](https://www.rfc-editor.org/rfc/rfc6749).
    ///
    /// Endpoint users **MUST** first call (once-per-client):
    ///
    /// - the [oauth_authenticate](Self::oauth_authenticate) endpoint to authenticate the user
    /// - the [oauth_register](Self::oauth_register) endpoint to register the OAuth-2.0 client
    ///
    /// Endpoint users will then get an initial access and refresh token from:
    ///
    /// - the [oauth_authorize](Self::oauth_authorize) endpoint to authorize the OAuth-2.0 client
    pub async fn oauth_refresh(State(state): State<Arc<AppState>>, req: Request) -> Response {
        let Ok(key_id) = state
            .signing_key(AlgorithmName::Ed25519)
            .await
            .and_then(|k| Iri::try_from(k.key_id()))
            .map_err(|err| {
                log::error!("oauth: refresh: {err}");
            })
        else {
            return Self::oauth_error(OAuthError::ServerError);
        };

        let endpoint = OAuthEndpoint::new(&state.clone_db(), state.uri().clone(), key_id);
        let Ok(mut refresh_flow) = RefreshFlow::prepare(endpoint).map_err(|err| {
            log::error!("oauth: refresh: {err}");
        }) else {
            return Self::oauth_error(OAuthError::ServerError);
        };

        let Ok(oauth_req) = OAuthRequest::from_request(req, &state)
            .await
            .map_err(|err| {
                log::error!("oauth: refresh: {err}");
            })
        else {
            return Self::oauth_error(OAuthError::InvalidRequest);
        };

        log::debug!("oauth: refresh: executing refresh flow");

        let Ok(res) = refresh_flow.execute(oauth_req).await.map_err(|err| {
            log::error!("oauth: refresh: {err}");
        }) else {
            return Self::oauth_error(OAuthError::UnauthorizedClient);
        };

        res.into_response()
    }

    /// Creates an OAuth-2.0 error response.
    pub(crate) fn oauth_error(error: OAuthError) -> Response {
        let body = format!("error={error}");

        Response::builder()
            .status(error.status())
            .header(
                "Content-Type",
                MimeType::ApplicationWwwFormUrlEncoded.as_str(),
            )
            .header("Content-Length", body.len())
            .body(Body::new(body))
            .unwrap_or_else(|err| {
                log::error!("oauth: error creating response: {err}");
                StatusCode::INTERNAL_SERVER_ERROR.into_response()
            })
    }
}
