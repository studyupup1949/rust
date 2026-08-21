//! Default session storage which uses async redis requests
//!
//! Sessions are serialized to binary format and stored using [uuid::Uuid] key
//! as bytes. All sessions must have expirations time after which they will be
//! automatically removed by redis.
//!
//! [RedisStorage] is constructed by [RedisMiddlewareFactory] from
//! [redis_async_pool::RedisPool] and shared between all [RedisMiddleware]
//! instances.

use std::marker::PhantomData;
use std::sync::Arc;

pub use deadpool_redis;
use deadpool_redis::Pool;
use redis::AsyncCommands;

use crate::*;

/// Redis implementation for [TokenStorage]
#[derive(Clone)]
struct RedisStorage<ClaimsType: Claims> {
    pool: Pool,
    _claims_type_marker: PhantomData<ClaimsType>,
}

impl<ClaimsType: Claims> RedisStorage<ClaimsType> {
    pub fn new(pool: Pool) -> Self {
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
    async fn get_by_jti(self: Arc<Self>, jti: &[u8]) -> Result<Vec<u8>, Error> {
        let pool = self.pool.clone();
        let mut conn = pool.get().await.map_err(|e| {
            #[cfg(feature = "use-tracing")]
            tracing::error!("Unable to obtain redis connection: {e}");
            Error::RedisConn
        })?;
        conn.get::<_, Vec<u8>>(jti).await.map_err(|e| {
            #[cfg(feature = "use-tracing")]
            tracing::error!("Session record not found in redis: {e}");
            Error::NotFound
        })
    }

    async fn set_by_jti(
        self: Arc<Self>,
        jwt_jti: &[u8],
        refresh_jti: &[u8],
        bytes: &[u8],
        mut exp: Duration,
    ) -> Result<(), Error> {
        bad_ttl!(
            exp,
            Duration::seconds(1),
            "Expiration time is bellow 1s. This is not allowed for redis server."
        );
        let pool = self.pool.clone();
        let mut conn = pool.get().await.map_err(|e| {
            #[cfg(feature = "use-tracing")]
            tracing::error!("Unable to obtain redis connection: {e}");
            Error::RedisConn
        })?;
        let mut pipeline = redis::Pipeline::new();
        let _: () = pipeline
            .set_ex(jwt_jti, bytes, exp.as_seconds_f32() as u64)
            .set_ex(refresh_jti, bytes, exp.as_seconds_f32() as u64)
            .query_async(&mut conn)
            .await
            .map_err(|e| {
                #[cfg(feature = "use-tracing")]
                tracing::error!("Failed to save session in redis: {e}");
                Error::WriteFailed
            })?;
        Ok(())
    }

    async fn remove_by_jti(self: Arc<Self>, jti: &[u8]) -> Result<(), Error> {
        let pool = self.pool.clone();
        let mut conn = pool.get().await.map_err(|e| {
            #[cfg(feature = "use-tracing")]
            tracing::error!("Unable to obtain redis connection: {e}");
            Error::RedisConn
        })?;
        let _: () = conn.del(jti).await.map_err(|e| {
            #[cfg(feature = "use-tracing")]
            tracing::error!("Session record can't be removed from redis: {e}");
            Error::NotFound
        })?;
        Ok(())
    }
}

impl<ClaimsType: Claims> SessionMiddlewareBuilder<ClaimsType> {
    #[must_use]
    pub fn with_redis_pool(mut self, pool: Pool) -> Self {
        let storage = Arc::new(RedisStorage::<ClaimsType>::new(pool));
        let storage = SessionStorage::new(storage, self.jwt_encoding_key.clone(), self.algorithm);
        self.storage = Some(storage);
        self
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Add;

    use actix_web::cookie::time::*;

    use super::*;

    #[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq, Hash)]
    #[serde(rename_all = "snake_case")]
    pub struct Claims {
        #[serde(rename = "exp")]
        pub expires_at: usize,
        #[serde(rename = "iat")]
        pub issues_at: usize,
        /// Account login
        #[serde(rename = "sub")]
        pub subject: String,
        #[serde(rename = "aud")]
        pub audience: String,
        #[serde(rename = "jti")]
        pub jwt_id: uuid::Uuid,
        #[serde(rename = "aci")]
        pub account_id: i32,
    }

    impl crate::Claims for Claims {
        fn jti(&self) -> uuid::Uuid {
            self.jwt_id
        }

        fn subject(&self) -> &str {
            &self.subject
        }
    }

    async fn create_storage() -> (SessionStorage, SessionMiddlewareFactory<Claims>) {
        use deadpool_redis::{Config, Runtime};

        let redis = {
            let cfg = Config::from_url("redis://localhost:6379");
            let pool = cfg.create_pool(Some(Runtime::Tokio1)).unwrap();
            pool
        };
        let jwt_signing_keys = JwtSigningKeys::generate(false).unwrap();
        SessionMiddlewareFactory::<Claims>::build(
            Arc::new(jwt_signing_keys.encoding_key),
            Arc::new(jwt_signing_keys.decoding_key),
            Algorithm::EdDSA,
        )
        .with_redis_pool(redis)
        .with_extractors(
            Extractors::default()
                .with_refresh_cookie(REFRESH_COOKIE_NAME)
                .with_refresh_header(REFRESH_HEADER_NAME)
                .with_jwt_cookie(JWT_COOKIE_NAME)
                .with_jwt_header(JWT_HEADER_NAME),
        )
        .finish()
    }

    #[tokio::test]
    async fn check_encode() {
        let (store, _) = create_storage().await;
        let jwt_exp = JwtTtl(Duration::days(31));
        let refresh_exp = RefreshTtl(Duration::days(31));

        let original = Claims {
            subject: "me".into(),
            expires_at: OffsetDateTime::now_utc()
                .add(Duration::days(31))
                .unix_timestamp() as usize,
            issues_at: OffsetDateTime::now_utc().unix_timestamp() as usize,
            audience: "web".into(),
            jwt_id: Uuid::new_v4(),
            account_id: 24234,
        };

        store
            .store(original.clone(), jwt_exp, refresh_exp)
            .await
            .unwrap();
        let loaded = store.find_jwt(original.jwt_id).await.unwrap();
        assert_eq!(original, loaded);
    }
}
