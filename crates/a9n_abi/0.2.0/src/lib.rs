//! ABI definitions for the A9N Microkernel user/kernel interface.
//!
//! This crate defines the low-level calling conventions used to communicate with the A9N
//! Microkernel from user space.
//!
//! It contains ABI-level constants, operation numbers, message register
//! layouts, kernel call identifiers, debug call identifiers, and capability
//! operation definitions. These items describe how values from `a9n-types` are
//! encoded into kernel calls and capability calls.
//!
//! Architecture-specific trap invocation and register manipulation may be
//! implemented in this crate or in architecture-specific modules owned by this
//! crate. Higher-level, Rust-friendly wrappers should be implemented in a
//! separate crate so that this crate remains a precise description of the ABI
//! boundary.
//!
//! In general:
//!
//! - shared object/value definitions belong to `a9n-types`
//! - operation numbers and call layouts belong to `a9n-abi`
//! - safe wrappers and ergonomic APIs belong to higher-level crates
//!
//! The definitions in this crate are part of the stable contract between
//! user-level components and the kernel. Any change to numeric values,
//! register assignments, bit layouts, or message formats should be treated as
//! an ABI change.

#![no_std]

pub mod arch;
pub mod capability_call;

pub use a9n_types::*;
