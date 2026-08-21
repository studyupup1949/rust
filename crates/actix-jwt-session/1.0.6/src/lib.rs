//! All in one creating session and session validation library for actix.
//!
//! It's designed to extract session using middleware and validate endpoint
//! simply by using actix-web extractors. Currently you can extract tokens from
//! Header or Cookie. It's possible to implement Path, Query or Body using
//! `[ServiceRequest::extract]` but you must have struct to which values will be
//! extracted so it's easy to do if you have your own fields.
//!
//! Example:
//!
//! ```
//! use serde::Deserialize;
//!
//! #[derive(Deserialize)]
//! struct MyJsonBody {
//!     jwt: Option<String>,
//!     refresh: Option<String>,
//! }
//! ```
//!
//! To start with this library you need to create your own `AppClaims` structure
//! and implement `actix_jwt_session::Claims` trait for it.
//!
//! ```
//! use serde::{Serialize, Deserialize};
//!
//! #[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
//! #[serde(rename_all = "snake_case")]
//! pub enum Audience {
//!     Web,
//! }
//!
//! #[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
//! #[serde(rename_all = "snake_case")]
//! pub struct Claims {
//!     #[serde(rename = "exp")]
//!     pub expiration_time: u64,
//!     #[serde(rename = "iat")]
//!     pub issues_at: usize,
//!     /// Account login
//!     #[serde(rename = "sub")]
//!     pub subject: String,
//!     #[serde(rename = "aud")]
//!     pub audience: Audience,
//!     #[serde(rename = "jti")]
//!     pub jwt_id: uuid::Uuid,
//!     #[serde(rename = "aci")]
//!     pub account_id: i32,
//!     #[serde(rename = "nbf")]
//!     pub not_before: u64,
//! }
//!
//! impl actix_jwt_session::Claims for Claims {
//!     fn jti(&self) -> uuid::Uuid {
//!         self.jwt_id
//!     }
//!
//!     fn subject(&self) -> &str {
//!         &self.subject
//!     }
//! }
//!
//! impl Claims {
//!     pub fn account_id(&self) -> i32 {
//!         self.account_id
//!     }
//! }
//! ```
//!
//! Then you must create middleware factory with session storage. Currently
//! there's adapter only for redis so we will goes with it in this example.
//!
//! * First create connection pool to redis using `redis_async_pool`.
//! * Next generate or load create jwt signing keys. They are required for
//!   creating JWT from claims.
//! * Finally pass keys and algorithm to builder, pass pool and add some
//!   extractors
//!
//! ```
//! use std::sync::Arc;
//! use actix_jwt_session::*;
//!
//! # async fn create<AppClaims: actix_jwt_session::Claims>() {
//!     // create redis connection
//!     let redis = deadpool_redis::Config::from_url("redis://localhost:6379")
//!         .create_pool(Some(deadpool_redis::Runtime::Tokio1)).unwrap();
//!  
//!     // create new [SessionStorage] and [SessionMiddlewareFactory]
//!     let (storage, factory) = SessionMiddlewareFactory::<AppClaims>::build_ed_dsa()
//!     // pass redis connection
//!     .with_redis_pool(redis.clone())
//!     .with_extractors(
//!         Extractors::default()
//!             // Check if header "Authorization" exists and contains Bearer with encoded JWT
//!             .with_jwt_header("Authorization")
//!             // Check if cookie "jwt" exists and contains encoded JWT
//!             .with_jwt_cookie("acx-a")
//!             .with_refresh_header("ACX-Refresh")
//!             // Check if cookie "jwt" exists and contains encoded JWT
//!             .with_refresh_cookie("acx-r")
//!     )
//!     .finish();
//! # }
//! ```
//!
//! As you can see we have there [SessionMiddlewareBuilder::with_refresh_cookie]
//! and [SessionMiddlewareBuilder::with_refresh_header]. Library uses
//! internal structure [RefreshToken] which is created and managed internally
//! without any additional user work.
//!
//! This will be used to extend JWT lifetime. This lifetime comes from 2
//! structures which describe time to live. [JwtTtl] describes how long access
//! token should be valid, [RefreshToken] describes how long refresh token is
//! valid. [SessionStorage] allows to extend livetime of both with single call
//! of [SessionStorage::refresh] and it will change time of creating tokens to
//! current time.
//!
//! ```
//! use actix_jwt_session::{JwtTtl, RefreshTtl, Duration};
//!
//! let jwt_ttl = JwtTtl(Duration::days(14));
//! let refresh_ttl = RefreshTtl(Duration::days(3 * 31));
//! ```
//!
//! Now you just need to add those structures to [actix_web::App] using
//! `.app_data` and `.wrap` and you are ready to go. Bellow you have full
//! example of usage.
//!
//! Examples:
//!
//! ```no_run
//! use std::sync::Arc;
//! use actix_jwt_session::*;
//! use actix_web::{get, post};
//! use actix_web::web::{Data, Json};
//! use actix_web::{HttpResponse, App, HttpServer};
//! use jsonwebtoken::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[tokio::main]
//! async fn main() {
//!     // create redis connection
//!     let redis = deadpool_redis::Config::from_url("redis://localhost:6379")
//!         .create_pool(Some(deadpool_redis::Runtime::Tokio1)).unwrap();
//!  
//!     let jwt_ttl = JwtTtl(Duration::days(14));
//!     let refresh_ttl = RefreshTtl(Duration::days(3 * 31));
//!  
//!     HttpServer::new(move || {
//!         App::new()
//!             .app_data(Data::new( jwt_ttl ))
//!             .app_data(Data::new( refresh_ttl ))
//!             .use_jwt::<AppClaims>(
//!                 Extractors::default()
//!                     // Check if header "Authorization" exists and contains Bearer with encoded JWT
//!                     .with_jwt_header(JWT_HEADER_NAME)
//!                     // Check if cookie JWT exists and contains encoded JWT
//!                     .with_jwt_cookie(JWT_COOKIE_NAME)
//!                     .with_refresh_header(REFRESH_HEADER_NAME)
//!                     // Check if cookie JWT exists and contains encoded JWT
//!                     .with_refresh_cookie(REFRESH_COOKIE_NAME),
//!                 Some(redis.clone())
//!             )
//!             .app_data(Data::new(redis.clone()))
//!             .service(must_be_signed_in)
//!             .service(may_be_signed_in)
//!             .service(register)
//!             .service(sign_in)
//!             .service(sign_out)
//!             .service(refresh_session)
//!             .service(session_info)
//!             .service(root)
//!     })
//!     .bind(("0.0.0.0", 8080)).unwrap()
//!     .run()
//!     .await.unwrap();
//! }
//!
//! #[derive(Clone, PartialEq, Serialize, Deserialize)]
//! pub struct SessionData {
//!     id: uuid::Uuid,
//!     subject: String,
//! }
//!
//! #[get("/authorized")]
//! async fn must_be_signed_in(session: Authenticated<AppClaims>) -> HttpResponse {
//!     use crate::actix_jwt_session::Claims;
//!     let jit = session.jti();
//!     HttpResponse::Ok().finish()
//! }
//!
//! #[get("/maybe-authorized")]
//! async fn may_be_signed_in(session: MaybeAuthenticated<AppClaims>) -> HttpResponse {
//!     if let Some(session) = session.into_option() {
//!     }
//!     HttpResponse::Ok().finish()
//! }
//!
//! #[derive(Deserialize)]
//! struct SignUpPayload {
//!     login: String,
//!     password: String,
//!     password_confirmation: String,
//! }
//!
//! #[post("/session/sign-up")]
//! async fn register(payload: Json<SignUpPayload>) -> Result<HttpResponse, actix_web::Error> {
//!     let payload = payload.into_inner();
//!     
//!     // Validate payload
//!     
//!     // Save model and return HttpResponse
//!     let model = AccountModel {
//!         id: -1,
//!         login: payload.login,
//!         // Encrypt password before saving to database
//!         pass_hash: Hashing::encrypt(&payload.password).unwrap(),
//!     };
//!     // Save model
//!
//!     # todo!()
//! }
//!
//! #[derive(Deserialize)]
//! struct SignInPayload {
//!     login: String,
//!     password: String,
//! }
//!
//! #[post("/session/sign-in")]
//! async fn sign_in(
//!     store: Data<SessionStorage>,
//!     payload: Json<SignInPayload>,
//!     jwt_ttl: Data<JwtTtl>,
//!     refresh_ttl: Data<RefreshTtl>,
//! ) -> Result<HttpResponse, actix_web::Error> {
//!     let payload = payload.into_inner();
//!     let store = store.into_inner();
//!     let account: AccountModel = {
//!         /* load account using login */
//! #         todo!()
//!     };
//!     if let Err(e) = Hashing::verify(account.pass_hash.as_str(), payload.password.as_str()) {
//!         return Ok(HttpResponse::Unauthorized().finish());
//!     }
//!     let claims = AppClaims {
//!          issues_at: OffsetDateTime::now_utc().unix_timestamp() as usize,
//!          subject: account.login.clone(),
//!          expiration_time: jwt_ttl.0.as_seconds_f64() as u64,
//!          audience: Audience::Web,
//!          jwt_id: uuid::Uuid::new_v4(),
//!          account_id: account.id,
//!          not_before: 0,
//!     };
//!     let pair = store
//!         .clone()
//!         .store(claims, *jwt_ttl.into_inner(), *refresh_ttl.into_inner())
//!         .await
//!         .unwrap();
//!     Ok(HttpResponse::Ok()
//!         .append_header((JWT_HEADER_NAME, pair.jwt.encode().unwrap()))
//!         .append_header((REFRESH_HEADER_NAME, pair.refresh.encode().unwrap()))
//!         .finish())
//! }
//!
//! #[post("/session/sign-out")]
//! async fn sign_out(store: Data<SessionStorage>, auth: Authenticated<AppClaims>) -> HttpResponse {
//!     let store = store.into_inner();
//!     store.erase::<AppClaims>(auth.jwt_id).await.unwrap();
//!     HttpResponse::Ok()
//!         .append_header((JWT_HEADER_NAME, ""))
//!         .append_header((REFRESH_HEADER_NAME, ""))
//!         .cookie(
//!             actix_web::cookie::Cookie::build(JWT_COOKIE_NAME, "")
//!                 .expires(OffsetDateTime::now_utc())
//!                 .finish(),
//!         )
//!         .cookie(
//!             actix_web::cookie::Cookie::build(REFRESH_COOKIE_NAME, "")
//!                 .expires(OffsetDateTime::now_utc())
//!                 .finish(),
//!         )
//!         .finish()
//! }
//!
//! #[get("/session/info")]
//! async fn session_info(auth: Authenticated<AppClaims>) -> HttpResponse {
//!     HttpResponse::Ok().json(&*auth)
//! }
//!
//! #[get("/session/refresh")]
//! async fn refresh_session(
//!     refresh_token: Authenticated<RefreshToken>,
//!     storage: Data<SessionStorage>,
//! ) -> HttpResponse {
//!     let s = storage.into_inner();
//!     let pair = match s.refresh::<AppClaims>(refresh_token.access_jti()).await {
//!         Err(e) => {
//!             tracing::warn!("Failed to refresh token: {e}");
//!             return HttpResponse::BadRequest().finish();
//!         }
//!         Ok(pair) => pair,
//!     };
//!
//!     let encrypted_jwt = match pair.jwt.encode() {
//!         Ok(text) => text,
//!         Err(e) => {
//!             tracing::warn!("Failed to encode claims: {e}");
//!             return HttpResponse::InternalServerError().finish();
//!         }
//!     };
//!     let encrypted_refresh = match pair.refresh.encode() {
//!         Err(e) => {
//!             tracing::warn!("Failed to encode claims: {e}");
//!             return HttpResponse::InternalServerError().finish();
//!         }
//!         Ok(text) => text,
//!     };
//!     HttpResponse::Ok()
//!         .append_header((
//!             actix_jwt_session::JWT_HEADER_NAME,
//!             format!("Bearer {encrypted_jwt}").as_str(),
//!         ))
//!         .append_header((
//!             actix_jwt_session::REFRESH_HEADER_NAME,
//!             format!("Bearer {}", encrypted_refresh).as_str(),
//!         ))
//!         .append_header((
//!             "ACX-JWT-TTL",
//!             (pair.refresh.issues_at + pair.refresh.refresh_ttl.0).to_string(),
//!         ))
//!         .finish()
//! }
//!
//! #[get("/")]
//! async fn root() -> HttpResponse {
//!     HttpResponse::Ok().finish()
//! }
//!
//! #[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
//! #[serde(rename_all = "snake_case")]
//! pub enum Audience {
//!     Web,
//! }
//!
//! #[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
//! #[serde(rename_all = "snake_case")]
//! pub struct AppClaims {
//!     #[serde(rename = "exp")]
//!     pub expiration_time: u64,
//!     #[serde(rename = "iat")]
//!     pub issues_at: usize,
//!     /// Account login
//!     #[serde(rename = "sub")]
//!     pub subject: String,
//!     #[serde(rename = "aud")]
//!     pub audience: Audience,
//!     #[serde(rename = "jti")]
//!     pub jwt_id: uuid::Uuid,
//!     #[serde(rename = "aci")]
//!     pub account_id: i32,
//!     #[serde(rename = "nbf")]
//!     pub not_before: u64,
//! }
//!
//! impl actix_jwt_session::Claims for AppClaims {
//!     fn jti(&self) -> uuid::Uuid {
//!         self.jwt_id
//!     }
//!
//!     fn subject(&self) -> &str {
//!         &self.subject
//!     }
//! }
//!
//! impl AppClaims {
//!     pub fn account_id(&self) -> i32 {
//!         self.account_id
//!     }
//! }
//!
//! struct AccountModel {
//!     id: i32,
//!     login: String,
//!     pass_hash: String,
//! }
//! ```

