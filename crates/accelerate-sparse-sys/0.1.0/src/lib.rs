//! This crate provides raw FFI declarations for a C shim over the sparse solvers in the Accelerate
//! framework included with macOS.
//!
//! The `SparseFactor`, `SparseSolve`, `SparseRefactor` and `SparseCleanup` entry points exposed by
//! Accelerate are Clang `__attribute__((overloadable))` functions, largely `static inline`, that
//! pass structs by value. Nothing on the Rust side can call them directly. A small C shim,
//! compiled by Clang, resolves the overloads and re-exports plain `extern "C"` functions whose ABI
//! is scalars and pointers only.
//!
//! The declarations below describe *that shim*, not Accelerate. They are written by hand, require
//! no `libclang`, and can be documented without a macOS SDK; no Apple header is read from Rust.
//! Shim `_Static_assert`s check the constants against the SDK on every build.
//!
//! Most users want [`accelerate-sparse`][1] instead; this crate is the unwrapped layer beneath it.
//!
//! [1]: https://docs.rs/accelerate-sparse
//!
//! # Trademarks
//!
//! `accelerate-sparse` is an independent project and is not affiliated with, sponsored by, or
//! endorsed by Apple Inc.
//!
//! Apple and macOS are trademarks of Apple Inc., registered in the U.S. and other countries and
//! regions. The Accelerate framework is included with macOS and is not redistributed by this
//! project.
//!
//! # Platform
//!
//! macOS only in substance. On other targets this compiles to an empty library, allowing it to
//! remain in cross-platform dependency trees.
//!
//! # Safety
//!
//! Every function here is `unsafe` and none of the invariants the shim relies on are checked at
//! this layer. In particular the pattern arrays must describe a structure Accelerate can traverse
//! without leaving them, and `values` must be as long as the pattern's non-zero count. Violating
//! either is an out-of-bounds read inside a C framework.

#![warn(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]
#![cfg_attr(docsrs, feature(doc_cfg))]

#[cfg(any(target_os = "macos", docsrs))]
mod ffi;

#[cfg(any(target_os = "macos", docsrs))]
pub use ffi::*;

/// Whether this build's shim was compiled against an SDK providing `SparseGetInertia`.
///
/// If false, [`accsp_get_inertia_d`] and [`accsp_get_inertia_f`] report
/// [`ACCSP_STATUS_UNSUPPORTED_OS`] for a factored handle. An unfactored handle reports
/// [`ACCSP_STATUS_NOT_FACTORED`] first. If true, calls still require macOS 13.0 or newer and an
/// [`ACCSP_KIND_LDLT_TPP`] factorization.
///
/// Exposes a build-script decision that cannot propagate to dependent crates as a `cfg`.
///
/// Published documentation renders its own build's value. That build is not a macOS host and
/// reports false, so read the constant from your build.
#[cfg(any(target_os = "macos", docsrs))]
pub const HAS_INERTIA: bool = cfg!(accsp_have_inertia);
