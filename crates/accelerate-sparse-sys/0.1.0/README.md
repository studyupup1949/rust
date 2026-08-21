# accelerate-sparse-sys

Raw FFI declarations for a C shim over the sparse direct solvers in the [Accelerate framework](https://developer.apple.com/documentation/accelerate/sparse_solvers) included with macOS.

> **Status:** This crate is pre-1.0 and its public API may change.

Most users want [`accelerate-sparse`](https://crates.io/crates/accelerate-sparse), the safe API built on top of this crate. Depend on `accelerate-sparse-sys` directly only to reach an unwrapped entry point; the safe crate re-exports it as `accelerate_sparse::sys`, so both share one shim version in a dependency tree.

## What this crate binds

Accelerate's `SparseFactor`, `SparseSolve`, `SparseRefactor`, and `SparseCleanup` entry points are Clang `__attribute__((overloadable))` functions, largely `static inline`, that pass structs by value. Nothing on the Rust side can call them directly. This crate ships a small C shim that resolves the overloads and re-exports plain `extern "C"` functions whose ABI is scalars and pointers only, together with a hand-written `extern "C"` block declaring that shim.

The declarations are written by hand: no `libclang`, no `bindgen`, and no Apple header is read from Rust. The shim's `_Static_assert`s check the declared constants against the SDK on every build.

## Requirements

- Rust 1.85 or newer.
- macOS with an Apple SDK and the Accelerate framework, for anything beyond an empty build.

The bindings exist only on macOS. On other targets this crate compiles to an empty library, so it can stay in a cross-platform dependency tree without a target condition.

## Trademarks

`accelerate-sparse` is an independent project and is not affiliated with, sponsored by, or endorsed by Apple Inc.

Apple and macOS are trademarks of Apple Inc., registered in the U.S. and other countries and regions. The Accelerate framework is included with macOS and is not redistributed by this project.

## License

Licensed under either of

- [Apache License, Version 2.0](LICENSE-APACHE), or
- [MIT License](LICENSE-MIT)

at your option.
