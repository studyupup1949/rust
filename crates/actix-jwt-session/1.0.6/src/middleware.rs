//! Create session storage and build middleware factory

use std::future::{ready, Ready};
use std::rc::Rc;
use std::sync::Arc;

// pub use actix_web::cookie::time::{Duration, OffsetDateTime};
use actix_web::dev::Transform;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse};
use futures_util::future::LocalBoxFuture;
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey};

use crate::*;

/// Session middleware factory builder
///
/// It should be constructed with [SessionMiddlewareFactory::build].
pub struct SessionMiddlewareBuilder<ClaimsType: Claims> {
    pub(crate) jwt_encoding_key: Arc<EncodingKey>,
    pub(crate) jwt_decoding_key: Arc<DecodingKey>,
    pub(crate) algorithm: Algorithm,
    pub(crate) storage: Option<SessionStorage>,
    pub(crate) extractors: Extractors<ClaimsType>,
}
impl<ClaimsType: Claims> SessionMiddlewareBuilder<ClaimsType> {
    #[doc(hidden)]
    pub(crate) fn new(
        jwt_encoding_key: Arc<EncodingKey>,
        jwt_decoding_key: Arc<DecodingKey>,
        algorithm: Algorithm,
    ) -> Self {
        Self {
            jwt_encoding_key: jwt_encoding_key.clone(),
            jwt_decoding_key,
            algorithm,
            storage: None,
            extractors: Extractors::default(),
        }
    }

    pub(crate) fn auto_ed_dsa() -> Self {
        let keys = JwtSigningKeys::load_or_create();
        Self::new(
            Arc::new(keys.encoding_key),
            Arc::new(keys.decoding_key),
            Algorithm::EdDSA,
        )
    }

    /// Set session storage to given instance. Good if for some reason you need
    /// to share 1 storage with multiple instances of session middleware
    #[must_use]
    pub fn with_storage(mut self, storage: SessionStorage) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Set how session and refresh token should be extracted
    #[must_use]
    pub fn with_extractors(mut self, extractors: Extractors<ClaimsType>) -> Self {
        self.extractors = extractors;
        self
    }

    /// Builds middleware factory and returns session storage with factory
    pub fn finish(self) -> (SessionStorage, SessionMiddlewareFactory<ClaimsType>) {
        let Self {
            storage,
            jwt_encoding_key,
            jwt_decoding_key,
            algorithm,
            extractors,
            ..
        } = self;
        let storage = storage
            .expect("Session storage must be constracted from pool or set from existing storage");
        (
            storage.clone(),
            SessionMiddlewareFactory {
                jwt_encoding_key,
                jwt_decoding_key,
                algorithm,
                storage,
                extractors,
            },
        )
    }
}

/// Factory creates middlware for every single request.
///
/// All fields here are immutable and have atomic access and only pointer is
/// copied so are very cheap
///
/// Example:
///
/// ```
/// use std::sync::Arc;
/// use actix_jwt_session::*;
///
/// # async fn create<AppClaims: actix_jwt_session::Claims>() {
/// // create redis connection
/// let redis = {
///     use deadpool_redis::{Config, Runtime};
///     let cfg = Config::from_url("redis://localhost:6379");
///     let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
///     pool
/// };
///
/// // load or create new keys in `./config`
/// let keys = JwtSigningKeys::load_or_create();
///
/// // create new [SessionStorage] and [SessionMiddlewareFactory]
/// let (storage, factory) = SessionMiddlewareFactory::<AppClaims>::build(
///     Arc::new(keys.encoding_key),
///     Arc::new(keys.decoding_key),
///     Algorithm::EdDSA
/// )
/// // pass redis connection
/// .with_redis_pool(redis.clone())
/// .with_extractors(
///     Extractors::default()
///     // Check if header "Authorization" exists and contains Bearer with encoded JWT
///     .with_jwt_header("Authorization")
///     // Check if cookie "jwt" exists and contains encoded JWT
///     .with_jwt_cookie("acx-a")
///     .with_refresh_header("ACX-Refresh")
///     // Check if cookie "jwt" exists and contains encoded JWT
///     .with_refresh_cookie("acx-r")
/// )
/// .finish();
/// # }
/// ```
#[derive(Clone)]
pub struct SessionMiddlewareFactory<ClaimsType: Claims> {
    pub(crate) jwt_encoding_key: Arc<EncodingKey>,
    pub(crate) jwt_decoding_key: Arc<DecodingKey>,
    pub(crate) algorithm: Algorithm,
    pub(crate) storage: SessionStorage,
    pub(crate) extractors: Extractors<ClaimsType>,
}

