//! Common middleware for Actix Web applications.
//!
//! These middleware are implemented natively for Actix (not via Tower)
//! to ensure maximum performance and compatibility.

pub mod auth;
pub mod cache;
pub mod compression;
pub mod metrics;
pub mod rate_limit;
pub mod request_id;
pub mod timeout;
pub mod tracing;

pub use auth::{AuthExtractor, AuthMiddleware, Authentication, Authorization};
pub use cache::Cache;
pub use compression::Compression;
pub use metrics::Metrics;
pub use rate_limit::{RateLimit, RateLimitConfig};
pub use request_id::RequestId;
pub use timeout::Timeout;
pub use tracing::{TracingConfig, TracingMiddleware};
