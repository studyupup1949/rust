//! # aam-rs
//!
//! Re-export facade. All functionality lives in `aam-core` +
//! `aam-derive`. Your `Cargo.toml` only needs `aam-rs`.
//!
//! ```no_run
//! use aam_rs::aaml::AAML;
//!
//! let cfg = AAML::load("config.aam").unwrap();
//! println!("{}", cfg.find_obj("host").unwrap());
//! ```

pub use aam_core::*;
pub use aam_derive::*;