impl<ClaimsType: Claims> SessionMiddlewareFactory<ClaimsType> {
    pub fn build_ed_dsa() -> SessionMiddlewareBuilder<ClaimsType> {
        SessionMiddlewareBuilder::auto_ed_dsa()
    }

    pub fn build(
        jwt_encoding_key: Arc<EncodingKey>,
        jwt_decoding_key: Arc<DecodingKey>,
        algorithm: Algorithm,
    ) -> SessionMiddlewareBuilder<ClaimsType> {
        SessionMiddlewareBuilder::new(jwt_encoding_key, jwt_decoding_key, algorithm)
    }
}

impl<S, ClaimsType> Transform<S, ServiceRequest> for SessionMiddlewareFactory<ClaimsType>
where
    S: Service<ServiceRequest, Error = actix_web::Error, Response = ServiceResponse> + 'static,
    ClaimsType: Claims,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Transform = SessionMiddleware<S, ClaimsType>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(SessionMiddleware {
            service: Rc::new(service),
            storage: self.storage.clone(),
            jwt_encoding_key: self.jwt_encoding_key.clone(),
            jwt_decoding_key: self.jwt_decoding_key.clone(),
            algorithm: self.algorithm,
            extractors: self.extractors.clone(),
        }))
    }
}

#[doc(hidden)]
pub struct SessionMiddleware<S, ClaimsType>
where
    ClaimsType: Claims,
{
    pub(crate) service: Rc<S>,
    pub(crate) jwt_encoding_key: Arc<EncodingKey>,
    pub(crate) jwt_decoding_key: Arc<DecodingKey>,
    pub(crate) algorithm: Algorithm,
    pub(crate) storage: SessionStorage,
    pub(crate) extractors: Extractors<ClaimsType>,
}

impl<S, ClaimsType: Claims> SessionMiddleware<S, ClaimsType> {
    async fn extract_token<C: Claims>(
        req: &mut ServiceRequest,
        jwt_encoding_key: Arc<EncodingKey>,
        jwt_decoding_key: Arc<DecodingKey>,
        algorithm: Algorithm,
        storage: SessionStorage,
        extractors: &[Arc<dyn SessionExtractor<C>>],
    ) -> Result<(), Error> {
        let mut last_error = None;
        for extractor in extractors.iter() {
            match extractor
                .extract_claims(
                    req,
                    jwt_encoding_key.clone(),
                    jwt_decoding_key.clone(),
                    algorithm,
                    storage.clone(),
                )
                .await
            {
                Ok(_) => break,
                Err(e) => {
                    last_error = Some(e);
                }
            };
        }
        if let Some(e) = last_error {
            return Err(e)?;
        }
        Ok(())
    }
}

impl<S, ClaimsType> Service<ServiceRequest> for SessionMiddleware<S, ClaimsType>
where
    ClaimsType: Claims,
    S: Service<ServiceRequest, Response = ServiceResponse, Error = actix_web::Error> + 'static,
{
    type Response = ServiceResponse;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        use futures_lite::FutureExt;

        let svc = self.service.clone();
        let jwt_decoding_key = self.jwt_decoding_key.clone();
        let jwt_encoding_key = self.jwt_encoding_key.clone();
        let algorithm = self.algorithm;
        let storage = self.storage.clone();
        let extractors = self.extractors.clone();

        async move {
            if !extractors.jwt_extractors.is_empty() {
                Self::extract_token(
                    &mut req,
                    jwt_encoding_key.clone(),
                    jwt_decoding_key.clone(),
                    algorithm,
                    storage.clone(),
                    &extractors.jwt_extractors,
                )
                .await?;
            }
            if !extractors.refresh_extractors.is_empty() {
                Self::extract_token(
                    &mut req,
                    jwt_encoding_key,
                    jwt_decoding_key,
                    algorithm,
                    storage,
                    &extractors.refresh_extractors,
                )
                .await?;
            }
            let res = svc.call(req).await?;
            Ok(res)
        }
        .boxed_local()
    }
}
