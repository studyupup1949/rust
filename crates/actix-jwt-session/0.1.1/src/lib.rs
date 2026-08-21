//! General purpose JWT session validator for actix_web
//!
//! It's designed to extract session using middleware and validate path simply by using extractors.
//!
//! Examples:
//!
//! ```no_run
//! use std::boxed::Box;
//! use std::sync::Arc;
//! use actix_jwt_session::*;
//! use actix_web::get;
//! use actix_web::web::Data;
//! use actix_web::{HttpResponse, App, HttpServer};
//! use ring::rand::SystemRandom;
//! use ring::signature::{Ed25519KeyPair, KeyPair};
//! use jsonwebtoken::*;
//! use serde::{Serialize, Deserialize};
//!
//! #[tokio::main]
//! async fn main() {
//!     let redis = {
//!         use redis_async_pool::{RedisConnectionManager, RedisPool};
//!         RedisPool::new(
//!             RedisConnectionManager::new(
//!                 redis::Client::open("redis://localhost:6379").expect("Fail to connect to redis"),
//!                 true,
//!                 None,
//!             ),
//!             5,
//!         )
//!     };
//!  
//!     let keys = JwtSigningKeys::generate().unwrap();
//!     let factory = RedisMiddlewareFactory::<AppClaims>::new(
//!         Arc::new(keys.encoding_key),
//!         Arc::new(keys.decoding_key),
//!         Algorithm::EdDSA,
//!         redis.clone(),
//!         vec![
//!             // Check if header "Authorization" exists and contains Bearer with encoded JWT
//!             Box::new(HeaderExtractor::new("Authorization")),
//!             // Check if cookie "jwt" exists and contains encoded JWT
//!             Box::new(CookieExtractor::new("jwt")),
//!         ]
//!     );
//!  
//!     HttpServer::new(move || {
//!         let factory = factory.clone();
//!         App::new()
//!             .app_data(Data::new(factory.storage()))
//!             .wrap(factory)
//!             .app_data(Data::new(redis.clone()))
//!             .service(storage_access)
//!             .service(must_be_signed_in)
//!             .service(may_be_signed_in)
//!     })
//!     .bind(("0.0.0.0", 8080)).unwrap()
//!     .run()
//!     .await.unwrap();
//! }
//!
//! #[derive(Clone, PartialEq, Serialize, Deserialize)]
//! pub struct AppClaims {
//!     id: uuid::Uuid,
//!     subject: String,
//! }
//!
//! impl Claims for AppClaims {
//!     fn jti(&self) -> uuid::Uuid { self.id }
//!     fn subject(&self) -> &str { &self.subject }
//! }
//!
//! #[derive(Clone, PartialEq, Serialize, Deserialize)]
//! pub struct SessionData {
//!     id: uuid::Uuid,
//!     subject: String,
//! }
//!
//! #[actix_web::post("/access-storage")]
//! async fn storage_access(
//!     session_store: Data<SessionStorage<AppClaims>>, 
//!     p: actix_web::web::Json<SessionData>,
//! ) -> HttpResponse {
//!     let p = p.into_inner();
//!     session_store.store(AppClaims {
//!         id: p.id,
//!         subject: p.subject,
//!     }, std::time::Duration::from_secs(60 * 60 * 24 * 14) ).await.unwrap();
//!     HttpResponse::Ok().body("")
//! }
//!
//! #[get("/authorized")]
//! async fn must_be_signed_in(session: Authenticated<AppClaims>) -> HttpResponse {
//!     let jit = session.jti();
//!     HttpResponse::Ok().body("")
//! }
//!
//! #[get("/maybe-authorized")]
//! async fn may_be_signed_in(session: MaybeAuthenticated<AppClaims>) -> HttpResponse {
//!     if let Some(session) = session.into_option() {
//!     }
//!     HttpResponse::Ok().body("")
//! }
//!
//! pub struct JwtSigningKeys {
//!     encoding_key: EncodingKey,
//!     decoding_key: DecodingKey,
//! }
//!
//! impl JwtSigningKeys {
//!     fn generate() -> Result<Self, Box<dyn std::error::Error>> {
//!         let doc = Ed25519KeyPair::generate_pkcs8(&SystemRandom::new())?;
//!         let keypair = Ed25519KeyPair::from_pkcs8(doc.as_ref())?;
//!         let encoding_key = EncodingKey::from_ed_der(doc.as_ref());
//!         let decoding_key = DecodingKey::from_ed_der(keypair.public_key().as_ref());
//!         Ok(JwtSigningKeys {
//!             encoding_key,
//!             decoding_key,
//!         })
//!     }
//! }
//! ```

