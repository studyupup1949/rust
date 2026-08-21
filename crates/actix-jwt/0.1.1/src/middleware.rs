//! Central JWT authentication middleware for actix-web.
//!
//! This module is a Rust port of
//! [`EchoJWTMiddleware`](https://github.com/LdDl/echo-jwt/blob/master/auth_jwt.go)
//! from the Go implementation.

use std::collections::HashMap;
use std::future::{Future, Ready, ready};
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use actix_web::cookie::{Cookie, SameSite};
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header;
use actix_web::{HttpMessage, HttpRequest, HttpResponse, HttpResponseBuilder};
use base64::Engine;
use base64::engine::general_purpose::URL_SAFE;
use chrono::{DateTime, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, TokenData, Validation};
use serde_json::Value;
use tracing::warn;

use crate::core::{Token, TokenStore};
use crate::errors::JwtError;
use crate::store::InMemoryRefreshTokenStore;

/// JWT claims map stored in [`HttpRequest`] extensions by the middleware.
///
/// Retrieve it in a handler via [`extract_claims`].
#[derive(Debug, Clone)]
pub struct JwtPayload(pub HashMap<String, Value>);

/// Raw JWT token string stored in [`HttpRequest`] extensions.
///
/// Retrieve it in a handler via [`get_token`].
#[derive(Debug, Clone)]
pub struct JwtTokenString(pub String);

/// Identity value stored in [`HttpRequest`] extensions.
///
/// Retrieve it in a handler via [`get_identity`].
#[derive(Debug, Clone)]
pub struct JwtIdentity(pub Value);

/// Central JWT authentication middleware for actix-web.
///
/// A full-featured Rust port of
/// [`EchoJWTMiddleware`](https://github.com/LdDl/echo-jwt/blob/master/auth_jwt.go)
/// from the Go implementation,
/// providing:
///
/// * Login / logout / refresh handlers with token rotation.
/// * Access + refresh token generation ([RFC 6749]).
/// * Token extraction from header, query, cookie, path param, form.
/// * Cookie management (access + refresh).
/// * Skipper, BeforeFunc, SuccessHandler, ErrorHandler (labstack features).
/// * HMAC and RSA signing (with optional passphrase-protected private keys).
///
/// # Lifecycle
///
/// 1. Create via [`ActixJwtMiddleware::new`].
/// 2. Configure fields (key, callbacks, etc.).
/// 3. Call [`init`](Self::init) to validate and prepare keys.
/// 4. Wrap in `Arc`, share via `web::Data` and register routes / middleware.
///
/// [RFC 6749]: https://datatracker.ietf.org/doc/html/rfc6749
///
/// # Examples
///
/// ```rust,no_run
/// use std::sync::Arc;
/// use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
/// use actix_jwt::{ActixJwtMiddleware, extract_claims, JwtError};
///
/// #[actix_web::main]
/// async fn main() -> std::io::Result<()> {
///     let mut jwt = ActixJwtMiddleware::new();
///     jwt.key = b"secret".to_vec();
///     jwt.authenticator = Some(Arc::new(|_req, body| {
///         #[derive(serde::Deserialize)]
///         struct L { username: String, password: String }
///         let result = serde_json::from_slice::<L>(body)
///             .map(|c| serde_json::json!({"user": c.username}))
///             .map_err(|_| JwtError::MissingLoginValues);
///         Box::pin(async move { result })
///     }));
///     jwt.init().unwrap();
///
///     let jwt = Arc::new(jwt);
///     let jwt_data = web::Data::new(jwt.clone());
///
///     HttpServer::new(move || {
///         App::new()
///             .app_data(jwt_data.clone())
///             .route("/login", web::post().to({
///                 let j = jwt.clone();
///                 move |req: HttpRequest, body: web::Bytes| {
///                     let j = j.clone();
///                     async move { j.login_handler(&req, &body).await }
///                 }
///             }))
///             .service(
///                 web::scope("/api")
///                     .wrap(jwt.middleware())
///                     .route("/me", web::get().to(|req: HttpRequest| async move {
///                         HttpResponse::Ok().json(extract_claims(&req))
///                     })),
///             )
///     })
///     .bind("127.0.0.1:8080")?
///     .run()
///     .await
/// }
/// ```
pub struct ActixJwtMiddleware {
    /// `WWW-Authenticate` realm value.
    pub realm: String,
    /// Key used to store the identity value in request extensions.
    pub identity_key: String,

    /// Signing algorithm name (`"HS256"`, `"RS256"`, etc.).
    pub signing_algorithm: String,
    /// HMAC secret key bytes (used when `signing_algorithm` is `HS*`).
    pub key: Vec<u8>,
    /// Optional callback for multi-key (KID) support. When set, `key` and
    /// RSA key fields are ignored for **decoding**.
    pub key_func:
        Option<Arc<dyn Fn(&jsonwebtoken::Header) -> Result<DecodingKey, JwtError> + Send + Sync>>,

    /// Access token lifetime (default: 1 hour).
    pub timeout: Duration,
    /// Optional per-user timeout override based on the identity payload.
    pub timeout_func: Option<Arc<dyn Fn(&Value) -> Duration + Send + Sync>>,
    /// Maximum duration for which a token can be refreshed (0 = disabled).
    pub max_refresh: Duration,
    /// Clock function (override for testing).
    pub time_func: Arc<dyn Fn() -> DateTime<Utc> + Send + Sync>,

