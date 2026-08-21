//! # adminx
//!
//! A framework-neutral admin-panel framework. This crate is a **facade**: it
//! re-exports the neutral [`adminx_core`] plus whichever framework and storage
//! adapters you enable via Cargo features, so you depend on a single crate.
//!
//! ```toml
//! # Axum + PostgreSQL/MySQL/SQLite:
//! adminx = { version = "2", features = ["axum", "seaorm"] }
//! # Actix + MongoDB:
//! adminx = { version = "2", features = ["actix", "mongo"] }
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use adminx::prelude::*;              // Resource, ReqCtx, ApiResponse, auth, ...
//!
//! # #[derive(Clone)] struct UserResource;
//! # #[async_trait::async_trait] impl Resource for UserResource {
//! #   fn resource_name(&self)->&'static str {"Users"}
//! #   fn base_path(&self)->&'static str {"users"}
//! #   fn table_name(&self)->&'static str {"users"}
//! #   fn clone_box(&self)->Box<dyn Resource>{Box::new(self.clone())}
//! # }
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     adminx::seaorm::init("postgres://localhost/mydb").await?;   // storage adapter
//!     register_resource(Box::new(UserResource));
//!
//!     let app = axum::Router::new().nest("/adminx", adminx::axum::router());
//!     let l = tokio::net::TcpListener::bind("0.0.0.0:8080").await?;
//!     axum::serve(l, app).await?;
//!     Ok(())
//! }
//! ```
//!
//! The framework/storage integrations live under [`self::actix`] / [`self::axum`]
//! and [`self::seaorm`] / [`self::mongo`], each present only when its feature is
//! enabled. Everything else (the [`Resource`] trait, [`prelude`], auth, UI) comes
//! straight from the core.
#![cfg_attr(docsrs, feature(doc_cfg))]

// Pick exactly one web framework. Both adapters register the same routes, so
// enabling both is almost always a mistake in Cargo features. (Both *storage*
// backends together is fine — the `cli` feature bundles them on purpose.)
#[cfg(all(feature = "actix", feature = "axum"))]
compile_error!(
    "adminx: enable only one web framework — pick feature `actix` OR `axum`, not both."
);

// Re-export the entire neutral core at the crate root, including its `prelude`,
// `auth`, `ui`, `storage`, etc. modules and the `Resource` trait.
pub use adminx_core::*;

/// The neutral core, also reachable explicitly as `adminx::core`.
pub use adminx_core as core;

/// Actix Web integration (`adminx::actix::scope()`). Requires feature `actix`.
#[cfg(feature = "actix")]
#[cfg_attr(docsrs, doc(cfg(feature = "actix")))]
pub use adminx_actix as actix;

/// Axum integration (`adminx::axum::router()`). Requires feature `axum`.
#[cfg(feature = "axum")]
#[cfg_attr(docsrs, doc(cfg(feature = "axum")))]
pub use adminx_axum as axum;

/// SeaORM storage (`adminx::seaorm::init(url)`). Requires feature `seaorm`.
#[cfg(feature = "seaorm")]
#[cfg_attr(docsrs, doc(cfg(feature = "seaorm")))]
pub use adminx_seaorm as seaorm;

/// MongoDB storage (`adminx::mongo::init(uri, db)`). Requires feature `mongo`.
#[cfg(feature = "mongo")]
#[cfg_attr(docsrs, doc(cfg(feature = "mongo")))]
pub use adminx_mongo as mongo;

/// DB-backed RBAC (`adminx::rbac::init(abilities)`). Requires feature `rbac`.
/// The `Action`, `Authorizer`, and `set_authorizer` items come through the core
/// re-export above; this adds the `DbAuthorizer` backend and `Ability` builder.
#[cfg(feature = "rbac")]
#[cfg_attr(docsrs, doc(cfg(feature = "rbac")))]
pub use adminx_rbac as rbac;
