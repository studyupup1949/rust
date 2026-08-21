use std::future::Future;

pub type LimitType = u32;

pub trait RateLimitId: serde::de::DeserializeOwned + std::fmt::Display {}

impl<Id: serde::de::DeserializeOwned + std::fmt::Display> RateLimitId for Id {}

pub trait RateLimitBackend {
    type Error;

    type Future: Future<Output = Result<LimitType, Self::Error>>;

    fn touch(&self, id: &str, limit: LimitType) -> Self::Future;
}