    /// Validates login credentials and returns user data on success.
    ///
    /// The callback receives the request and raw body bytes, and returns a
    /// future that resolves to the authenticated user data (or an error).
    /// Extract what you need from the arguments synchronously, then return
    /// an async block that owns all captured data.
    pub authenticator: Option<
        Arc<
            dyn Fn(
                    &HttpRequest,
                    &[u8],
                )
                    -> Pin<Box<dyn Future<Output = Result<Value, JwtError>> + Send>>
                + Send
                + Sync,
        >,
    >,
    /// Decides whether the authenticated identity is allowed to proceed.
    pub authorizer: Arc<dyn Fn(&HttpRequest, &Value) -> bool + Send + Sync>,
    /// Maps user data to custom JWT claims.
    pub payload_func: Option<Arc<dyn Fn(&Value) -> HashMap<String, Value> + Send + Sync>>,
    /// Extracts the identity value from request extensions.
    pub identity_handler: Arc<dyn Fn(&HttpRequest) -> Option<Value> + Send + Sync>,

    /// Builds the "unauthorized" HTTP response.
    pub unauthorized: Arc<dyn Fn(&HttpRequest, u16, &str) -> HttpResponse + Send + Sync>,
    /// Builds the login success response.
    pub login_response: Arc<dyn Fn(&HttpRequest, &Token) -> HttpResponse + Send + Sync>,
    /// Builds the logout success response.
    pub logout_response: Arc<dyn Fn(&HttpRequest) -> HttpResponse + Send + Sync>,
    /// Builds the refresh success response.
    pub refresh_response: Arc<dyn Fn(&HttpRequest, &Token) -> HttpResponse + Send + Sync>,
    /// Maps a [`JwtError`] to a human-readable message for the response body.
    pub http_status_message_func: Arc<dyn Fn(&HttpRequest, &JwtError) -> String + Send + Sync>,

    /// Comma-separated list of `"source:name"` pairs (e.g.
    /// `"header:Authorization,query:token"`).
    pub token_lookup: String,
    /// Expected prefix in the `Authorization` header (default: `"Bearer"`).
    pub token_head_name: String,
    /// Name of the expiration claim (default: `"exp"`).
    pub exp_field: String,

    /// Path to an RSA private key PEM file.
    pub priv_key_file: Option<String>,
    /// RSA private key PEM bytes (alternative to file).
    pub priv_key_bytes: Option<Vec<u8>>,
    /// Path to an RSA public key PEM file.
    pub pub_key_file: Option<String>,
    /// RSA public key PEM bytes (alternative to file).
    pub pub_key_bytes: Option<Vec<u8>>,
    /// Passphrase for encrypted PKCS#8 private keys.
    pub private_key_passphrase: Option<String>,
    encoding_key: Option<EncodingKey>,
    decoding_key: Option<DecodingKey>,

    /// When `true`, access and refresh tokens are also sent as cookies.
    pub send_cookie: bool,
    /// Max-Age for the access-token cookie.
    pub cookie_max_age: Duration,
    /// Sets the `Secure` flag on cookies.
    pub secure_cookie: bool,
    /// Sets the `HttpOnly` flag on cookies.
    pub cookie_http_only: bool,
    /// Optional domain for cookies.
    pub cookie_domain: Option<String>,
    /// Name of the access-token cookie (default: `"jwt"`).
    pub cookie_name: String,
    /// `SameSite` attribute for cookies (default: `Lax`).
    pub cookie_same_site: SameSite,
    /// When `true`, the validated token is echoed back in the response
    /// `Authorization` header.
    pub send_authorization: bool,

    /// Refresh-token lifetime (default: 30 days).
    pub refresh_token_timeout: Duration,
    /// Cookie name for the refresh token (default: `"refresh_token"`).
    pub refresh_token_cookie_name: String,
    /// Length of the random refresh-token bytes before base64 encoding.
    pub refresh_token_length: usize,
    /// Pluggable storage backend for refresh tokens.
    pub refresh_token_store: Arc<dyn TokenStore>,

    /// When this returns `true` for a request the middleware is bypassed
    /// entirely (labstack feature).
    pub skipper: Option<Arc<dyn Fn(&ServiceRequest) -> bool + Send + Sync>>,
    /// Called before token extraction (labstack feature).
    pub before_func: Option<Arc<dyn Fn(&ServiceRequest) + Send + Sync>>,
    /// Called after successful token validation (labstack feature).
    pub success_handler: Option<Arc<dyn Fn(&HttpRequest) -> Result<(), JwtError> + Send + Sync>>,
    /// Intercepts errors; returning `None` suppresses the error (labstack
    /// feature).
    pub error_handler:
        Option<Arc<dyn Fn(&HttpRequest, JwtError) -> Option<JwtError> + Send + Sync>>,
    /// When `true` **and** `error_handler` returns `None`, the request is
    /// forwarded to the inner service instead of being rejected (hybrid /
    /// public+auth routes).
    pub continue_on_ignored_error: bool,
}

