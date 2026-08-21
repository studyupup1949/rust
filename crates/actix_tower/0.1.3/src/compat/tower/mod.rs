//! Tower middleware compatibility for Actix Web.
//!
//! This module provides the bridge between the Tower ecosystem and Actix Web.
//! It allows you to use any Tower `Layer` as an Actix `Transform`, enabling
//! middleware like `tower-http`, `tower-governor`, etc.
//!
//! # Example
//!
//! ```no_run
//! use actix_tower::prelude::*;
//! use actix_web::App;
//!
//! let app = App::new()
//!     .wrap(tower_layer!(
//!         tower_http::trace::TraceLayer::new_for_http()
//!     ));
//! ```

pub mod body;
pub mod future;
pub(crate) mod future_impl;
pub(crate) mod header_bridge;
pub mod layer;
pub mod request;
pub mod response;
pub mod service;
pub mod transform;

pub use body::{
    collect_body, collect_body_limited, ActixRequestBody, ActixResponseBody, TowerBodyStream,
};
pub use layer::{TowerLayer, TowerLayerCompat};
pub use request::DEFAULT_MAX_BODY_BYTES;
pub use service::{ActixServiceWrapper, TowerMiddlewareService};
pub use transform::{LayerAsTransform, TransformAsLayer};

/// Apply a Tower [`Layer`](tower_layer::Layer) as an Actix [`Transform`](actix_service::Transform).
///
/// # Example
///
/// ```no_run
/// use actix_tower::prelude::*;
///
/// App::new().wrap(apply_tower(
///     tower_http::trace::TraceLayer::new_for_http()
/// ));
/// ```
pub fn apply_tower<L>(layer: L) -> TowerLayer<L> {
    TowerLayer::new(layer)
}

/// Macro to ergonomically wrap a Tower layer for use with Actix.
///
/// ```no_run
/// use actix_tower::prelude::*;
/// use actix_web::App;
///
/// let app = App::new()
///     .wrap(tower_layer!(tower_http::trace::TraceLayer::new_for_http()));
/// ```
#[macro_export]
macro_rules! tower_layer {
    ($layer:expr) => {
        $crate::compat::tower::apply_tower($layer)
    };
}