use std::borrow::Cow;
use std::marker::PhantomData;
use std::sync::Arc;

pub use actix_web::cookie::time::{Duration, OffsetDateTime};
use actix_web::dev::ServiceRequest;
use actix_web::{FromRequest, HttpMessage, HttpResponse};
use async_trait::async_trait;
use derive_more::{Constructor, Deref};
pub use jsonwebtoken::Algorithm;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Validation};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
pub use uuid::Uuid;

/// This is maximum duration of json web token after which token will be invalid
/// and depends on implementation removed.
///
/// This value should never be lower than 1 second since some implementations
/// don't accept values lower than 1s.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Deref, Constructor)]
#[serde(transparent)]
pub struct JwtTtl(pub Duration);

/// This is maximum duration of refresh token after which token will be invalid
/// and depends on implementation removed
///
/// This value should never be lower than 1 second since some implementations
/// don't accept values lower than 1s.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Deref, Constructor)]
#[serde(transparent)]
pub struct RefreshTtl(pub Duration);

/// Default json web token header name
///
/// Examples:
///
/// ```
/// use actix_web::{get, HttpResponse, cookie::Cookie};
/// use actix_jwt_session::*;
///
/// async fn create_response<C: Claims>(pair: Pair<C>) -> HttpResponse {
///     let jwt_text = pair.jwt.encode().unwrap();
///     let refresh_text = pair.refresh.encode().unwrap();
///     HttpResponse::Ok()
///         .append_header((JWT_HEADER_NAME, jwt_text.as_str()))
///         .append_header((REFRESH_HEADER_NAME, refresh_text.as_str()))
///         .cookie(
///             actix_web::cookie::Cookie::build(JWT_COOKIE_NAME, jwt_text.as_str())
///                 .finish()
///         )
///         .cookie(
///             actix_web::cookie::Cookie::build(REFRESH_COOKIE_NAME, refresh_text.as_str())
///                 .finish()
///         )
///         .finish()
/// }
/// ```
pub static JWT_HEADER_NAME: &str = "Authorization";

