pub mod json_store;
pub mod middleware;
pub mod store;

pub use json_store::JsonUserStore;
pub use middleware::RequireAuth;
pub use store::{hash_password, verify_password, AuthError, User, UserStore};

/// Helper to generate a unique user ID (nanosecond timestamp as hex).
pub use store::generate_id;

/// Helper to get current timestamp in ISO 8601 format.
pub use store::timestamp;