impl ActixJwtMiddleware {
    /// Creates a new middleware instance with sensible defaults.
    ///
    /// Mirrors
    /// [`MiddlewareInit`](https://github.com/LdDl/echo-jwt/blob/master/auth_jwt.go)
    /// defaults from the Go implementation.  You **must** call [`init`](Self::init) after configuring
    /// the instance.
    pub fn new() -> Self {
        Self {
            realm: "actix jwt".to_string(),
            identity_key: "identity".to_string(),

            signing_algorithm: "HS256".to_string(),
            key: Vec::new(),
            key_func: None,

            timeout: Duration::from_secs(3600), // 1 hour
            timeout_func: None,
            max_refresh: Duration::ZERO,
            time_func: Arc::new(Utc::now),

            authenticator: None,
            authorizer: Arc::new(|_req, _data| true),
            payload_func: None,
            identity_handler: Arc::new(|req| {
                let ext = req.extensions();
                let payload = ext.get::<JwtPayload>()?;
                payload.0.get("identity").cloned()
            }),

            unauthorized: Arc::new(|_req, code, message| {
                HttpResponse::build(
                    actix_web::http::StatusCode::from_u16(code)
                        .unwrap_or(actix_web::http::StatusCode::UNAUTHORIZED),
                )
                .json(serde_json::json!({
                    "code": code,
                    "message": message,
                }))
            }),
            login_response: Arc::new(|_req, token| {
                HttpResponse::Ok().json(Self::generate_token_response_static(token))
            }),
            logout_response: Arc::new(|_req| {
                HttpResponse::Ok().json(serde_json::json!({ "code": 200 }))
            }),
            refresh_response: Arc::new(|_req, token| {
                HttpResponse::Ok().json(Self::generate_token_response_static(token))
            }),
            http_status_message_func: Arc::new(|_req, err| err.to_string()),

            token_lookup: "header:Authorization".to_string(),
            token_head_name: "Bearer".to_string(),
            exp_field: "exp".to_string(),

            priv_key_file: None,
            priv_key_bytes: None,
            pub_key_file: None,
            pub_key_bytes: None,
            private_key_passphrase: None,
            encoding_key: None,
            decoding_key: None,

            send_cookie: false,
            cookie_max_age: Duration::from_secs(3600),
            secure_cookie: false,
            cookie_http_only: false,
            cookie_domain: None,
            cookie_name: "jwt".to_string(),
            cookie_same_site: SameSite::Lax,
            send_authorization: false,

            refresh_token_timeout: Duration::from_secs(30 * 24 * 3600), // 30 days
            refresh_token_cookie_name: "refresh_token".to_string(),
            refresh_token_length: 32,
            refresh_token_store: Arc::new(InMemoryRefreshTokenStore::new()),

            skipper: None,
            before_func: None,
            success_handler: None,
            error_handler: None,
            continue_on_ignored_error: false,
        }
    }

    /// Validates configuration and prepares signing / decoding keys.
    ///
    /// **Must be called before the middleware is used.** Mirrors
    /// [`MiddlewareInit`](https://github.com/LdDl/echo-jwt/blob/master/auth_jwt.go)
    /// from the Go implementation.
    ///
    /// # Errors
    ///
    /// Returns [`JwtError::MissingSecretKey`] when no HMAC key is set (and no
    /// `key_func` or RSA key is configured).
    pub fn init(&mut self) -> Result<(), JwtError> {
        if self.token_lookup.is_empty() {
            self.token_lookup = "header:Authorization".to_string();
        }

        if self.signing_algorithm.is_empty() {
            self.signing_algorithm = "HS256".to_string();
        }

        if self.timeout == Duration::ZERO {
            self.timeout = Duration::from_secs(3600);
        }

        let token_head = self.token_head_name.trim().to_string();
        self.token_head_name = if token_head.is_empty() {
            "Bearer".to_string()
        } else {
            token_head
        };

        if self.realm.is_empty() {
            self.realm = "actix jwt".to_string();
        }

        if self.cookie_max_age == Duration::ZERO {
            self.cookie_max_age = self.timeout;
        }

        if self.cookie_name.is_empty() {
            self.cookie_name = "jwt".to_string();
        }

        if self.refresh_token_cookie_name.is_empty() {
            self.refresh_token_cookie_name = "refresh_token".to_string();
        }

        if self.exp_field.is_empty() {
            self.exp_field = "exp".to_string();
        }

        if self.identity_key.is_empty() {
            self.identity_key = "identity".to_string();
        }

        if self.refresh_token_timeout == Duration::ZERO {
            self.refresh_token_timeout = Duration::from_secs(30 * 24 * 3600);
        }

        if self.refresh_token_length == 0 {
            self.refresh_token_length = 32;
        }

        // Bypass other key settings if KeyFunc is set
        if self.key_func.is_some() {
            return Ok(());
        }

        if self.using_public_key_algo() {
            return self.read_keys();
        }

        if self.key.is_empty() {
            return Err(JwtError::MissingSecretKey);
        }

        self.encoding_key = Some(EncodingKey::from_secret(&self.key));
        self.decoding_key = Some(DecodingKey::from_secret(&self.key));

        Ok(())
    }

    /// Returns `true` when the signing algorithm is RSA-based.
    pub fn using_public_key_algo(&self) -> bool {
        matches!(self.signing_algorithm.as_str(), "RS256" | "RS384" | "RS512")
    }

    /// Parse the `signing_algorithm` string into a `jsonwebtoken::Algorithm`.
    fn algorithm(&self) -> Result<Algorithm, JwtError> {
        match self.signing_algorithm.as_str() {
            "HS256" => Ok(Algorithm::HS256),
            "HS384" => Ok(Algorithm::HS384),
            "HS512" => Ok(Algorithm::HS512),
            "RS256" => Ok(Algorithm::RS256),
            "RS384" => Ok(Algorithm::RS384),
            "RS512" => Ok(Algorithm::RS512),
            _ => Err(JwtError::InvalidSigningAlgorithm),
        }
    }

    fn read_keys(&mut self) -> Result<(), JwtError> {
        self.load_private_key()?;
        self.load_public_key()?;
        Ok(())
    }

    fn load_private_key(&mut self) -> Result<(), JwtError> {
        let key_data = if let Some(ref path) = self.priv_key_file {
            std::fs::read(path).map_err(|e| {
                warn!("Failed to read private key file {}: {}", path, e);
                JwtError::NoPrivKeyFile
            })?
        } else if let Some(ref bytes) = self.priv_key_bytes {
            bytes.clone()
        } else {
            return Err(JwtError::NoPrivKeyFile);
        };

        if let Some(ref passphrase) = self.private_key_passphrase {
            // Encrypted PKCS#8 private key
            let pem_str = std::str::from_utf8(&key_data).map_err(|_| JwtError::InvalidPrivKey)?;
            let doc = pkcs8::EncryptedPrivateKeyInfo::try_from(pem_str.as_bytes())
                .map_err(|_| JwtError::InvalidPrivKey)?;

            let decrypted = doc
                .decrypt(passphrase.as_bytes())
                .map_err(|_| JwtError::InvalidPrivKey)?;

            let der_bytes = decrypted.as_bytes();

            // Re-encode as PEM for jsonwebtoken
            let pem = pem::encode(&pem::Pem::new("PRIVATE KEY", der_bytes.to_vec()));
            self.encoding_key = Some(
                EncodingKey::from_rsa_pem(pem.as_bytes()).map_err(|_| JwtError::InvalidPrivKey)?,
            );
        } else {
            self.encoding_key =
                Some(EncodingKey::from_rsa_pem(&key_data).map_err(|_| JwtError::InvalidPrivKey)?);
        }

        Ok(())
    }