/// Default refresh token header name
///
/// Examples:
///
/// ```
/// use actix_web::{get, HttpResponse, cookie::Cookie};
/// use actix_jwt_session::*;
///
/// async fn create_response<C: Claims>(pair: Pair<C>) -> HttpResponse {
///     let jwt_text = pair.jwt.encode().unwrap();
///     let refresh_text = pair.refresh.encode().unwrap();
///     HttpResponse::Ok()
///         .append_header((JWT_HEADER_NAME, jwt_text.as_str()))
///         .append_header((REFRESH_HEADER_NAME, refresh_text.as_str()))
///         .cookie(
///             actix_web::cookie::Cookie::build(JWT_COOKIE_NAME, jwt_text.as_str())
///                 .finish()
///         )
///         .cookie(
///             actix_web::cookie::Cookie::build(REFRESH_COOKIE_NAME, refresh_text.as_str())
///                 .finish()
///         )
///         .finish()
/// }
/// ```
pub static REFRESH_HEADER_NAME: &str = "ACX-Refresh";

/// Default json web token cookie name
///
/// Examples:
///
/// ```
/// use actix_web::{get, HttpResponse, cookie::Cookie};
/// use actix_jwt_session::*;
///
/// async fn create_response<C: Claims>(pair: Pair<C>) -> HttpResponse {
///     let jwt_text = pair.jwt.encode().unwrap();
///     let refresh_text = pair.refresh.encode().unwrap();
///     HttpResponse::Ok()
///         .append_header((JWT_HEADER_NAME, jwt_text.as_str()))
///         .append_header((REFRESH_HEADER_NAME, refresh_text.as_str()))
///         .cookie(
///             actix_web::cookie::Cookie::build(JWT_COOKIE_NAME, jwt_text.as_str())
///                 .finish()
///         )
///         .cookie(
///             actix_web::cookie::Cookie::build(REFRESH_COOKIE_NAME, refresh_text.as_str())
///                 .finish()
///         )
///         .finish()
/// }
/// ```
pub static JWT_COOKIE_NAME: &str = "ACX-Auth";

