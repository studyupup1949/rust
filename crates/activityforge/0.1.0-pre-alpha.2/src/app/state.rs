use std::sync::Arc;

use axum::extract::Request;
use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use chrono::Utc;
use http::{StatusCode, header};
use reqwest::Method;

use tokio::sync::{Mutex, MutexGuard};

use activitystreams_vocabulary::MimeType;

use crate::app::App;
use crate::app::oauth::{OAuthToken, OAuthTokenType, Scope};
use crate::crypto::{
    AlgorithmName, DigestAlgorithm, HttpContentDigest, HttpMessageComponentId,
    HttpMessageSignature, HttpPrivateKey, HttpSignatureHeadersMap, HttpSignatureParams, KeyType,
    PrivateKey, SymmetricKey,
};
use crate::db::{
    Actor as DbActor, Application as DbApplication, Db, DbConfig, Iri, Key, Name, TableEntry,
    TableType, Transaction,
};
use crate::{Error, Result, Role};

mod factory;
mod grant;
mod inbox;
mod keys;
mod oauth;
mod outbox;
mod person;
mod repository;

/// Represents the shared application state used by routers.
#[derive(Clone)]
pub struct AppState {
    db: Arc<Mutex<Db>>,
    uri: Iri,
    app: DbApplication,
}

impl AppState {
    /// Represents the maximum body length for incoming requests (in bytes);
    pub const MAX_REQUEST_LENGTH: usize = 1024 * 1024 * 4;

