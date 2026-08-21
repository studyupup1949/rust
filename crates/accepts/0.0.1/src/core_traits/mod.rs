//! Foundational acceptor traits exposed by the crate.
//!
//! These traits abstract over types that can *accept* values or asynchronous
//! computations.  They serve as the building blocks for adapters provided in
//! other modules and make it possible to write code that is generic over
//! different acceptor implementations.
mod implementations;

//sync
mod accepts;
pub use accepts::Accepts;

//async
mod async_accepts;
pub use async_accepts::AsyncAccepts;

//box
#[cfg(feature = "alloc")]
mod dyn_async_accepts;
#[cfg(feature = "alloc")]
pub use dyn_async_accepts::DynAsyncAccepts;

mod next_acceptors;

pub use next_acceptors::*;