    fn load_public_key(&mut self) -> Result<(), JwtError> {
        let key_data = if let Some(ref path) = self.pub_key_file {
            std::fs::read(path).map_err(|e| {
                warn!("Failed to read public key file {}: {}", path, e);
                JwtError::NoPubKeyFile
            })?
        } else if let Some(ref bytes) = self.pub_key_bytes {
            bytes.clone()
        } else {
            return Err(JwtError::NoPubKeyFile);
        };

        self.decoding_key =
            Some(DecodingKey::from_rsa_pem(&key_data).map_err(|_| JwtError::InvalidPubKey)?);

        Ok(())
    }

    /// Generate a signed JWT access token. Returns `(token_string, expiry)`.
    pub fn generate_access_token(&self, data: &Value) -> Result<(String, DateTime<Utc>), JwtError> {
        let alg = self.algorithm()?;

        let mut claims = serde_json::Map::new();

        // Framework-controlled claims that PayloadFunc must not overwrite.
        let framework_claims: &[&str] = &["exp", "orig_iat"];

        if let Some(ref pf) = self.payload_func {
            for (k, v) in pf(data) {
                if !framework_claims.contains(&k.as_str()) {
                    claims.insert(k, v);
                }
            }
        }

        let now = (self.time_func)();
        let timeout = self
            .timeout_func
            .as_ref()
            .map(|f| f(data))
            .unwrap_or(self.timeout);
        let expire = now
            + chrono::Duration::from_std(timeout)
                .unwrap_or_else(|_| chrono::Duration::seconds(3600));

        claims.insert(
            self.exp_field.clone(),
            Value::Number(expire.timestamp().into()),
        );
        claims.insert(
            "orig_iat".to_string(),
            Value::Number(now.timestamp().into()),
        );

        let header = Header::new(alg);
        let claims_value = Value::Object(claims);

        let encoding_key = self
            .encoding_key
            .as_ref()
            .ok_or(JwtError::MissingSecretKey)?;

        let token_string = jsonwebtoken::encode(&header, &claims_value, encoding_key)
            .map_err(|_| JwtError::FailedTokenCreation)?;

        Ok((token_string, expire))
    }

    /// Generate a cryptographically secure random refresh token (base64url-encoded).
    pub fn generate_refresh_token(&self) -> Result<String, JwtError> {
        use rand::RngCore;
        let mut buf = vec![0u8; self.refresh_token_length];
        rand::thread_rng()
            .try_fill_bytes(&mut buf)
            .map_err(|e| JwtError::Internal(format!("RNG failure: {e}")))?;
        Ok(URL_SAFE.encode(&buf))
    }

    /// Store a refresh token with associated user data.
    async fn store_refresh_token(&self, token: &str, user_data: &Value) -> Result<(), JwtError> {
        let expiry = (self.time_func)()
            + chrono::Duration::from_std(self.refresh_token_timeout)
                .unwrap_or_else(|_| chrono::Duration::days(30));
        self.refresh_token_store
            .set(token, user_data.clone(), expiry)
            .await
    }

    /// Validate a refresh token and return its associated user data.
    async fn validate_refresh_token(&self, token: &str) -> Result<Value, JwtError> {
        self.refresh_token_store
            .get(token)
            .await
            .map_err(|e| match e {
                JwtError::RefreshTokenNotFound => JwtError::InvalidRefreshToken,
                other => other,
            })
    }

    /// Revoke (delete) a refresh token from storage.
    async fn revoke_refresh_token(&self, token: &str) -> Result<(), JwtError> {
        self.refresh_token_store.delete(token).await
    }

    /// Generate a complete token pair (access + refresh) and store the refresh
    /// token. Mirrors [`TokenGenerator`](https://github.com/LdDl/echo-jwt/blob/master/auth_jwt.go)
    /// from the Go implementation.
    pub async fn token_generator(&self, data: &Value) -> Result<Token, JwtError> {
        let (access_token, expire) = self.generate_access_token(data)?;
        let refresh_token = self.generate_refresh_token()?;

        self.store_refresh_token(&refresh_token, data).await?;

        let now = (self.time_func)();
        Ok(Token {
            access_token,
            token_type: "Bearer".to_string(),
            refresh_token: Some(refresh_token),
            expires_at: expire.timestamp(),
            created_at: now.timestamp(),
        })
    }

    /// Generate a new token pair and revoke the old refresh token (rotation).
    pub async fn token_generator_with_revocation(
        &self,
        data: &Value,
        old_refresh_token: &str,
    ) -> Result<Token, JwtError> {
        let token_pair = self.token_generator(data).await?;

        // Revoke old token; ignore "not found" errors
        if let Err(e) = self.revoke_refresh_token(old_refresh_token).await {
            if !matches!(e, JwtError::RefreshTokenNotFound) {
                return Err(e);
            }
        }

        Ok(token_pair)
    }

    /// Parse and validate a JWT from the request according to `token_lookup`.
    pub fn parse_token_from_request(
        &self,
        req: &HttpRequest,
    ) -> Result<TokenData<Value>, JwtError> {
        let token_str = self.extract_token_string(req)?;

        // Store the raw token string in request extensions
        req.extensions_mut()
            .insert(JwtTokenString(token_str.clone()));

        self.parse_token_string(&token_str)
    }