    /// Creates a new [AppState] from the provided database configuration.
    pub async fn create(config: DbConfig, uri: Iri, name: Name) -> Result<Self> {
        let db = Db::connect(config).await?;

        let (app_uuid, app_id) = TableType::Application.id_from_name(&uri, &name)?;

        let app = if let Some(app) = DbApplication::find_by_id(&db, &app_id).await? {
            app
        } else {
            let ed25519_key_uuid = db.rand_uuid();
            let ed25519_key_id = TableType::Key.id_from_uuid(&uri, ed25519_key_uuid)?;

            let ed25519_key = PrivateKey::random(KeyType::Ed25519)
                .and_then(Key::try_from)
                .map(|k| k.with_uuid(ed25519_key_uuid).with_id(ed25519_key_id))?;

            let rsa_key_uuid = db.rand_uuid();
            let rsa_key_id = TableType::Key.id_from_uuid(&uri, ed25519_key_uuid)?;

            let rsa_key = PrivateKey::random(KeyType::Rsa2048)
                .and_then(Key::try_from)
                .map(|k| k.with_uuid(rsa_key_uuid).with_id(rsa_key_id))?;

            DbApplication::builder(app_id, name)
                .and_then(|b| b.uuid(app_uuid))
                .and_then(|b| b.keys([ed25519_key, rsa_key]))?
                .build(&db)
                .await?
        };

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            uri,
            app,
        })
    }

    /// Gets a guarded reference to the database.
    pub async fn db(&self) -> MutexGuard<'_, Db> {
        self.db.lock().await
    }

    /// Clones the database connection.
    pub fn clone_db(&self) -> Arc<Mutex<Db>> {
        self.db.clone()
    }

    /// Gets a reference to the host URI.
    #[inline]
    pub const fn uri(&self) -> &Iri {
        &self.uri
    }

    /// Gets the default callback URI used for OAuth-2.0 authorization.
    #[inline]
    pub fn oauth_callback_uri(&self) -> Result<Iri> {
        App::oauth_callback_uri(self.uri())
    }

    /// Gets a reference to the [Application](DbApplication) database record.
    #[inline]
    pub const fn app(&self) -> &DbApplication {
        &self.app
    }

    /// Gets the maximum body length accepted for incoming requests.
    // TODO: make configurable
    #[inline]
    pub const fn max_request_length(&self) -> usize {
        Self::MAX_REQUEST_LENGTH
    }

    /// Attempts to fetch a key from the database.
    ///
    /// If the key is not found, creates a new signing key.
    pub async fn signing_key(&self, algo: AlgorithmName) -> Result<HttpPrivateKey> {
        let db = self.db().await;
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let key = self.signing_key_tx(&mut dbtx, &db_key, algo).await?;

        dbtx.commit()
            .await
            .map(|_| key)
            .map_err(|err| Error::http(format!("signing_key: {err}")))
    }

    /// Attempts to fetch a key from the database.
    ///
    /// If the key is not found, creates a new signing key.
    pub async fn signing_key_tx(
        &self,
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        algo: AlgorithmName,
    ) -> Result<HttpPrivateKey> {
        let key_type = KeyType::try_from(&algo)?;

        self.app
            .find_key_by_type_tx(dbtx, db_key, key_type)
            .await
            .and_then(|k| {
                k.ok_or(Error::db(format!(
                    "app: missing signing key for key type: {key_type}"
                )))
            })
            .and_then(HttpPrivateKey::try_from)
    }

    /// Creates a signed request to a remote host.
    ///
    /// Uses HTTP Message Signatures (RFC 9421) to create a signed request.
    pub async fn signed_request<S: serde::ser::Serialize>(
        &self,
        method: Method,
        uri: &Iri,
        body: Option<&S>,
    ) -> Result<reqwest::Response> {
        let ed25519_key = self.signing_key(AlgorithmName::Ed25519).await?;
        let rsa_key = self.signing_key(AlgorithmName::RsaV1_5Sha256).await?;

        Self::signed_request_with_keys(&[ed25519_key, rsa_key], method, uri, body).await
    }

    /// Creates a signed request to a remote host.
    ///
    /// Uses HTTP Message Signatures (RFC 9421) to create a signed request.
    pub async fn signed_request_tx<S: serde::ser::Serialize>(
        &self,
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        method: Method,
        uri: &Iri,
        body: Option<&S>,
    ) -> Result<reqwest::Response> {
        let ed25519_key = self
            .signing_key_tx(dbtx, db_key, AlgorithmName::Ed25519)
            .await?;
        let rsa_key = self
            .signing_key_tx(dbtx, db_key, AlgorithmName::RsaV1_5Sha256)
            .await?;

        Self::signed_request_with_keys(&[ed25519_key, rsa_key], method, uri, body).await
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
        log::debug!("signed_request: creating signed request for URI: {uri}, method: {method}");

        let date = Utc::now();
        let timestamp = date.timestamp();
        let date_str = date.to_rfc3339();

        let http_uri = http::Uri::try_from(uri.as_str())
            .map_err(|err| Error::http(format!("invalid uri: {err}")))?;

        let mut params = vec![
            HttpMessageComponentId::try_from("date")?,
            HttpMessageComponentId::try_from("@method")?,
            HttpMessageComponentId::try_from("@path")?,
        ];

        if http_uri.query().is_some() {
            HttpMessageComponentId::try_from("@query").map(|p| params.push(p))?;
        }

        let req_builder = http::Request::builder()
            .method(method.as_str())
            .uri(uri.as_str())
            .header("date", date_str);

        let mut req = if let Some(body) = body {
            params.append(&mut vec![
                HttpMessageComponentId::try_from("content-type")?,
                HttpMessageComponentId::try_from("content-digest")?,
            ]);

            let body_json = serde_json::to_string(body)?;

            req_builder
                .header("content-type", MimeType::ApplicationActivityJson.as_str())
                .body(Bytes::from(body_json))?
                .set_content_digest(DigestAlgorithm::Sha256)
                .await?
                .set_content_digest(DigestAlgorithm::Sha512)
                .await?
        } else {
            req_builder.body(Bytes::new())?
        };

        let mut sig_params = Vec::new();
        for (i, key) in keys.iter().enumerate() {
            let mut p = HttpSignatureParams::try_new(&params)?;
            let created = u64::try_from(timestamp)
                .map_err(|err| Error::http(format!("invalid created: {err}")))?;

            p.set_created(created);
            p.set_keyid(key.key_id());
            sig_params.push((p, key, Some(format!("sig{}", i + 1))));
        }

        let msg_params = sig_params
            .iter()
            .map(|&(ref p, k, ref s)| (p, k, s.as_deref()))
            .collect::<Vec<_>>();
        req.set_message_signatures(msg_params.as_slice())?;

        let client = reqwest::Client::new();

        client
            .request(method, uri.as_str())
            .headers(req.headers().clone())
            .body(req.body().to_vec())
            .send()
            .await
            .map_err(Error::from)
    }

    /// Gets an actor record associated with the key ID(s) in the request signature headers.
    pub async fn get_actor_by_key_id(
        &self,
        ctx: &str,
        sig_headers: &HttpSignatureHeadersMap,
    ) -> Result<DbActor> {
        let mut actor: Option<DbActor> = None;

        while let Some(key_id) = sig_headers
            .iter()
            .filter_map(|(_, h)| {
                h.signature_params()
                    .keyid
                    .as_deref()
                    .and_then(|k| Iri::try_from(k).ok())
            })
            .next()
            && actor.is_none()
        {
            log::debug!("{ctx}: looking up signing actor for key ID: {key_id}");

            if let Ok(a) = DbActor::find_by_key_id(&*self.db().await, &key_id)
                .await
                .map_err(|err| {
                    log::warn!("{ctx}: error looking up actor for key ID: {key_id}, error: {err}");
                })
            {
                actor = a;
            }
        }

        actor.ok_or(Error::http(format!("{ctx}: no actor found")))
    }

    /// Middleware helper function to check a request is authorized to access a given resource.
    pub async fn check_authorization<M>(
        self: Arc<Self>,
        req: Request,
        table_entry: TableEntry,
        scope_matcher: M,
        role: Role,
    ) -> core::result::Result<(Request, DbActor), Response>
    where
        M: FnOnce(&[Scope]) -> bool,
    {
        let table = table_entry.table();
        let req_uri = req.uri();

        if req.headers().get(header::AUTHORIZATION).is_some() {
            let token = req
                .headers()
                .get(header::AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|h| h.strip_prefix("Bearer "))
                .ok_or_else(|| {
                    log::error!("{table}: oauth: missing/invalid authz header");
                    StatusCode::BAD_REQUEST.into_response()
                })?;

            let oauth_token = self
                .find_oauth_token(token, OAuthTokenType::Access)
                .await
                .and_then(|t| t.ok_or(Error::db(format!("missing OAuth token: {req_uri}"))))
                .map_err(|err| {
                    log::error!(
                        "{table}: oauth: no token record found for: {req_uri}, error: {err}"
                    );
                    StatusCode::BAD_REQUEST.into_response()
                })?;

            let scopes = oauth_token.scope().ok_or_else(|| {
                log::error!("{table}: oauth: token: missing scopes for token: {req_uri}");
                StatusCode::UNAUTHORIZED.into_response()
            })?;

            if scope_matcher(scopes) {
                OAuthToken::find_owner_by_token(&*self.db().await, token, OAuthTokenType::Access)
                    .await
                    .and_then(|r| r.ok_or(Error::db("{table}: oauth: missing token owner")))
                    .map(|p| (req, DbActor::person(p)))
                    .map_err(|err| {
                        log::error!("{table}: oauth: {err}");
                        StatusCode::INTERNAL_SERVER_ERROR.into_response()
                    })
            } else {
                log::error!("{table}: oauth: token: invalid scopes for token: {req_uri}");
                Err(StatusCode::UNAUTHORIZED.into_response())
            }
        } else {
            let sig_headers = req.extract_signature_headers_with_name().map_err(|err| {
                log::error!("{table}: httpsig: error parsing signature headers: {err}");
                StatusCode::BAD_REQUEST.into_response()
            })?;

            let actor = self
                .get_actor_by_key_id(table_entry.table().as_str(), &sig_headers)
                .await
                .map_err(|err| {
                    log::error!("{table}: httpsig: signing actor not found: {err}");
                    StatusCode::UNAUTHORIZED.into_response()
                })?;

            self.check_grants(&actor, role, table_entry)
                .await
                .map(|_| (req, actor))
                .map_err(|err| {
                    log::error!("{table}: httpsig: {err}");
                    StatusCode::UNAUTHORIZED.into_response()
                })
        }
    }
}