/// Default refresh token cookie name
///
/// Examples:
///
/// ```
/// use actix_web::{get, HttpResponse, cookie::Cookie};
/// use actix_jwt_session::*;
///
/// async fn create_response<C: Claims>(pair: Pair<C>) -> HttpResponse {
///     let jwt_text = pair.jwt.encode().unwrap();
///     let refresh_text = pair.refresh.encode().unwrap();
///     HttpResponse::Ok()
///         .append_header((JWT_HEADER_NAME, jwt_text.as_str()))
///         .append_header((REFRESH_HEADER_NAME, refresh_text.as_str()))
///         .cookie(
///             actix_web::cookie::Cookie::build(JWT_COOKIE_NAME, jwt_text.as_str())
///                 .finish()
///         )
///         .cookie(
///             actix_web::cookie::Cookie::build(REFRESH_COOKIE_NAME, refresh_text.as_str())
///                 .finish()
///         )
///         .finish()
/// }
/// ```
pub static REFRESH_COOKIE_NAME: &str = "ACX-Refresh";

/// Serializable and storable struct which represent JWT claims
///
/// * It must have JWT ID as [uuid::Uuid]
/// * It must have subject as a String
pub trait Claims:
    PartialEq + DeserializeOwned + Serialize + Clone + Send + Sync + std::fmt::Debug + 'static
{
    /// Unique token identifier
    fn jti(&self) -> uuid::Uuid;

    /// Login, email or other identifier
    fn subject(&self) -> &str;
}

/// Internal claims which allows to extend tokens pair livetime
///
/// After encoding it can be used as HTTP token send to endpoint, decoded and
/// extend pair livetime. It's always created while calling
/// [SessionStorage::store]. If there's any extractor for refresh you can use
/// this structure as guard for an endpoint.
///
/// Example:
///
/// ```
/// use actix_web::{get, HttpResponse};
/// use actix_web::web::Data;
/// use actix_jwt_session::*;
///
/// #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// pub struct AppClaims { id: uuid::Uuid, sub: String }
/// impl actix_jwt_session::Claims for AppClaims {
///     fn jti(&self) -> uuid::Uuid { self.id }
///     fn subject(&self) -> &str { &self.sub }
/// }
///
/// #[get("/session/refresh")]
/// async fn refresh_session(
///     auth: Authenticated<RefreshToken>,
///     storage: Data<SessionStorage>,
/// ) -> HttpResponse {
///     let storage = storage.into_inner();
///     storage.refresh::<AppClaims>(auth.refresh_jti).await.unwrap();
///     HttpResponse::Ok().json(&*auth)
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefreshToken {
    /// date and time when token was created
    #[serde(rename = "iat")]
    pub issues_at: OffsetDateTime,

    /// related JWT unique identifier
    #[serde(rename = "sub")]
    access_jti: String,

    /// JWT lifetime
    pub access_ttl: JwtTtl,

    /// this token unique identifier
    pub refresh_jti: uuid::Uuid,

    /// this token lifetime
    pub refresh_ttl: RefreshTtl,

    // REQUIRED
    /// this token lifetime as integer
    /// (this field is required by standard)
    #[serde(rename = "exp")]
    pub expiration_time: u64,

    /// time before which token is not validate
    /// (this field is required by standard and always set `0`)
    #[serde(rename = "nbf")]
    pub not_before: u64,

    /// target audience
    /// (this field is required by standard)
    #[serde(rename = "aud")]

    /// who created this token
    /// (this field is required by standard)
    pub audience: String,
    #[serde(rename = "iss")]
    pub issuer: String,
}

