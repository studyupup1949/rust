//! Common type definitions shared across the A9N ecosystem.
//!
//! This crate provides architecture-independent data types used by the
//! A9N Microkernel, user-level runtimes, system libraries, and applications.
//!
//! The definitions in this crate describe values that are exchanged across
//! kernel/user boundaries or shared by multiple A9N components, such as
//! capability descriptors, capability object types, message information,
//! rights, errors, fault reasons, and initialization data.
//!
//! This crate intentionally does not provide capability call implementations,
//! architecture-specific register operations, inline assembly, runtime
//! initialization, or high-level convenience APIs.
//!
//! In general:
//!
//! - object/value definitions belong to `a9n-types`
//! - call numbers, operation numbers, register layouts, and wire-level calling
//!   conventions belong to `a9n-abi`
//! - safe or ergonomic wrappers belong to higher-level crates
//!
//! Types that are part of the A9N ABI should use explicit representations such
//! as `#[repr(C)]`, `#[repr(usize)]`, or another fixed representation.
//! Their numeric values must be treated as stable once exposed across the
//! kernel/user boundary.

#![no_std]

mod frame_buffer_info;
mod init;
mod ipc_buffer;
mod message_info;
mod types;

pub use frame_buffer_info::*;
pub use init::*;
pub use ipc_buffer::*;
pub use message_info::*;
pub use types::*;
