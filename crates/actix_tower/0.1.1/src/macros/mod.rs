//! Declarative macros for reducing boilerplate.

pub mod extract;
pub mod handler;
pub mod route;

// Re-export macros at module level
pub use extract::extract;
pub use handler::handler;
pub use route::route;