    /// Parse a raw JWT string and return its decoded data.
    pub fn parse_token_string(&self, token: &str) -> Result<TokenData<Value>, JwtError> {
        let alg = self.algorithm()?;

        if let Some(ref kf) = self.key_func {
            // Decode header first to pass to key_func
            let header = jsonwebtoken::decode_header(token)
                .map_err(|e| JwtError::TokenParsing(e.to_string()))?;
            let dk = kf(&header)?;
            let mut validation = Validation::new(alg);
            validation.validate_exp = true;
            validation.validate_aud = false;
            validation.required_spec_claims.clear();
            return jsonwebtoken::decode::<Value>(token, &dk, &validation)
                .map_err(|e| JwtError::TokenParsing(e.to_string()));
        }

        let decoding_key = self
            .decoding_key
            .as_ref()
            .ok_or(JwtError::MissingSecretKey)?;

        let mut validation = Validation::new(alg);
        validation.validate_exp = true;
        validation.validate_aud = false;
        validation.required_spec_claims.clear();

        jsonwebtoken::decode::<Value>(token, decoding_key, &validation)
            .map_err(|e| JwtError::TokenParsing(e.to_string()))
    }

    /// Walk `token_lookup` to find the first available token string in the request.
    fn extract_token_string(&self, req: &HttpRequest) -> Result<String, JwtError> {
        let methods: Vec<&str> = self.token_lookup.split(',').collect();
        let mut last_err: Option<JwtError> = None;

        for method in methods {
            let parts: Vec<&str> = method.trim().splitn(2, ':').collect();
            if parts.len() != 2 {
                continue;
            }
            let source = parts[0].trim();
            let name = parts[1].trim();

            let result = match source {
                "header" => self.jwt_from_header(req, name),
                "query" => self.jwt_from_query(req, name),
                "cookie" => self.jwt_from_cookie(req, name),
                "param" => self.jwt_from_param(req, name),
                "form" => self.jwt_from_form(req, name),
                _ => continue,
            };

            match result {
                Ok(t) if !t.is_empty() => return Ok(t),
                Ok(_) => {}
                Err(e) => {
                    last_err = Some(e);
                }
            }
        }

        Err(last_err.unwrap_or(JwtError::TokenExtraction(
            "no token found in request".to_string(),
        )))
    }

    fn jwt_from_header(&self, req: &HttpRequest, key: &str) -> Result<String, JwtError> {
        let auth_header = req
            .headers()
            .get(key)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if auth_header.is_empty() {
            return Err(JwtError::EmptyAuthHeader);
        }

        let parts: Vec<&str> = auth_header.splitn(2, ' ').collect();
        if parts.len() != 2 || parts[0] != self.token_head_name {
            return Err(JwtError::InvalidAuthHeader);
        }

        Ok(parts[1].to_string())
    }

