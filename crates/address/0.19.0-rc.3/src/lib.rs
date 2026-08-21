#![doc = include_str!("../README.md")]
#![allow(clippy::module_inception)]
#![warn(clippy::must_use_candidate)]
#![cfg_attr(docsrs, feature(doc_auto_cfg))]

pub use authority::*;
pub use domain::*;
pub use endpoint::*;
pub use host::*;
pub use ip::*;
pub use parse::*;
pub use socket::*;

mod authority;
mod domain;
mod endpoint;
mod host;
mod ip;
mod parse;
mod socket;

#[cfg(feature = "serde")]
mod serde;
