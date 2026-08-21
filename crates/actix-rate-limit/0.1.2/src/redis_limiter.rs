use super::redis_backend::*;
use super::types::*;

use super::RateLimit;

pub trait ToRedisAddr {
    fn to_redis_addr(self) -> RedisAddr;
}

impl ToRedisAddr for RedisAddr {
    fn to_redis_addr(self) -> RedisAddr {
        self
    }
}

impl ToRedisAddr for &str {
    fn to_redis_addr(self) -> RedisAddr {
        actix_redis::RedisActor::start(self)
    }
}

impl ToRedisAddr for String {
    fn to_redis_addr(self) -> RedisAddr {
        actix_redis::RedisActor::start(self)
    }
}

impl<Id: RateLimitId> RateLimit<Id, RedisBackend> {
    pub fn redis<T: ToRedisAddr>(redis: T) -> Self {
        let addr = redis.to_redis_addr();
        RateLimit::new(RedisBackend::new(addr))
    }

    pub fn prefix<T: ToString>(mut self, prefix: T) -> Self {
        self.backend_mut().set_prefix(prefix.to_string());
        self
    }
}
