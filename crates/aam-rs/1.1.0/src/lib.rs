//! # aam-rs
//!
//! A lightweight AAML (AAM Markup Language) parser and validator.
//!
//! ## Features
//! - Simple `key = value` configuration syntax with comment support (`#`)
//! - Directive system: `@import`, `@derive`, `@schema`, `@type`
//! - Schema-based type validation — fields are checked automatically during parsing
//! - Built-in types: `i32`, `f64`, `string`, `bool`, `color`,
//!   `math::vector2/3/4`, `physics::kilogram`, `time::datetime`, and more
//! - Custom type aliases via `@type`
//! - Inheritance via `@derive` with child-wins-on-conflict semantics
//!
//! ## Quick start
//! ```no_run
//! use aam_rs::aaml::AAML;
//!
//! let cfg = AAML::load("config.aam").unwrap();
//! println!("{}", cfg.find_obj("host").unwrap());
//! ```

pub mod aaml;
pub mod found_value;
pub mod error;
pub mod builder;
pub mod commands;
mod types;