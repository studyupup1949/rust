//! # ACPICA bindings
//! Incomplete rust bindings to Intel's ACPICA kernel subsystem. This crate is very much still under development - I am adding features as they are needed by [my OS project].
//! If you are using this crate, expect lots of compiler warnings and `todo!`s.
//!
//! ## Build Dependencies
//! This crate builds ACPICA from source, using the [`cc`] crate. This crate requires the presence of a C compiler on the system -
//! see that crate's documentation for more information.
//!
//! The crate also uses unstable rust features, so needs a nightly or beta compiler.
//!
//! ## Runtime Dependencies
//! As the crate is designed to be used in an OS kernel, it has minimal dependencies. The crate does require dynamic memory allocation, however.
//!
//! The crate also uses the [`log`] crate for logging.
//!
//! ## Usage
//! The ACPICA kernel subsystem calls into OS code using various functions prefixed with `AcpiOs`. These are translated by this library into calls to methods on the `AcpiHandler` trait. An object implementing this trait must be passed to `register_handler` before any ACPI functionality can be used. Initializing the library could look like this:
//!
//! ```rust, ignore
//! struct HandlerStruct {}
//!
//! impl AcpiHandler for HandlerStruct {
//!     // ...
//! }
//!
//! let handler = HandlerStruct {};
//!
//! let initialization = register_interface(handler)?;
//! let initialization = initialization.load_tables()?;
//! let initialization = initialization.enable_subsystem()?;
//! let initialization = initialization.initialize_objects()?;
//! ```
//!
//! [my OS project]: https://github.com/MarkRoss470/rust-os
//! [`cc`]: https://crates.io/crates/cc

#![no_std]
#![feature(c_variadic)]
#![feature(pointer_byte_offsets)]
// Safety best practises
#![warn(unsafe_op_in_unsafe_fn, clippy::undocumented_unsafe_blocks)]
#![deny(unsafe_op_in_unsafe_fn, clippy::undocumented_unsafe_blocks)]
#![deny(clippy::missing_safety_doc)]
// Public API best practises
#![deny(
    missing_debug_implementations,
    missing_docs,
    clippy::missing_safety_doc,
    clippy::missing_panics_doc
)]
// Correctness lints
#![warn(clippy::all, clippy::correctness)]
// FFI correctness
#![deny(improper_ctypes, improper_ctypes_definitions)]
// Pedantic lints
#![warn(clippy::pedantic)]
#![allow(clippy::missing_errors_doc, clippy::module_name_repetitions)]

extern crate alloc;

mod bindings;
mod interface;
#[cfg(test)]
pub(crate) mod testing;

pub use interface::*;

/// If no code in the crate calls ACPICA functions when compiling tests, cargo will not link the ACPICA library.
/// This means that some linker errors will go silently unnoticed until the crate is compiled outside of tests.
/// This test forces ACPICA to be linked to find these errors.
#[test]
#[ignore = "This just forces the compiler to actually link in ACPICA"]
#[allow(unreachable_code)]
fn force_link() {
    use crate::bindings::functions::AcpiDisable;

    panic!("This test should not be run,just compiled");
    // SAFETY: This code is never reached
    unsafe { AcpiDisable() };
}
