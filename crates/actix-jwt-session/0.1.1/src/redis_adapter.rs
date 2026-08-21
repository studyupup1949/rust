//! Default session storage which uses async redis requests
//!
//! Sessions are serialized to binary format and stored using [uuid::Uuid] key as bytes.
//! All sessions must have expirations time after which they will be automatically removed by
//! redis.
//!
//! [RedisStorage] is constructed by [RedisMiddlewareFactory] from [redis_async_pool::RedisPool] and shared
//! between all [RedisMiddleware] instances.

use super::*;
use actix_web::dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform};
use futures_util::future::LocalBoxFuture;
use redis::AsyncCommands;
use std::future::{ready, Ready};
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

/// Redis implementation for [TokenStorage]
#[derive(Clone)]
struct RedisStorage<ClaimsType: Claims> {
    pool: redis_async_pool::RedisPool,
    _claims_type_marker: PhantomData<ClaimsType>,
}

impl<ClaimsType: Claims> RedisStorage<ClaimsType> {
    pub fn new(pool: redis_async_pool::RedisPool) -> Self {
        Self {
            pool,
            _claims_type_marker: Default::default(),
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<ClaimsType> TokenStorage for RedisStorage<ClaimsType>
where
    ClaimsType: Claims,
{
    type ClaimsType = ClaimsType;

    async fn get_from_jti(self: Arc<Self>, jti: uuid::Uuid) -> Result<ClaimsType, Error> {
        let pool = self.pool.clone();
        let mut conn = pool.get().await.map_err(|_| Error::RedisConn)?;
        let val = conn
            .get::<_, Vec<u8>>(jti.as_bytes())
            .await
            .map_err(|_| Error::NotFound)?;
        bincode::deserialize(&val).map_err(|_| Error::RecordMalformed)
    }

    async fn set_by_jti(
        self: Arc<Self>,
        claims: Self::ClaimsType,
        exp: std::time::Duration,
    ) -> Result<(), Error> {
        let pool = self.pool.clone();
        let mut conn = pool.get().await.map_err(|_| Error::RedisConn)?;
        let val = bincode::serialize(&claims).map_err(|_| Error::SerializeFailed)?;
        conn.set_ex::<_, _, String>(claims.jti().as_bytes(), val, exp.as_secs() as usize)
            .await
            .map_err(|_| Error::WriteFailed)?;
        Ok(())
    }

    async fn remove_by_jti(self: Arc<Self>, jti: Uuid) -> Result<(), Error> {
        let pool = self.pool.clone();
        let mut conn = pool.get().await.map_err(|_| Error::RedisConn)?;
        conn.del(jti.as_bytes())
            .await
            .map_err(|_| Error::NotFound)?;
        Ok(())
    }
}

pub struct RedisMiddleware<S, ClaimsType>
where
    ClaimsType: Claims,
{
    _claims_type_marker: std::marker::PhantomData<ClaimsType>,
    service: Rc<S>,
    jwt_encoding_key: Arc<EncodingKey>,
    jwt_decoding_key: Arc<DecodingKey>,
    algorithm: Algorithm,
    storage: SessionStorage<ClaimsType>,
    extractors: Arc<Vec<Box<dyn SessionExtractor<ClaimsType>>>>,
}

impl<S, B, ClaimsType> Service<ServiceRequest> for RedisMiddleware<S, ClaimsType>
where
    ClaimsType: Claims,
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        use futures_lite::FutureExt;

        let svc = self.service.clone();
        let jwt_decoding_key = self.jwt_decoding_key.clone();
        let jwt_encoding_key = self.jwt_encoding_key.clone();
        let algorithm = self.algorithm;
        let storage = self.storage.clone();
        let extractors = self.extractors.clone();

        async move {
            let mut last_error = None;
            for extractor in extractors.iter() {
                match extractor
                    .extract_jwt(
                        &req,
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
            let res = svc.call(req).await?;
            Ok(res)
        }
        .boxed_local()
    }
}

#[derive(Clone)]
pub struct RedisMiddlewareFactory<ClaimsType: Claims> {
    jwt_encoding_key: Arc<EncodingKey>,
    jwt_decoding_key: Arc<DecodingKey>,
    algorithm: Algorithm,
    storage: SessionStorage<ClaimsType>,
    extractors: Arc<Vec<Box<dyn SessionExtractor<ClaimsType>>>>,
    _claims_type_marker: PhantomData<ClaimsType>,
}

impl<ClaimsType: Claims> RedisMiddlewareFactory<ClaimsType> {
    pub fn new(
        jwt_encoding_key: Arc<EncodingKey>,
        jwt_decoding_key: Arc<DecodingKey>,
        algorithm: Algorithm,
        pool: redis_async_pool::RedisPool,
        extractors: Vec<Box<dyn SessionExtractor<ClaimsType>>>,
    ) -> Self {
        let storage = Arc::new(RedisStorage::new(pool));

        Self {
            jwt_encoding_key: jwt_encoding_key.clone(),
            jwt_decoding_key,
            algorithm,
            storage: SessionStorage {
                storage,
                jwt_encoding_key: jwt_encoding_key.clone(),
                algorithm,
            },
            extractors: Arc::new(extractors),
            _claims_type_marker: Default::default(),
        }
    }

    pub fn storage(&self) -> SessionStorage<ClaimsType> {
        self.storage.clone()
    }
}

impl<S, B, ClaimsType> Transform<S, ServiceRequest> for RedisMiddlewareFactory<ClaimsType>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = actix_web::Error> + 'static,
    ClaimsType: Claims,
{
    type Response = ServiceResponse<B>;
    type Error = actix_web::Error;
    type Transform = RedisMiddleware<S, ClaimsType>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RedisMiddleware {
            service: Rc::new(service),
            storage: self.storage.clone(),
            jwt_encoding_key: self.jwt_encoding_key.clone(),
            jwt_decoding_key: self.jwt_decoding_key.clone(),
            algorithm: self.algorithm,
            extractors: self.extractors.clone(),
            _claims_type_marker: PhantomData,
        }))
    }
}
