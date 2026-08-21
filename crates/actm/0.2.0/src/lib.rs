//! # actm - A tiny actors framework
//!
//! `actm` provides a minimal, but quite usable, platform for implementing the [actor
//! pattern](https://en.wikipedia.org/wiki/Actor_model).
#![warn(
    clippy::all,
    clippy::pedantic,
    rust_2018_idioms,
    missing_docs,
    clippy::missing_docs_in_private_items
)]
#![allow(
    clippy::option_if_let_else,
    clippy::module_name_repetitions,
    clippy::shadow_unrelated,
    clippy::must_use_candidate,
    clippy::implicit_hasher
)]
pub mod executor;
#[cfg(test)]
pub(crate) mod testing_util;
pub mod traits;
pub mod types;
pub mod util;

/// Rexport `async_trait`, as this crate uses async traits
pub use async_trait;
/// Reexport `enum_dispatch`, this is critical for ergonomic use of this crate
pub use enum_dispatch;
/// Reexport `flume`, as this appears in the public api
pub use flume;

/// Commonly used types from this library
pub mod prelude {
    pub use async_trait::async_trait;
    pub use enum_dispatch::enum_dispatch;

    pub use crate::{
        async_actor,
        executor::Executor,
        sync_actor,
        traits::{Actor, Event, EventConsumer, EventProducer},
        types::{CompletionToken, Trigger, Waiter},
        util::{Adaptor, AsyncActor, AsyncActorError, SyncActor, SyncActorError, WrappedEvent},
        wrapped_event,
    };
}
