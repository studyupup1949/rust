//! `actix-rate-limiter` is a simple yet powerful per-route rate limiter for
//! [Actix](https://docs.rs/actix-web/latest/actix_web/) with support for regex.
//!
//! ### Available backends
//!
//! Right now, only in-memory storage is supported officially. But you can
//! create your own backend using the `BackendProvider` trait. You can use
//! `MemoryBackendProvider` as an example implementation.
//!
//! We plan to add support for some other backends in the future, such as Redis.
//! If you want to help with their development, please checkout our
//! [GitHub](https://github.com/Pelfox/actix-rate-limiter).
//!
//! ### Examples
//!
//! Check the examples folder of our repository to see the available code samples.
//!

#![deny(
    missing_docs,
    missing_debug_implementations,
    missing_copy_implementations,
    unsafe_code
)]

pub mod backend;
pub mod middleware;

pub mod limit;
pub mod limiter;
pub mod route;

/// General type for tne ID of the request. It consists of the requester's
/// identifier and the request's path. Guaranteed format: `{id}:{path}`.
pub type RequestId = String;
