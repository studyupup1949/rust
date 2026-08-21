use super::limiter::RateLimit;
use super::redis_backend::*;
use super::types::*;

impl<Id: RateLimitId> RateLimit<Id, RedisBackend> {
    /// Create a Redis-backend Rate-Limiter from the shared `RedisActor`
    pub fn redis_shared(addr: RedisAddr) -> Self {
        RateLimit::new(RedisBackend::new(addr))
    }

    /// Create a Redis-backend Rate-Limiter from URL
    pub fn redis<T: ToString>(addr: T) -> Self {
        Self::redis_shared(actix_redis::RedisActor::start(addr.to_string()))
    }

    /// Set key prefix of the Redis record
    pub fn prefix<T: ToString>(mut self, prefix: T) -> Self {
        self.backend_mut().set_prefix(prefix.to_string());
        self
    }
}