impl PartialEq for RefreshToken {
    fn eq(&self, o: &Self) -> bool {
        self.access_jti == o.access_jti
            && self.refresh_jti == o.refresh_jti
            && self.refresh_ttl == o.refresh_ttl
            && self.expiration_time == o.expiration_time
            && self.not_before == o.not_before
            && self.audience == o.audience
            && self.issuer == o.issuer
    }
}

impl RefreshToken {
    pub fn is_access_valid(&self) -> bool {
        self.issues_at + self.access_ttl.0 >= OffsetDateTime::now_utc()
    }

    pub fn is_refresh_valid(&self) -> bool {
        self.issues_at + self.refresh_ttl.0 >= OffsetDateTime::now_utc()
    }

    pub fn access_jti(&self) -> uuid::Uuid {
        Uuid::parse_str(&self.access_jti).unwrap()
    }
}

impl Claims for RefreshToken {
    fn jti(&self) -> uuid::Uuid {
        self.refresh_jti
    }
    fn subject(&self) -> &str {
        "refresh-token"
    }
}

/// JSON Web Token and internally created refresh token.
///
/// Both should be encoded using [Authenticated::encode] and added to response
/// as cookie, header or in body.
pub struct Pair<ClaimsType: Claims> {
    /// Access token in form of JWT decrypted token
    pub jwt: Authenticated<ClaimsType>,
    /// Refresh token in form of JWT decrypted token
    pub refresh: Authenticated<RefreshToken>,
}

/// Session related errors
#[derive(Debug, thiserror::Error, PartialEq, Clone, Copy)]
pub enum Error {
    #[error("Failed to obtain redis connection")]
    RedisConn,
    #[error("Record not found")]
    NotFound,
    #[error("Record malformed")]
    RecordMalformed,
    #[error("Invalid session")]
    InvalidSession,
    #[error("Claims can't be loaded")]
    LoadError,
    #[error("Storage claims and given claims are different")]
    DontMatch,
    #[error("Given token in invalid. Can't decode claims")]
    CantDecode,
    #[error("No http authentication header")]
    NoAuthHeader,
    #[error("Failed to serialize claims")]
    SerializeFailed,
    #[error("Unable to write claims to storage")]
    WriteFailed,
    #[error("Access token expired")]
    JWTExpired,
}

impl actix_web::ResponseError for Error {
    fn status_code(&self) -> actix_web::http::StatusCode {
        match self {
            Self::RedisConn => actix_web::http::StatusCode::INTERNAL_SERVER_ERROR,
            _ => actix_web::http::StatusCode::UNAUTHORIZED,
        }
    }

    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        HttpResponse::build(self.status_code()).body("")
    }
}

/// Extractable user session which requires presence of JWT in request.
/// If there's no JWT endpoint which requires this structure will automatically
/// returns `401`.
///
/// Examples:
///
/// ```
/// use actix_web::get;
/// use actix_web::HttpResponse;
/// use actix_jwt_session::Authenticated;
///
/// #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// pub struct AppClaims { id: uuid::Uuid, sub: String }
/// impl actix_jwt_session::Claims for AppClaims {
///     fn jti(&self) -> uuid::Uuid { self.id }
///     fn subject(&self) -> &str { &self.sub }
/// }
///
/// // If there's no JWT in request server will automatically returns 401
/// #[get("/session")]
/// async fn read_session(session: Authenticated<AppClaims>) -> HttpResponse {
///     let encoded = session.encode().unwrap(); // JWT as encrypted string
///     HttpResponse::Ok().finish()
/// }
/// ```
#[derive(Clone)]
pub struct Authenticated<T> {
    pub claims: Arc<T>,
    pub jwt_encoding_key: Arc<EncodingKey>,
    pub algorithm: Algorithm,
}

impl<T> std::ops::Deref for Authenticated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &self.claims
    }
}

impl<T: Claims> Authenticated<T> {
    /// Encode claims as JWT encrypted string
    pub fn encode(&self) -> Result<String, jsonwebtoken::errors::Error> {
        encode(
            &jsonwebtoken::Header::new(self.algorithm),
            &*self.claims,
            &self.jwt_encoding_key,
        )
    }
}

impl<T: Claims> FromRequest for Authenticated<T> {
    type Error = actix_web::error::Error;
    type Future = std::future::Ready<Result<Self, actix_web::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req
            .extensions_mut()
            .get::<Authenticated<T>>()
            .map(Clone::clone);
        std::future::ready(value.ok_or_else(|| Error::NotFound.into()))
    }
}

/// Similar to [Authenticated] but JWT is optional
///
/// Examples:
///
/// ```
/// use actix_web::get;
/// use actix_web::HttpResponse;
/// use actix_jwt_session::MaybeAuthenticated;
///
/// # #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// # pub struct Claims { id: uuid::Uuid, sub: String }
/// # impl actix_jwt_session::Claims for Claims {
/// #     fn jti(&self) -> uuid::Uuid { self.id }
/// #     fn subject(&self) -> &str { &self.sub }
/// # }
///
/// // If there's no JWT in request server will NOT automatically returns 401
/// #[get("/session")]
/// async fn read_session(session: MaybeAuthenticated<Claims>) -> HttpResponse {
///     if let Some(session) = session.into_option() {
///         // handle authenticated request
///     }
///     HttpResponse::Ok().finish()
/// }
/// ```
pub struct MaybeAuthenticated<ClaimsType: Claims>(Option<Authenticated<ClaimsType>>);

impl<ClaimsType: Claims> MaybeAuthenticated<ClaimsType> {
    pub fn is_authenticated(&self) -> bool {
        self.0.is_some()
    }

