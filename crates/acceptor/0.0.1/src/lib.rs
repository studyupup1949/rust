#![cfg_attr(not(test), no_std)]
#![allow(
    clippy::manual_async_fn,
    clippy::module_inception,
    clippy::manual_is_multiple_of
)]

mod map;
pub use map::*;

mod filter;
pub use filter::*;

mod inspect;
pub use inspect::*;

mod branch;
pub use branch::*;

mod iterator;
pub use iterator::*;

mod repeat;
pub use repeat::*;

mod around;
pub use around::*;

mod batch;
pub use batch::*;

mod router;
pub use router::*;

mod result_router;
pub use result_router::*;

mod once;
pub use once::*;

mod deref_forwarder;
pub use deref_forwarder::*;

mod stateful_callback;
pub use stateful_callback::*;

#[cfg(test)]
pub(crate) mod support;
