
//! Actix Tower — Bringing the Rust Web Ecosystem to Actix Web
//!
//! A collection of extensions that modernize the Actix Web ecosystem
//! while preserving its performance and stability.
//!
//! # Quick Start
//!
//! ```no_run
//! use actix_tower::prelude::*;
//! use actix_web::App;
//!
//! let app = App::new()
//!     .wrap(RequestId::new())
//!     .wrap(Timeout::new(std::time::Duration::from_secs(30)))
//!     .wrap(tower_layer!(tower_http::compression::CompressionLayer::new()));
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![allow(clippy::type_complexity)]

pub mod prelude;

#[cfg(feature = "tower")]
pub mod compat;

#[cfg(feature = "extract")]
pub mod extract;

#[cfg(feature = "middleware")]
pub mod middleware;

#[cfg(feature = "macros")]
pub mod macros;

#[cfg(feature = "utils")]
pub mod utils;

pub mod internal;

// ---------------------------------------------------------------------------
// Re-exports
// ---------------------------------------------------------------------------

/// Re-export of actix_web for convenience.
pub use actix_web;

/// Re-export of tower for convenience.
pub use tower;

/// Re-export of http for convenience.
pub use http;

/// A type-erased, `Send + Sync` error pointer — the canonical Tower error type.
///
/// This is the same type used throughout `tower`, `tower-http`, `axum`, and
/// `hyper`. Custom Tower middleware should use this as `Service::Error`.
pub use crate::internal::common::BoxError;

/// The toolkit version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// The toolkit crate name.
pub const NAME: &str = env!("CARGO_PKG_NAME");