    /// Transform extractor to simple [Option] with [Some] containing
    /// [Authenticated] as value. This allow to handle signed in request and
    /// encrypt claims if needed
    pub fn into_option(self) -> Option<Authenticated<ClaimsType>> {
        self.0
    }
}

impl<ClaimsType: Claims> std::ops::Deref for MaybeAuthenticated<ClaimsType> {
    type Target = Option<Authenticated<ClaimsType>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: Claims> FromRequest for MaybeAuthenticated<T> {
    type Error = actix_web::error::Error;
    type Future = std::future::Ready<Result<Self, actix_web::Error>>;

    fn from_request(
        req: &actix_web::HttpRequest,
        _payload: &mut actix_web::dev::Payload,
    ) -> Self::Future {
        let value = req
            .extensions_mut()
            .get::<Authenticated<T>>()
            .map(Clone::clone);
        std::future::ready(Ok(MaybeAuthenticated(value)))
    }
}

/// Allows to customize where and how sessions are stored in persistant storage.
/// By default redis can be used to store sesions but it's possible and easy to
/// use memcached or postgresql.
#[async_trait(?Send)]
pub trait TokenStorage: Send + Sync {
    /// Load claims from storage or returns [Error] if record does not exists or
    /// there was other error while trying to fetch data from storage.
    async fn get_by_jti(self: Arc<Self>, jti: &[u8]) -> Result<Vec<u8>, Error>;

    /// Save claims in storage in a way claims can be loaded from database using
    /// `jti` as [uuid::Uuid] (JWT ID)
    async fn set_by_jti(
        self: Arc<Self>,
        jwt_jti: &[u8],
        refresh_jti: &[u8],
        bytes: &[u8],
        exp: Duration,
    ) -> Result<(), Error>;

    /// Erase claims from storage. You may ignore if claims does not exists in
    /// storage. Redis implementation returns [Error::NotFound] if record
    /// does not exists.
    async fn remove_by_jti(self: Arc<Self>, jti: &[u8]) -> Result<(), Error>;
}

/// Allow to save, read and remove session from storage.
#[derive(Clone)]
pub struct SessionStorage {
    storage: Arc<dyn TokenStorage>,
    jwt_encoding_key: Arc<EncodingKey>,
    algorithm: Algorithm,
}

impl std::ops::Deref for SessionStorage {
    type Target = Arc<dyn TokenStorage>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

#[doc(hidden)]
/// This structure is saved to session storage (for example Redis)
/// It's internal structure and should not be used unless you plan to create new
/// session storage
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SessionRecord {
    refresh_jti: uuid::Uuid,
    jwt_jti: uuid::Uuid,
    refresh_token: String,
    jwt: String,
}

impl SessionRecord {
    /// Create new record from user claims and generated refresh token
    ///
    /// Both claims are serialized to text and saved as a string
    fn new<ClaimsType: Claims>(claims: ClaimsType, refresh: RefreshToken) -> Result<Self, Error> {
        let refresh_jti = claims.jti();
        let jwt_jti = refresh.refresh_jti;
        let refresh_token = serde_json::to_string(&refresh).map_err(|e| {
            #[cfg(feature = "use-tracing")]
            tracing::debug!("Failed to serialize Refresh Token to construct pair: {e:?}");
            Error::SerializeFailed
        })?;
        let jwt = serde_json::to_string(&claims).map_err(|e| {
            #[cfg(feature = "use-tracing")]
            tracing::debug!("Failed to serialize JWT from to construct pair {e:?}");
            Error::SerializeFailed
        })?;
        Ok(Self {
            refresh_jti,
            jwt_jti,
            refresh_token,
            jwt,
        })
    }

    /// Deserialize loaded refresh token
    fn refresh_token(&self) -> Result<RefreshToken, Error> {
        serde_json::from_str(&self.refresh_token).map_err(|e| {
            #[cfg(feature = "use-tracing")]
            tracing::debug!("Failed to deserialize refresh token from pair: {e:?}");
            Error::RecordMalformed
        })
    }

    /// Deserialize field content to structure
    fn from_field<CT: Claims>(s: &str) -> Result<CT, Error> {
        serde_json::from_str(s).map_err(|e| {
            #[cfg(feature = "use-tracing")]
            tracing::debug!(
                "Failed to deserialize {} for pair: {e:?}",
                std::any::type_name::<CT>()
            );
            Error::RecordMalformed
        })
    }

    /// Serialize refresh token in this record and replace field with generated
    /// text
    fn set_refresh_token(&mut self, mut refresh: RefreshToken) -> Result<(), Error> {
        refresh.expiration_time = refresh.refresh_ttl.0.as_seconds_f64() as u64;
        let refresh_token = serde_json::to_string(&refresh).map_err(|e| {
            #[cfg(feature = "use-tracing")]
            tracing::debug!("Failed to serialize refresh token for pair: {e:?}");
            Error::SerializeFailed
        })?;
        self.refresh_token = refresh_token;
        Ok(())
    }
}

impl SessionStorage {
    /// Abstraction layer over database holding tokens information
    ///
    /// It allows read/write/update/delete operation on tokens
    pub fn new(
        storage: Arc<dyn TokenStorage>,
        jwt_encoding_key: Arc<EncodingKey>,
        algorithm: Algorithm,
    ) -> Self {
        Self {
            storage,
            jwt_encoding_key,
            algorithm,
        }
    }