use actix_web::{dev::ServiceRequest, HttpResponse};
use actix_web::{FromRequest, HttpMessage};
use async_trait::async_trait;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{de::DeserializeOwned, Serialize};
use std::marker::PhantomData;
use std::sync::Arc;
use uuid::Uuid;

/// Default authorization header is "Authorization"
pub static DEFAULT_HEADER_NAME: &str = "Authorization";

/// Serializable and storable struct which represent JWT claims
///
/// * It must have JWT ID as [uuid::Uuid]
/// * It must have subject as a String
pub trait Claims: PartialEq + DeserializeOwned + Serialize + Clone + Send + Sync + 'static {
    fn jti(&self) -> uuid::Uuid;
    fn subject(&self) -> &str;
}

/// Session related errors
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error("Failed to obtain redis connection")]
    RedisConn,
    #[error("Record not found")]
    NotFound,
    #[error("Record malformed")]
    RecordMalformed,
    #[error("Invalid session")]
    InvalidSession,
    #[error("No http authentication header")]
    NoAuthHeader,
    #[error("Failed to serialize claims")]
    SerializeFailed,
    #[error("Unable to write claims to storage")]
    WriteFailed,
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
/// If there's no JWT endpoint which requires this structure will automatically returns `401`.
///
/// Examples:
///
/// ```
/// use actix_web::get;
/// use actix_web::HttpResponse;
/// use actix_jwt_session::Authenticated;
///
/// # #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
/// # pub struct Claims { id: uuid::Uuid, sub: String }
/// # impl actix_jwt_session::Claims for Claims {
/// #     fn jti(&self) -> uuid::Uuid { self.id }
/// #     fn subject(&self) -> &str { &self.sub }
/// # }
///
/// // If there's no JWT in request server will automatically returns 401
/// #[get("/session")]
/// async fn read_session(session: Authenticated<Claims>) -> HttpResponse {
///     let encoded = session.encode().unwrap(); // JWT as encrypted string
///     HttpResponse::Ok().finish()
/// }
/// ```
#[derive(Clone)]
#[cfg_attr(feature = "serde-transparent", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde-transparent", serde(transparent))]
pub struct Authenticated<T> {
    pub claims: Arc<T>,
    pub jwt_encoding_key: Arc<EncodingKey>,
    pub algorithm: Algorithm,
}

impl<T> std::ops::Deref for Authenticated<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        &*self.claims
    }
}

