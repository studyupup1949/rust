//! This module represents backends for rate-limiter with some general public
//! tools for your own implementation.

use std::{error::Error, future::Future};

use chrono::{DateTime, Utc};

use crate::{limit::Limit, RequestId};

pub mod memory;

/// Representation of errors that the backend can return while validating the
/// request.
#[derive(Debug)]
pub enum BackendError {
    /// The incoming request is rate limited.
    RateLimited,
    /// The backend is unable to verify the user's request because of an
    /// internal error.
    VerificationError(Box<dyn Error>),
}

/// Reprensetation of a bucket - the container for each user request.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct Bucket {
    /// The amount of requests that user can make before they're be rate limited.
    pub remaining: i64,
    /// Representation of the datetime when this bucket was created.
    pub created_at: DateTime<Utc>,
}

/// General interface for all backend providers for rate limiter.
pub trait BackendProvider {
    /// Validates the incoming request and performs calculations on whether to
    /// allow or deny it. Also, updates the bucket associated with that ID.
    fn validate_request(
        &mut self,
        id: &RequestId,
        limit: Limit,
    ) -> impl Future<Output = Result<(), BackendError>> + Send;
}
