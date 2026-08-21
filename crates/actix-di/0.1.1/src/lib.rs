pub mod error;
pub mod registry;
pub mod service;
pub mod state;
pub mod macros;

pub use error::ServiceError;
pub use registry::ServiceRegistry;
pub use service::{Service, DependencyProvider};
pub use state::AppState;

// Re-export commonly used types
pub mod prelude {
    pub use crate::error::ServiceError;
    pub use crate::registry::ServiceRegistry;
    pub use crate::service::{Service, DependencyProvider};
    pub use crate::state::AppState;
    pub use crate::macros::*;
}