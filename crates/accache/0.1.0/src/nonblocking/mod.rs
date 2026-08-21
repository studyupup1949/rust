//! Async (`tokio`) variant of [`Accache`](crate::Accache).
//!
//! Mirrors the sync API one-for-one. Enabled with the `nonblocking` cargo feature.

mod accache;
pub use accache::Accache;