impl<T: Claims> Authenticated<T> {
    /// Encode claims as JWT encrypted string
    pub fn encode(&self) -> Result<String, jsonwebtoken::errors::Error> {
        encode(
            &jsonwebtoken::Header::new(self.algorithm),
            &*self.claims,
            &*self.jwt_encoding_key,
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

    /// Transform extractor to simple [Option] with [Some] containing [Authenticated] as value.
    /// This allow to handle signed in request and encrypt claims if needed
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
/// By default redis can be used to store sesions but it's possible and easy to use memcached or
/// postgresql.
#[async_trait(?Send)]
pub trait TokenStorage: Send + Sync {
    type ClaimsType: Claims;

    /// Load claims from storage or returns [Error] if record does not exists or there was other
    /// error while trying to fetch data from storage.
    async fn get_from_jti(self: Arc<Self>, jti: uuid::Uuid) -> Result<Self::ClaimsType, Error>;

    /// Save claims in storage in a way claims can be loaded from database using `jti` as [uuid::Uuid] (JWT ID)
    async fn set_by_jti(
        self: Arc<Self>,
        claims: Self::ClaimsType,
        exp: std::time::Duration,
    ) -> Result<(), Error>;

    /// Erase claims from storage. You may ignore if claims does not exists in storage.
    /// Redis implementation returns [Error::NotFound] if record does not exists.
    async fn remove_by_jti(self: Arc<Self>, jti: Uuid) -> Result<(), Error>;
}

/// Allow to save, read and remove session from storage.
#[derive(Clone)]
pub struct SessionStorage<ClaimsType: Claims> {
    storage: Arc<dyn TokenStorage<ClaimsType = ClaimsType>>,
    jwt_encoding_key: Arc<EncodingKey>,
    algorithm: Algorithm,
}

impl<ClaimsType: Claims> std::ops::Deref for SessionStorage<ClaimsType> {
    type Target = Arc<dyn TokenStorage<ClaimsType = ClaimsType>>;

    fn deref(&self) -> &Self::Target {
        &self.storage
    }
}

impl<ClaimsType: Claims> SessionStorage<ClaimsType> {
    pub async fn set_by_jti(
        &self,
        claims: ClaimsType,
        exp: std::time::Duration,
    ) -> Result<(), Error> {
        self.storage.clone().set_by_jti(claims, exp).await
    }

    /// Load claims from storage or returns [Error] if record does not exists or there was other
    /// error while trying to fetch data from storage.
    pub async fn get_from_jti(&self, jti: uuid::Uuid) -> Result<ClaimsType, Error> {
        self.storage.clone().get_from_jti(jti).await
    }

    /// Save claims in storage in a way claims can be loaded from database using `jti` as [uuid::Uuid] (JWT ID)
    pub async fn store(
        &self,
        claims: ClaimsType,
        exp: std::time::Duration,
    ) -> Result<Authenticated<ClaimsType>, Error> {
        self.set_by_jti(claims.clone(), exp).await?;
        Ok(Authenticated {
            claims: Arc::new(claims),
            jwt_encoding_key: self.jwt_encoding_key.clone(),
            algorithm: self.algorithm,
        })
    }

    /// Erase claims from storage. You may ignore if claims does not exists in storage.
    /// Redis implementation returns [Error::NotFound] if record does not exists.
    pub async fn erase(&self, jti: Uuid) -> Result<(), Error> {
        self.storage.clone().remove_by_jti(jti).await
    }
}

/// Trait allowing to extract JWt token from [actix_web::dev::ServiceRequest]
///
/// Two extractor are implemented by default
/// * [HeaderExtractor] which is best for any PWA or micro services requests
/// * [CookieExtractor] which is best for simple server with session stored in cookie
///
/// It's possible to implement GraphQL, JSON payload or query using `req.extract::<JSON<YourStruct>>()` if this is needed.
///
/// All implementation can use [SessionExtractor::decode] method for decoding raw JWT string into
/// Claims and then [SessionExtractor::validate] to validate claims agains session stored in [SessionStorage]
#[async_trait(?Send)]
pub trait SessionExtractor<ClaimsType: Claims>: Send + Sync + 'static {
    /// Extract claims from [actix_web::dev::ServiceRequest]
    ///
    /// Examples:
    ///
    /// ```
    /// use actix_web::dev::ServiceRequest;
    /// use jsonwebtoken::*;
    /// use actix_jwt_session::{Extractor, Authenticated, Error, SessionStorage};
    /// use std::sync::Arc;
    /// use actix_web::HttpMessage;
    /// # #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    /// # pub struct Claims { id: uuid::Uuid, sub: String }
    /// # impl actix_jwt_session::Claims for Claims {
    /// #     fn jti(&self) -> uuid::Uuid { self.id }
    /// #     fn subject(&self) -> &str { &self.sub }
    /// # }
    ///
    /// #[derive(Debug, Clone, Copy, Default)]
    /// struct ExampleExtractor;
    ///
    /// #[async_trait::async_trait(?Send)]
    /// impl Extractor<Claims> for ExampleExtractor {
    ///     async fn extract_jwt(
    ///         &self,
    ///         req: &ServiceRequest,
    ///         jwt_encoding_key: Arc<EncodingKey>,
    ///         jwt_decoding_key: Arc<DecodingKey>,
    ///         algorithm: Algorithm,
    ///         storage: SessionStorage<Claims>,
    ///     ) -> Result<(), Error> {
    ///         if req.peer_addr().unwrap().ip().is_multicast() {
    ///            req.extensions_mut().insert(Authenticated {
    ///                claims: Arc::new(Claims { id: uuid::Uuid::default(), sub: "HUB".into() }),
    ///                jwt_encoding_key,
    ///                algorithm,
    ///            });
    ///         }
    ///         Ok(())
    ///     }
    /// }
    /// ```
    async fn extract_jwt(
        &self,
        req: &ServiceRequest,
        jwt_encoding_key: Arc<EncodingKey>,
        jwt_decoding_key: Arc<DecodingKey>,
        algorithm: Algorithm,
        storage: SessionStorage<ClaimsType>,
    ) -> Result<(), Error>;

    /// Decode encrypted JWT to structure
    fn decode(
        &self,
        value: &str,
        jwt_decoding_key: Arc<DecodingKey>,
        algorithm: Algorithm,
    ) -> Result<ClaimsType, Error> {
        decode::<ClaimsType>(value, &*jwt_decoding_key, &Validation::new(algorithm))
            .map_err(|_e| {
                // let error_message = e.to_string();
                Error::InvalidSession
            })
            .map(|t| t.claims)
    }

    /// Validate JWT Claims agains stored in storage tokens.
    ///
    /// * Token must exists in storage
    /// * Token must be exactly the same as token from storage
    async fn validate(
        &self,
        claims: &ClaimsType,
        storage: SessionStorage<ClaimsType>,
    ) -> Result<(), Error> {
        let stored = storage
            .clone()
            .get_from_jti(claims.jti())
            .await
            .map_err(|_| Error::InvalidSession)?;

        if &stored != claims {
            return Err(Error::InvalidSession);
        }
        Ok(())
    }
}

/// Extracts JWT token from HTTP Request cookies. This extractor should be used when you can't set
/// your own header, for example when user enters http links to browser and you don't have any
/// advanced frontend.
///
/// This exractor is may be used by PWA application or micro services but [HeaderExtractor] is much
/// more suitable for this purpose.
pub struct CookieExtractor<ClaimsType> {
    __ty: PhantomData<ClaimsType>,
    cookie_name: &'static str,
}

impl<ClaimsType: Claims> CookieExtractor<ClaimsType> {
    pub fn new(cookie_name: &'static str) -> Self {
        Self {
            __ty: Default::default(),
            cookie_name,
        }
    }
}

#[async_trait(?Send)]
impl<ClaimsType: Claims> SessionExtractor<ClaimsType> for CookieExtractor<ClaimsType> {
    async fn extract_jwt(
        &self,
        req: &ServiceRequest,
        jwt_encoding_key: Arc<EncodingKey>,
        jwt_decoding_key: Arc<DecodingKey>,
        algorithm: Algorithm,
        storage: SessionStorage<ClaimsType>,
    ) -> Result<(), Error> {
        let Some(cookie) = req.cookie(self.cookie_name) else {
            return Ok(())
        };
        let as_str = cookie.value();
        let decoded_claims = self.decode(as_str, jwt_decoding_key, algorithm)?;
        self.validate(&decoded_claims, storage).await?;
        req.extensions_mut().insert(Authenticated {
            claims: Arc::new(decoded_claims),
            jwt_encoding_key,
            algorithm,
        });
        Ok(())
    }
}

/// Extracts JWT token from HTTP Request headers
///
/// This exractor is very useful for all PWA application or for micro services
/// because you can set your own headers while making http requests.
///
/// If you want to have users authorized using simple html <a> you should use [CookieExtractor]
pub struct HeaderExtractor<ClaimsType> {
    __ty: PhantomData<ClaimsType>,
    header_name: &'static str,
}

impl<ClaimsType: Claims> HeaderExtractor<ClaimsType> {
    pub fn new(header_name: &'static str) -> Self {
        Self {
            __ty: Default::default(),
            header_name,
        }
    }
}

#[async_trait(?Send)]
impl<ClaimsType: Claims> SessionExtractor<ClaimsType> for HeaderExtractor<ClaimsType> {
    async fn extract_jwt(
        &self,
        req: &ServiceRequest,
        jwt_encoding_key: Arc<EncodingKey>,
        jwt_decoding_key: Arc<DecodingKey>,
        algorithm: Algorithm,
        storage: SessionStorage<ClaimsType>,
    ) -> Result<(), Error> {
        let Some(authorisation_header) = req
            .headers()
            .get(self.header_name)
            else {
                return Ok(())
            };
        let as_str = authorisation_header
            .to_str()
            .map_err(|_| Error::NoAuthHeader)?;

        let as_str = as_str
            .strip_prefix("Bearer ")
            .or_else(|| as_str.strip_prefix("bearer "))
            .unwrap_or(as_str);

        let decoded_claims = self.decode(as_str, jwt_decoding_key, algorithm)?;
        self.validate(&decoded_claims, storage).await?;
        req.extensions_mut().insert(Authenticated {
            claims: Arc::new(decoded_claims),
            jwt_encoding_key,
            algorithm,
        });
        Ok(())
    }
}

#[cfg(feature = "redis")]
mod redis_adapter;
#[cfg(feature = "redis")]
pub use redis_adapter::*;
