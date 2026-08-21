#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(feature = "macros")]
pub mod macros {
    pub use actix_cloud_extra_macros::*;
}
#[cfg(feature = "api")]
pub mod api;
#[cfg(feature = "entity")]
pub mod entity;
#[cfg(feature = "hyuuid")]
pub mod hyuuid;
#[cfg(feature = "logger")]
pub mod logger;
#[cfg(feature = "utils")]
pub mod utils;

#[cfg(feature = "hyuuid")]
pub use hyuuid::HyUuid;