    /// Load claims from storage or returns [Error] if record does not exists or
    /// there was other error while trying to fetch data from storage.
    pub async fn find_jwt<ClaimsType: Claims>(&self, jti: uuid::Uuid) -> Result<ClaimsType, Error> {
        let record = self.load_pair_by_jwt(jti).await?;
        let refresh_token = record.refresh_token()?;
        if std::any::type_name::<ClaimsType>() == std::any::type_name::<RefreshToken>() {
            SessionRecord::from_field(&record.refresh_token)
        } else {
            if !refresh_token.is_access_valid() {
                #[cfg(feature = "use-tracing")]
                tracing::debug!("JWT expired");
                return Err(Error::JWTExpired);
            }
            SessionRecord::from_field(&record.jwt)
        }
    }

    /// Changes [RefreshToken::issues_at] allowing Claims and RefreshToken to be
    /// accessible longer
    ///
    /// Examples:
    ///
    /// ```
    /// use actix_jwt_session::SessionStorage;
    /// use actix_web::{Error, HttpResponse};
    ///
    /// #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    /// pub struct AppClaims { id: uuid::Uuid, sub: String }
    /// impl actix_jwt_session::Claims for AppClaims {
    ///     fn jti(&self) -> uuid::Uuid { self.id }
    ///     fn subject(&self) -> &str { &self.sub }
    /// }
    ///
    /// async fn extend_tokens_lifetime(
    ///     session_storage: SessionStorage,
    ///     jti: uuid::Uuid
    /// ) -> Result<HttpResponse, Error> {
    ///     session_storage.refresh::<AppClaims>(jti).await?;
    ///     Ok(HttpResponse::Ok().finish())
    /// }
    /// ```
    pub async fn refresh<ClaimsType: Claims>(
        &self,
        refresh_jti: uuid::Uuid,
    ) -> Result<Pair<ClaimsType>, Error> {
        let mut record = self.load_pair_by_refresh(refresh_jti).await?;
        let mut refresh_token = record.refresh_token()?;
        let ttl = refresh_token.refresh_ttl;
        refresh_token.issues_at = OffsetDateTime::now_utc();
        record.set_refresh_token(refresh_token)?;
        self.store_pair(record.clone(), ttl).await?;

        let claims = SessionRecord::from_field::<ClaimsType>(&record.jwt)?;
        let refresh = SessionRecord::from_field::<RefreshToken>(&record.refresh_token)?;
        Ok(Pair {
            jwt: Authenticated {
                claims: Arc::new(claims),
                jwt_encoding_key: self.jwt_encoding_key.clone(),
                algorithm: self.algorithm,
            },
            refresh: Authenticated {
                claims: Arc::new(refresh),
                jwt_encoding_key: self.jwt_encoding_key.clone(),
                algorithm: self.algorithm,
            },
        })
    }

    /// Save claims in storage in a way claims can be loaded from database using
    /// `jti` as [uuid::Uuid] (JWT ID)
    pub async fn store<ClaimsType: Claims>(
        &self,
        claims: ClaimsType,
        access_ttl: JwtTtl,
        refresh_ttl: RefreshTtl,
    ) -> Result<Pair<ClaimsType>, Error> {
        let now = OffsetDateTime::now_utc();
        let refresh = RefreshToken {
            refresh_jti: uuid::Uuid::new_v4(),
            refresh_ttl,
            access_jti: claims.jti().hyphenated().to_string(),
            access_ttl,
            issues_at: now,
            expiration_time: refresh_ttl.0.as_seconds_f64() as u64,
            issuer: claims.jti().hyphenated().to_string(),
            not_before: 0,
            audience: claims.subject().to_string(),
        };

        let record = SessionRecord::new(claims.clone(), refresh.clone())?;
        self.store_pair(record, refresh_ttl).await?;

        Ok(Pair {
            jwt: Authenticated {
                claims: Arc::new(claims),
                jwt_encoding_key: self.jwt_encoding_key.clone(),
                algorithm: self.algorithm,
            },
            refresh: Authenticated {
                claims: Arc::new(refresh),
                jwt_encoding_key: self.jwt_encoding_key.clone(),
                algorithm: self.algorithm,
            },
        })
    }

    /// Erase claims from storage. You may ignore if claims does not exists in
    /// storage. Redis implementation returns [Error::NotFound] if record
    /// does not exists.
    pub async fn erase<ClaimsType: Claims>(&self, jti: Uuid) -> Result<(), Error> {
        let record = self.load_pair_by_jwt(jti).await?;

        self.storage
            .clone()
            .remove_by_jti(record.refresh_jti.as_bytes())
            .await?;
        self.storage
            .clone()
            .remove_by_jti(record.jwt_jti.as_bytes())
            .await?;

        Ok(())
    }

    /// Write to storage tokens pair as [SessionRecord]
    /// This operation allows to load pair using JWT ID and Refresh Token ID
    async fn store_pair(
        &self,
        record: SessionRecord,
        refresh_ttl: RefreshTtl,
    ) -> Result<(), Error> {
        let value = bincode::serialize(&record).map_err(|e| {
            #[cfg(feature = "use-tracing")]
            tracing::debug!("Serialize pair to bytes failed: {e:?}");
            Error::SerializeFailed
        })?;

        self.storage
            .clone()
            .set_by_jti(
                record.jwt_jti.as_bytes(),
                record.refresh_jti.as_bytes(),
                &value,
                refresh_ttl.0,
            )
            .await?;

        Ok(())
    }

