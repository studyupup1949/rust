//! Compatibility layers for bridging Actix Web with the wider Rust ecosystem.

pub mod tower;

#[doc(hidden)]
pub mod hyper;

#[doc(hidden)]
pub mod http;
