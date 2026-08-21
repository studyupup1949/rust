use super::redis_backend::*;
use super::types::*;

use super::limiter::RateLimit;
use super::limiter::RateLimitBuilder;

impl<Id: RateLimitId> RateLimit<Id, RedisBackend> {
    /// Create a Redis-backend Rate-Limiter from URL
    pub fn redis<T: Into<String>>(addr: T) -> RateLimitBuilder<Id, RedisBackend> {
        Self::redis_shared(actix_redis::RedisActor::start(addr))
    }

    /// Create a Redis-backend Rate-Limiter from the shared `RedisActor`
    pub fn redis_shared(addr: RedisAddr) -> RateLimitBuilder<Id, RedisBackend> {
        RateLimitBuilder::new(RedisBackend::new(addr))
    }
}

impl<Id: RateLimitId> RateLimitBuilder<Id, RedisBackend> {
    /// Set key prefix of the Redis record
    pub fn prefix<T: ToString>(mut self, prefix: T) -> Self {
        self.backend_mut().set_prefix(prefix.to_string());
        self
    }
}