    /// Load [SessionRecord] as tokens pair from storage using JWT ID (jti)
    async fn load_pair_by_jwt(&self, jti: Uuid) -> Result<SessionRecord, Error> {
        self.storage
            .clone()
            .get_by_jti(jti.as_bytes())
            .await
            .and_then(|bytes| {
                bincode::deserialize(&bytes).map_err(|e| {
                    #[cfg(feature = "use-tracing")]
                    tracing::debug!("Deserialize pair while loading for JWT ID failed: {e:?}");
                    Error::RecordMalformed
                })
            })
    }

    /// Load [SessionRecord] as tokens pair from storage using Refresh ID (jti)
    async fn load_pair_by_refresh(&self, jti: Uuid) -> Result<SessionRecord, Error> {
        self.storage
            .clone()
            .get_by_jti(jti.as_bytes())
            .await
            .and_then(|bytes| {
                bincode::deserialize(&bytes).map_err(|e| {
                    #[cfg(feature = "use-tracing")]
                    tracing::debug!("Deserialize pair while loading for refresh id failed: {e:?}");
                    Error::RecordMalformed
                })
            })
    }
}

pub mod builder;
pub use builder::*;

#[cfg(feature = "routes")]
pub mod actix_routes;
#[cfg(feature = "routes")]
pub use actix_routes::configure;

mod extractors;
pub use extractors::*;

/// Load or generate new Ed25519 signing keys.
///
/// [JwtSigningKeys::load_or_create] should be called only once at the boot of
/// the server.
///
/// If there's any issue during generating new keys or loading exiting one
/// application will panic.
///
/// Examples:
///
/// ```rust
/// use actix_jwt_session::*;
///
/// pub fn boot_server() {
///     let keys = JwtSigningKeys::load_or_create();
/// }
/// ```
pub struct JwtSigningKeys {
    pub encoding_key: EncodingKey,
    pub decoding_key: DecodingKey,
}

impl JwtSigningKeys {
    /// Loads signing keys from `./config` directory or creates new pair and
    /// save it to directory.
    ///
    /// Pair is composed of encode key and decode key saved in
    /// `./config/jwt-encoding.bin` and `./config/jwt-decoding.bin`
    /// written as binary file.
    ///
    /// Decode key can be transform to base64 and shared with clients if this is
    /// required.
    ///
    /// Files must be shared between restarts otherwise all old sessions will be
    /// invalidated.
    pub fn load_or_create() -> Self {
        match Self::load_from_files() {
            Ok(s) => s,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Self::generate(true).expect("Generating new jwt signing keys must succeed")
            }
            Err(e) => panic!("Failed to load or generate jwt signing keys: {:?}", e),
        }
    }

    pub fn generate(save: bool) -> Result<Self, Box<dyn std::error::Error>> {
        use jsonwebtoken::*;
        use ring::rand::SystemRandom;
        use ring::signature::{Ed25519KeyPair, KeyPair};

        let doc = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())?;
        let keypair = Ed25519KeyPair::from_pkcs8(doc.as_ref())?;
        let encoding_key = EncodingKey::from_ed_der(doc.as_ref());
        let decoding_key = DecodingKey::from_ed_der(keypair.public_key().as_ref());

        if save {
            std::fs::write("./config/jwt-encoding.bin", doc.as_ref()).unwrap_or_else(|e| {
                panic!("Failed to write ./config/jwt-encoding.bin: {:?}", e);
            });
            std::fs::write("./config/jwt-decoding.bin", keypair.public_key()).unwrap_or_else(|e| {
                panic!("Failed to write ./config/jwt-decoding.bin: {:?}", e);
            });
        }

        Ok(JwtSigningKeys {
            encoding_key,
            decoding_key,
        })
    }

    pub fn load_from_files() -> std::io::Result<Self> {
        use std::io::Read;

        use jsonwebtoken::*;

        let mut buf = Vec::new();
        let mut e = std::fs::File::open("./config/jwt-encoding.bin")?;
        e.read_to_end(&mut buf).unwrap_or_else(|e| {
            panic!("Failed to read jwt encoding key: {:?}", e);
        });
        let encoding_key: EncodingKey = EncodingKey::from_ed_der(&buf);

        let mut buf = Vec::new();
        let mut e = std::fs::File::open("./config/jwt-decoding.bin")?;
        e.read_to_end(&mut buf).unwrap_or_else(|e| {
            panic!("Failed to read jwt decoding key: {:?}", e);
        });
        let decoding_key = DecodingKey::from_ed_der(&buf);
        Ok(Self {
            encoding_key,
            decoding_key,
        })
    }
}

#[macro_export]
macro_rules! bad_ttl {
    ($ttl: expr, $min: expr, $panic_msg: expr) => {
        if $ttl < $min {
            #[cfg(feature = "use-tracing")]
            tracing::warn!(
                "Expiration time is bellow 1s. This is not allowed for redis server. Overriding!"
            );
            if cfg!(feature = "panic-bad-ttl") {
                panic!($panic_msg);
            } else if cfg!(feature = "override-bad-ttl") {
                $ttl = $min;
            }
        }
    };
}

mod middleware;
pub use middleware::*;

#[cfg(feature = "redis")]
mod redis_adapter;
#[allow(unused_imports)]
#[cfg(feature = "redis")]
pub use redis_adapter::*;
#[cfg(feature = "hashing")]
mod hashing;
#[cfg(feature = "hashing")]
pub use hashing::*;
