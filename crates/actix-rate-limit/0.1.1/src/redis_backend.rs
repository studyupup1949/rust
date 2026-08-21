use std::{future::Future, pin::Pin, rc::Rc};

use actix_redis::{Command, RespValue};

use super::types::*;
use super::util::*;

pub struct RedisBackend {
    redis: Rc<RedisAddr>,
    prefix: String,
}

static RATE_LIMIT_NUMKEYS: &str = "2";
static RATE_LIMIT_SCRIPT: &str = include_str!("redis_limit.lua");

impl RedisBackend {
    pub fn new(redis: RedisAddr, prefix: &str) -> Self {
        RedisBackend {
            redis: Rc::new(redis),
            prefix: prefix.to_string(),
        }
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
