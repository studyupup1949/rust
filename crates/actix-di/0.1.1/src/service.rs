use std::any::{Any, TypeId};
use async_trait::async_trait;
use crate::error::ServiceError;

/// Trait for type conversions, similar to std::convert::As but for our use case
pub trait As<T: ?Sized> {
    /// Convert a reference of self to a reference of target type
    fn as_ref(&self) -> &T;
}

// Implement As<T> for T (reflexive case)
impl<T: ?Sized> As<T> for T {
    fn as_ref(&self) -> &T {
        self
    }
}

// Example implementations for concrete types
impl<T> As<dyn Any> for T
where
    T: Any,
{
    fn as_ref(&self) -> &dyn Any {
        self
    }
}


#[async_trait]
pub trait Service: Any + Send + Sync {
    async fn init(&self) -> Result<(), ServiceError> {
        Ok(())
    }

    async fn shutdown(&self) -> Result<(), ServiceError> {
        Ok(())
    }
}

pub trait DependencyProvider: Service {
    fn required_services() -> Vec<TypeId> where Self: Sized;
}