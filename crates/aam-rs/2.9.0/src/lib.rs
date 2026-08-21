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
//!
//! The pipeline-backed [`AAM`] API is re-exported in the same way, including
//! its reload surface:
//!
//! ```no_run
//! use aam_rs::aam::AAM;
//!
//! // Load from disk — the source path is remembered so the config can be
//! // refreshed later without tracking it yourself.
//! let mut cfg = AAM::load("config.aam").expect("load");
//! // ...edit config.aam on disk...
//! cfg.update().expect("reload from disk");
//!
//! // Or replace the entire contents from raw text without touching disk:
//! cfg.update_from_text("host = localhost\nport = 8080\n")
//!     .expect("reparse from text");
//! ```
//!
//! [`AAM`]: aam_core::aam::AAM

pub use aam_core::*;
pub use aam_derive::*;
