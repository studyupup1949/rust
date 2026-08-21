//! Core types and parser for addrezz.
//!
//! See the `addrezz` facade crate for the full public API.

#![cfg_attr(docsrs, feature(doc_cfg))]

mod addr;
mod error;
mod host;
mod macros;
mod options;
mod origin;
mod parser;
mod scheme;
mod userinfo;

#[cfg(any(feature = "url", feature = "http", feature = "reqwest"))]
mod convert;

#[cfg(feature = "resolve")]
mod resolve;

#[cfg(feature = "resolve_async")]
mod resolve_async;
#[cfg(feature = "resolve_async")]
pub use resolve_async::{CLOUDFLARE, GOOGLE, QUAD9, ResolverConfig};

#[cfg(feature = "arbitrary")]
mod arbitrary_impl;

#[cfg(feature = "proptest")]
mod proptest_impl;

#[cfg(feature = "sqlx")]
mod sqlx_impl;

#[cfg(feature = "ipnet")]
mod cidr;

#[cfg(feature = "psl")]
mod psl_impl;

#[cfg(feature = "whois")]
mod whois_impl;

#[cfg(feature = "whois")]
pub use whois_impl::{WhoisError, WhoisResponse};

pub use addr::Addr;
pub use error::{HostError, ParseError};
pub use host::Host;
pub use options::ParseOptions;
pub use origin::Origin;
pub use scheme::Scheme;
pub use userinfo::Userinfo;
