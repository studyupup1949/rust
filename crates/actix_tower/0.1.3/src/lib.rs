
//! Actix Tower — Bringing the Rust Web Ecosystem to Actix Web
//!
//! A collection of extensions that modernize the Actix Web ecosystem
//! while preserving its performance and stability.
//!
//! Actix Tower — Bridging the Rust Web Ecosystem
//!
//! A collection of extensions that modernize the Actix Web ecosystem by allowing
//! developers to tap directly into the broader `tower` and `tower-http` middleware 
//! ecosystem without sacrificing Actix Web's raw performance.
//!
//! # Core Features
//!
//! - **Tower Compatibility**: Safely wraps `Send + Sync` Tower middleware around `!Send` Actix workers.
//! - **Ergonomic Extractors**: `AutoJson`, `AutoQuery`, and `AutoPath` to drastically reduce boilerplate.
//! - **Production Middleware**: Built-in rate limiting, response caching, timeouts, and structured tracing.
//! - **Typed Responses**: A standardized `ApiResponse` envelope and `ApiError` utility.
//!
//! # Quick Start: The Ultimate Microservice
//!
//! This crate allows you to mix native Actix middleware with native Tower middleware seamlessly:
//!
//! ```no_run
//! use actix_web::{App, web, HttpResponse};
//! use actix_tower::prelude::*;
//! use tower_http::timeout::TimeoutLayer;
//! use tower_http::trace::TraceLayer;
//! use std::time::Duration;
//!
//! async fn handler(body: AutoJson<serde_json::Value>) -> actix_web::Result<HttpResponse> {
//!     Ok(HttpResponse::Ok().json(body.into_inner()))
//! }
//!
//! let app = App::new()
//!     // 1. Tower Trace (Logging)
//!     .wrap(TowerLayerCompat::new(TraceLayer::new_for_http()))
//!     // 2. Tower Timeout (Abort slow requests)
//!     .wrap(TowerLayerCompat::new(TimeoutLayer::new(Duration::from_secs(5))))
//!     // 3. Native Actix Tower Rate Limit (10 reqs/sec per IP)
//!     .wrap(RateLimit::new(10, Duration::from_secs(1)))
//!     .route("/", web::post().to(handler));
//! ```
//!
//! # Understanding the Concurrency Bridge
//!
//! Integrating Tower into Actix is difficult because Tower expects `Service::Future: Send`
//! (enabling tasks to move between threads), whereas Actix Web workers execute on a thread-local runtime (`!Send`).
//!
//! `actix_tower` bridges this gap safely using a custom `TowerMiddlewareService` wrapper.
//! By isolating the `Tower` layer inside a thread-local `Rc<RefCell<..>>` pool, it satisfies
//! Tower's `poll_ready` contracts without risking concurrency deadlocks on the Actix runtime.
//!
//! Furthermore, the bridge is aggressively optimized for **sub-nanosecond overhead**:
//! - **Zero Heap Allocations**: The hot path uses stack-allocated `pin_project!` state machines instead of `Box::pin`.
//! - **Static Dispatch**: Replaces `dyn Future` with concrete generic types, eliminating vtable lookups and enabling LLVM inlining.
//! - **Optimized Headers**: Uses zero-scan validation (`from_maybe_shared_unchecked`) when translating headers between Actix and Tower.

// `unsafe` is denied crate-wide. The single exception is
// `src/compat/tower/header_bridge.rs`, which uses `from_maybe_shared_unchecked`
// to skip redundant byte-scan validation when copying header values between
// `actix_web::http::header` and `http` crate types. That module carries a
// file-level `#![allow(unsafe_code)]`.
#![cfg_attr(docsrs, feature(doc_cfg))]
#![deny(unsafe_code)]
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
