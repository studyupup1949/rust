#![cfg_attr(not(feature = "std"), no_std)]

//! Building blocks for composing synchronous and asynchronous "acceptor" pipelines.
//!
//! The `accepts` crate offers a small collection of traits and utilities for types that
//! consume values and optionally forward them to other acceptors.  The traits can be used
//! directly, or combined with the code-generation and utility modules to build complex
//! processing graphs.  Most modules are feature-gated so the crate remains lightweight in
//! `no_std` environments.
//!
//! * [`core_traits`] exposes the foundational trait definitions together with blanket
//!   implementations for common standard-library types.
//! * [`common`] provides reusable data structures shared by multiple utilities.
//! * [`utils`] contains ready-made acceptor implementations that can be composed with the
//!   core traits.
//! * [`ext`] exposes extension traits for ergonomics when working with iterators and
//!   asynchronous executors.
//! * [`macros`] and [`codegen`] enable generating boilerplate-heavy acceptor implementations
//!   when the optional `macros` or `codegen` features are enabled.

#[doc(hidden)]
pub mod __internal;

/// Foundational traits and blanket implementations for acceptor abstractions.
pub mod core_traits;

/// Shared data structures and helpers used across optional utilities.
#[cfg(feature = "__common_flag")]
pub mod common;

/// Ready-to-use acceptor implementations for core and `std` environments.
#[cfg(feature = "utils")]
pub mod utils;

/// Extension traits that make it easier to wire acceptors into existing code.
#[cfg(feature = "__ext_flag")]
pub mod ext;

/// Public procedural macros for generating acceptor implementations.
#[cfg(feature = "macros")]
pub mod macros;

#[cfg(all(not(feature = "macros"), feature = "__internal_macros_flag"))]
pub(crate) mod macros;

/// Code generation helpers used by the procedural macros.
#[cfg(feature = "codegen")]
pub mod codegen;

#[cfg(all(not(feature = "codegen"), feature = "__internal_macros_flag"))]
pub(crate) mod codegen;
