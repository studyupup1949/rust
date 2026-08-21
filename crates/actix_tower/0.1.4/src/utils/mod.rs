//! Utility functions, types, and traits.

pub mod builder;
pub mod error;
pub mod extensions;
pub mod response;
pub mod validation;

pub use builder::{AppBuilder, ServiceConfigBuilder};
pub use error::{ApiError, ApiErrorResponse, ErrorCode};
pub use extensions::RequestExt;
pub use response::{ApiResponse, TypedResponse};
pub use validation::{in_range, is_email, not_empty, Validator};
