use std::{future::Future, pin::Pin};

use actix_redis::{Command, RespValue};

use super::types::*;
use super::util::*;

pub type RedisAddr = actix::Addr<actix_redis::RedisActor>;

pub struct RedisBackend {
    redis: RedisAddr,
    prefix: String,
}

static RATE_LIMIT_NUMKEYS: &str = "2";
static RATE_LIMIT_SCRIPT: &str = include_str!("redis_limiter.lua");

impl RedisBackend {
    pub fn new(addr: RedisAddr) -> Self {
        const KEY_PREFIX: &str = "RateLimit";

        RedisBackend {
            prefix: KEY_PREFIX.to_string(),
            redis: addr,
        }
    }

    pub fn set_prefix(&mut self, prefix: String) {
        self.prefix = prefix;
    }

    fn normalize_id(&self, id: &str) -> String {
        format!("{}:{}", self.prefix, id)
    }
}

macro_rules! command {
    ($e:expr) => {
        Command(
            RespValue::BulkString($e.to_string().into_bytes())
        )
    };

    ($($e:expr),+ $(,)?) => {
        Command(RespValue::Array(vec![
            $(RespValue::BulkString($e.to_string().into_bytes()),)+
        ]))
    };
}

impl RateLimitBackend for RedisBackend {
    type Error = ();

    type Future = Pin<Box<dyn Future<Output = Result<LimitType, Self::Error>>>>;

    fn touch(&self, id: &str, limit: LimitType) -> Self::Future {
        let redis = self.redis.clone();
        let command = command![
            "EVAL",
            RATE_LIMIT_SCRIPT,
            RATE_LIMIT_NUMKEYS,
            self.normalize_id(id),
            current_hour(),
            limit
        ];

        Box::pin(async move {
            fn fail<T: std::fmt::Debug>(it: T) {
                error!("{:?}", it);
            }

            redis
                .send(command)
                .await
                .map_err(fail)
                .and_then(|it| it.map_err(fail))
                .and_then(|value| match value {
                    RespValue::Integer(x) => Ok(x as LimitType),
                    others => {
                        fail(others);
                        Err(())
                    }
                })
        })
    }
}