    fn jwt_from_query(&self, req: &HttpRequest, key: &str) -> Result<String, JwtError> {
        let qs = req.query_string();
        // Simple query-string parser
        for pair in qs.split('&') {
            let mut kv = pair.splitn(2, '=');
            if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                if k == key && !v.is_empty() {
                    return Ok(v.to_string());
                }
            }
        }
        Err(JwtError::EmptyQueryToken)
    }

    fn jwt_from_cookie(&self, req: &HttpRequest, key: &str) -> Result<String, JwtError> {
        req.cookie(key)
            .map(|c| c.value().to_string())
            .filter(|v| !v.is_empty())
            .ok_or(JwtError::EmptyCookieToken)
    }

    fn jwt_from_param(&self, req: &HttpRequest, key: &str) -> Result<String, JwtError> {
        let val = req.match_info().get(key).unwrap_or("");
        if val.is_empty() {
            return Err(JwtError::EmptyParamToken);
        }
        Ok(val.to_string())
    }

    fn jwt_from_form(&self, _req: &HttpRequest, _key: &str) -> Result<String, JwtError> {
        // Form extraction requires body access which is not available from
        // HttpRequest alone. This source is best-effort and typically handled
        // via a pre-parsed body in the handler. Return empty for middleware path.
        Err(JwtError::EmptyParamToken)
    }

    /// Get JWT claims from the request by parsing the token.
    fn get_claims_from_jwt(&self, req: &HttpRequest) -> Result<HashMap<String, Value>, JwtError> {
        let token_data = self.parse_token_from_request(req)?;

        // Token string is already stored in extensions by
        // parse_token_from_request; nothing extra needed for
        // send_authorization here - the middleware layer reads it later.

        let claims_map = match token_data.claims {
            Value::Object(map) => map.into_iter().collect(),
            _ => HashMap::new(),
        };

        Ok(claims_map)
    }

    /// Inner implementation: validate token, set identity, check authorizer.
    fn middleware_impl(&self, req: &HttpRequest) -> Result<(), JwtError> {
        let claims = self
            .get_claims_from_jwt(req)
            .map_err(|e| JwtError::TokenParsing(e.to_string()))?;

        // exp is required (backwards-compat with gin-jwt)
        if !claims.contains_key("exp") {
            return Err(JwtError::TokenExtraction(
                JwtError::MissingExpField.to_string(),
            ));
        }

        req.extensions_mut().insert(JwtPayload(claims));

        let identity = (self.identity_handler)(req);
        if let Some(ref id) = identity {
            req.extensions_mut().insert(JwtIdentity(id.clone()));
        }

        let auth_data = identity.unwrap_or(Value::Null);
        if !(self.authorizer)(req, &auth_data) {
            return Err(JwtError::Forbidden);
        }

        Ok(())
    }

    /// Build a default "unauthorized" `HttpResponse` with `WWW-Authenticate` header.
    fn unauthorized_response(&self, req: &HttpRequest, code: u16, message: &str) -> HttpResponse {
        let mut resp = (self.unauthorized)(req, code, message);
        resp.headers_mut().insert(
            header::WWW_AUTHENTICATE,
            format!("Bearer realm=\"{}\"", self.realm).parse().unwrap(),
        );
        resp
    }

    /// Handle a middleware error when no custom `ErrorHandler` is set.
    fn handle_middleware_error(&self, req: &HttpRequest, err: &JwtError) -> HttpResponse {
        match err {
            JwtError::Forbidden => {
                let msg = (self.http_status_message_func)(req, &JwtError::Forbidden);
                self.unauthorized_response(req, 403, &msg)
            }
            JwtError::TokenParsing(inner) => self.handle_token_error(req, inner),
            JwtError::TokenExtraction(inner) => {
                let msg = inner.clone();
                self.unauthorized_response(req, 400, &msg)
            }
            other => {
                let msg = (self.http_status_message_func)(req, other);
                self.unauthorized_response(req, 401, &msg)
            }
        }
    }

    fn handle_token_error(&self, req: &HttpRequest, detail: &str) -> HttpResponse {
        let lower = detail.to_lowercase();
        if lower.contains("expired") {
            let msg = (self.http_status_message_func)(req, &JwtError::ExpiredToken);
            self.unauthorized_response(req, 401, &msg)
        } else if lower.contains("exp") && lower.contains("invalid") {
            let msg = (self.http_status_message_func)(req, &JwtError::WrongFormatOfExp);
            self.unauthorized_response(req, 400, &msg)
        } else if lower.contains("exp") && lower.contains("required") {
            let msg = (self.http_status_message_func)(req, &JwtError::MissingExpField);
            self.unauthorized_response(req, 400, &msg)
        } else {
            let err = JwtError::TokenParsing(detail.to_string());
            let msg = (self.http_status_message_func)(req, &err);
            self.unauthorized_response(req, 401, &msg)
        }
    }

    /// Set the access-token cookie on the response builder.
    pub fn set_cookie(builder: &mut HttpResponseBuilder, config: &CookieConfig, value: &str) {
        let mut cookie = Cookie::build(config.name.clone(), value.to_string())
            .path("/")
            .max_age(actix_web::cookie::time::Duration::seconds(
                config.max_age.as_secs() as i64,
            ))
            .secure(config.secure)
            .http_only(config.http_only)
            .same_site(config.same_site)
            .finish();

        if let Some(ref domain) = config.domain {
            cookie.set_domain(domain.clone());
        }

        builder.cookie(cookie);
    }

    /// Build a `CookieConfig` for the access token from the middleware settings.
    pub fn access_cookie_config(&self) -> CookieConfig {
        CookieConfig {
            name: self.cookie_name.clone(),
            max_age: self.cookie_max_age,
            secure: self.secure_cookie,
            http_only: self.cookie_http_only,
            domain: self.cookie_domain.clone(),
            same_site: self.cookie_same_site,
        }
    }

    /// Build a `CookieConfig` for the refresh token from the middleware settings.
    pub fn refresh_cookie_config(&self) -> CookieConfig {
        CookieConfig {
            name: self.refresh_token_cookie_name.clone(),
            max_age: self.refresh_token_timeout,
            secure: true,    // always secure for refresh tokens
            http_only: true, // always httpOnly for security
            domain: self.cookie_domain.clone(),
            same_site: self.cookie_same_site,
        }
    }

    /// Append a Set-Cookie header directly to an already-built response's
    /// `HeaderMap`.  Used when the response is created by a callback
    /// (`login_response`, `refresh_response`) and cookies must be added
    /// afterwards.
    fn append_cookie(
        headers: &mut actix_web::http::header::HeaderMap,
        config: &CookieConfig,
        value: &str,
    ) {
        let mut cookie = Cookie::build(config.name.clone(), value.to_string())
            .path("/")
            .max_age(actix_web::cookie::time::Duration::seconds(
                config.max_age.as_secs() as i64,
            ))
            .secure(config.secure)
            .http_only(config.http_only)
            .same_site(config.same_site)
            .finish();

        if let Some(ref domain) = config.domain {
            cookie.set_domain(domain.clone());
        }

        headers.append(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    }

    /// Append a "delete" Set-Cookie header (MaxAge = -1) directly to an
    /// already-built response's `HeaderMap`.
    fn append_delete_cookie(
        headers: &mut actix_web::http::header::HeaderMap,
        config: &CookieConfig,
    ) {
        let mut cookie = Cookie::build(config.name.clone(), "")
            .path("/")
            .max_age(actix_web::cookie::time::Duration::seconds(-1))
            .secure(config.secure)
            .http_only(config.http_only)
            .same_site(config.same_site)
            .finish();

        if let Some(ref domain) = config.domain {
            cookie.set_domain(domain.clone());
        }

        headers.append(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    }

    /// Append a "delete" cookie (MaxAge = -1) to the response builder.
    pub fn delete_cookie(builder: &mut HttpResponseBuilder, config: &CookieConfig) {
        let mut cookie = Cookie::build(config.name.clone(), "")
            .path("/")
            .max_age(actix_web::cookie::time::Duration::seconds(-1))
            .secure(config.secure)
            .http_only(config.http_only)
            .same_site(config.same_site)
            .finish();

        if let Some(ref domain) = config.domain {
            cookie.set_domain(domain.clone());
        }

        builder.cookie(cookie);
    }

    /// Login handler. Expects JSON body parsed by the `authenticator` callback.
    ///
    /// Usage with `web::Data<Arc<ActixJwtMiddleware>>`:
    /// ```ignore
    /// async fn login(
    ///     jwt: web::Data<Arc<ActixJwtMiddleware>>,
    ///     req: HttpRequest,
    ///     body: web::Bytes,
    /// ) -> HttpResponse {
    ///     jwt.login_handler(&req, &body).await
    /// }
    /// ```
    pub async fn login_handler(&self, req: &HttpRequest, body: &[u8]) -> HttpResponse {
        let authenticator = match self.authenticator {
            Some(ref auth) => auth,
            None => {
                let msg = (self.http_status_message_func)(req, &JwtError::MissingAuthenticator);
                return self.unauthorized_response(req, 500, &msg);
            }
        };

        let data = match authenticator(req, body).await {
            Ok(d) => d,
            Err(e) => {
                let msg = (self.http_status_message_func)(req, &e);
                return self.unauthorized_response(req, 401, &msg);
            }
        };

        let token_pair = match self.token_generator(&data).await {
            Ok(t) => t,
            Err(_) => {
                let msg = (self.http_status_message_func)(req, &JwtError::FailedTokenCreation);
                return self.unauthorized_response(req, 500, &msg);
            }
        };

        let mut resp = (self.login_response)(req, &token_pair);

        if self.send_cookie {
            Self::append_cookie(
                resp.headers_mut(),
                &self.access_cookie_config(),
                &token_pair.access_token,
            );
            if let Some(ref rt) = token_pair.refresh_token {
                Self::append_cookie(resp.headers_mut(), &self.refresh_cookie_config(), rt);
            }
        }

        resp
    }

    /// Extract a refresh token from the request (cookie, form, or JSON body).
    pub fn extract_refresh_token(&self, req: &HttpRequest, body: &[u8]) -> Option<String> {
        // 1. Try cookie
        if let Some(cookie) = req.cookie(&self.refresh_token_cookie_name) {
            let val = cookie.value().to_string();
            if !val.is_empty() {
                return Some(val);
            }
        }

        // 2. Try JSON or form body
        let content_type = req
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("");

        if content_type.contains("application/x-www-form-urlencoded")
            || content_type.contains("multipart/form-data")
        {
            // Attempt to parse as form-urlencoded
            let body_str = std::str::from_utf8(body).unwrap_or("");
            for pair in body_str.split('&') {
                let mut kv = pair.splitn(2, '=');
                if let (Some(k), Some(v)) = (kv.next(), kv.next()) {
                    if k == "refresh_token" && !v.is_empty() {
                        return Some(v.to_string());
                    }
                }
            }
        } else if content_type.contains("application/json") {
            #[derive(serde::Deserialize)]
            struct RefreshBody {
                refresh_token: Option<String>,
            }
            if let Ok(parsed) = serde_json::from_slice::<RefreshBody>(body) {
                if let Some(rt) = parsed.refresh_token {
                    if !rt.is_empty() {
                        return Some(rt);
                    }
                }
            }
        }

        None
    }

    /// Logout handler. Revokes the refresh token and clears cookies.
    pub async fn logout_handler(&self, req: &HttpRequest, body: &[u8]) -> HttpResponse {
        // Try to extract claims for the LogoutResponse callback
        if let Ok(claims) = self.get_claims_from_jwt(req) {
            req.extensions_mut().insert(JwtPayload(claims));
            let identity = (self.identity_handler)(req);
            if let Some(ref id) = identity {
                req.extensions_mut().insert(JwtIdentity(id.clone()));
            }
        }

        // Revoke refresh token
        if let Some(ref rt) = self.extract_refresh_token(req, body) {
            if let Err(e) = self.revoke_refresh_token(rt).await {
                warn!("Failed to revoke refresh token on logout: {}", e);
            }
        }

        let mut resp = (self.logout_response)(req);

        if self.send_cookie {
            Self::append_delete_cookie(resp.headers_mut(), &self.access_cookie_config());
            Self::append_delete_cookie(resp.headers_mut(), &self.refresh_cookie_config());
        }

        resp
    }

    /// Refresh handler. Validates old refresh token, generates new token pair,
    /// revokes old refresh token (rotation).
    pub async fn refresh_handler(&self, req: &HttpRequest, body: &[u8]) -> HttpResponse {
        let refresh_token = match self.extract_refresh_token(req, body) {
            Some(rt) => rt,
            None => {
                let msg = (self.http_status_message_func)(req, &JwtError::MissingRefreshToken);
                return self.unauthorized_response(req, 400, &msg);
            }
        };

        let user_data = match self.validate_refresh_token(&refresh_token).await {
            Ok(d) => d,
            Err(e) => {
                let msg = (self.http_status_message_func)(req, &e);
                return self.unauthorized_response(req, 401, &msg);
            }
        };

        let token_pair = match self
            .token_generator_with_revocation(&user_data, &refresh_token)
            .await
        {
            Ok(t) => t,
            Err(e) => {
                let msg = (self.http_status_message_func)(req, &e);
                return self.unauthorized_response(req, 500, &msg);
            }
        };

        let mut resp = (self.refresh_response)(req, &token_pair);

        if self.send_cookie {
            Self::append_cookie(
                resp.headers_mut(),
                &self.access_cookie_config(),
                &token_pair.access_token,
            );
            if let Some(ref rt) = token_pair.refresh_token {
                Self::append_cookie(resp.headers_mut(), &self.refresh_cookie_config(), rt);
            }
        }

        resp
    }

    /// Builds an [RFC 6749](https://datatracker.ietf.org/doc/html/rfc6749#section-5.1)
    /// token response map containing `access_token`, `token_type`,
    /// `expires_in` and (optionally) `refresh_token`.
    pub fn generate_token_response(token: &Token) -> serde_json::Map<String, Value> {
        let mut map = serde_json::Map::new();
        map.insert(
            "access_token".into(),
            Value::String(token.access_token.clone()),
        );
        map.insert("token_type".into(), Value::String(token.token_type.clone()));
        map.insert(
            "expires_in".into(),
            Value::Number(token.expires_in().into()),
        );

        if let Some(ref rt) = token.refresh_token {
            map.insert("refresh_token".into(), Value::String(rt.clone()));
        }

        map
    }

    /// Same as `generate_token_response` but returns a `Value` for direct
    /// serialisation (used by default response callbacks).
    fn generate_token_response_static(token: &Token) -> Value {
        let map = Self::generate_token_response(token);
        Value::Object(map)
    }

    /// Creates an actix-web `Transform` that can be passed to `.wrap()`.
    ///
    /// The middleware must be wrapped in `Arc` before calling this method.
    pub fn middleware(self: &Arc<Self>) -> JwtAuth {
        JwtAuth {
            inner: self.clone(),
        }
    }
}

impl Default for ActixJwtMiddleware {
    fn default() -> Self {
        Self::new()
    }
}

/// Describes cookie parameters for either access or refresh tokens.
///
/// Constructed internally by
/// [`ActixJwtMiddleware::access_cookie_config`] /
/// [`ActixJwtMiddleware::refresh_cookie_config`].
pub struct CookieConfig {
    /// Cookie name.
    pub name: String,
    /// `Max-Age` value.
    pub max_age: Duration,
    /// `Secure` flag.
    pub secure: bool,
    /// `HttpOnly` flag.
    pub http_only: bool,
    /// Optional `Domain` attribute.
    pub domain: Option<String>,
    /// `SameSite` attribute.
    pub same_site: SameSite,
}

/// Extracts JWT claims from [`HttpRequest`] extensions.
///
/// Returns an empty map if the middleware has not yet validated a token for
/// this request.
///
/// # Examples
///
/// ```rust,no_run
/// use actix_web::{HttpRequest, HttpResponse};
/// use actix_jwt::extract_claims;
///
/// async fn handler(req: HttpRequest) -> HttpResponse {
///     let claims = extract_claims(&req);
///     HttpResponse::Ok().json(claims)
/// }
/// ```
pub fn extract_claims(req: &HttpRequest) -> HashMap<String, Value> {
    req.extensions()
        .get::<JwtPayload>()
        .map(|p| p.0.clone())
        .unwrap_or_default()
}

/// Extracts the raw JWT token string from [`HttpRequest`] extensions.
///
/// Returns `None` if the middleware has not processed the request yet.
pub fn get_token(req: &HttpRequest) -> Option<String> {
    req.extensions()
        .get::<JwtTokenString>()
        .map(|t| t.0.clone())
}

/// Extracts the identity value from [`HttpRequest`] extensions.
///
/// Returns `None` if the middleware has not processed the request yet or no
/// identity was resolved.
pub fn get_identity(req: &HttpRequest) -> Option<Value> {
    req.extensions().get::<JwtIdentity>().map(|i| i.0.clone())
}

/// [`Transform`] factory produced by
/// [`ActixJwtMiddleware::middleware`].
///
/// You do not need to construct this directly - use
/// `jwt.middleware()` instead.
pub struct JwtAuth {
    inner: Arc<ActixJwtMiddleware>,
}

impl<S, B> Transform<S, ServiceRequest> for JwtAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = actix_web::Error;
    type Transform = JwtAuthMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(JwtAuthMiddleware {
            service: Arc::new(service),
            inner: self.inner.clone(),
        }))
    }
}

/// Per-request middleware service created by [`JwtAuth`].
pub struct JwtAuthMiddleware<S> {
    service: Arc<S>,
    inner: Arc<ActixJwtMiddleware>,
}

impl<S, B> Service<ServiceRequest> for JwtAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    B: 'static,
{
    type Response = ServiceResponse<actix_web::body::EitherBody<B>>;
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;

    fn poll_ready(
        &self,
        ctx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mw = self.inner.clone();
        let service = self.service.clone();

        Box::pin(async move {
            // 1. Skipper
            if let Some(ref skipper) = mw.skipper {
                if skipper(&req) {
                    let res = service.call(req).await?;
                    return Ok(res.map_into_left_body());
                }
            }

            // 2. BeforeFunc
            if let Some(ref bf) = mw.before_func {
                bf(&req);
            }

            // 3. Run middleware logic
            //
            // IMPORTANT: We must NOT hold an `HttpRequest` clone while calling
            // `service.call(req)` because actix-web's `Rc::get_mut` on the
            // inner request data will panic if the refcount > 1. Instead, we
            // borrow `req.request()` only within a limited scope.
            let mw_result = mw.middleware_impl(req.request());

            if let Err(err) = mw_result {
                // ErrorHandler
                if let Some(ref eh) = mw.error_handler {
                    let maybe_err = eh(req.request(), err);
                    if maybe_err.is_none() && mw.continue_on_ignored_error {
                        // Extensions are already on the shared HttpRequest
                        // inner, so the ServiceRequest can see them without
                        // an explicit copy.
                        let res = service.call(req).await?;
                        return Ok(res.map_into_left_body());
                    }
                    if let Some(e) = maybe_err {
                        let resp = mw.handle_middleware_error(req.request(), &e);
                        return Ok(req.into_response(resp).map_into_right_body());
                    }
                    // ErrorHandler returned None but ContinueOnIgnoredError is false
                    return Ok(req
                        .into_response(HttpResponse::Ok().finish())
                        .map_into_right_body());
                }

                // No ErrorHandler - default unauthorized
                let resp = mw.handle_middleware_error(req.request(), &err);
                return Ok(req.into_response(resp).map_into_right_body());
            }

            // 4. SuccessHandler
            if let Some(ref sh) = mw.success_handler {
                if let Err(e) = sh(req.request()) {
                    let resp = mw.handle_middleware_error(req.request(), &e);
                    return Ok(req.into_response(resp).map_into_right_body());
                }
            }

            // 5. Send authorization header if configured
            let send_auth = if mw.send_authorization {
                let ext = req.extensions();
                ext.get::<JwtTokenString>()
                    .map(|t| format!("{} {}", mw.token_head_name, t.0))
            } else {
                None
            };

            // 6. Call inner service
            let mut res = service.call(req).await?;

            if let Some(val) = send_auth {
                res.headers_mut()
                    .insert(header::AUTHORIZATION, val.parse().unwrap());
            }

            Ok(res.map_into_left_body())
        })
    }
}
