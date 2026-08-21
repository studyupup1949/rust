//! Collection of ActivityPub `Actor` types.
//!
//! ## Creating a Custom Actor
//!
//! ```rust
//! use activitystreams_vocabulary::{create_actor, field_access};
//!
//! // Create a custom `Actor` type that inherits all of the base `Actor` + `Object` properties.
//! create_actor! {
//!     /// Externally created actor.
//!     ExternalActor: external_vocab::ExternalType::TestActor {
//!         custom_field1: Option<usize>,
//!         custom_field2: Option<u8>,
//!         string_field: Option<String>,
//!         vec_field: Option<Vec<u8>>,
//!    }
//! }
//!
//! // Field access definitions need to be grouped based on the access type, e.g. `option`, `option_deref`, etc.
//! field_access! {
//!     ExternalActor<Vocab> {
//!         custom_field1: option { usize },
//!         custom_field2: option { u8 },
//!     }
//! }
//!
//! // `option_deref` uses the `Option::as_deref` function to get a reference to the `Deref` type, e.g. `Option<&str>` for `Option<String>`.
//! field_access! {
//!     ExternalActor<Vocab> {
//!         string_field: option_deref { &str, String },
//!         vec_field: option_deref { &[u8], Vec<u8> },
//!     }
//! }
//!
//! # use activitystreams_vocabulary::Context;
//! # use external_vocab::ExternalType;
//! # fn main() {
//! let actor = ExternalActor::<ExternalType>::new();
//!
//! // all Actor types have the following fields
//! //   (along with `set_`, `with_`, and `unset_` access functions)
//! assert_eq!(actor.context_property(), Some(&Context::new()));
//! assert_eq!(actor.kind(), &ExternalType::TestActor);
//! assert!(actor.inbox().is_none());
//! assert!(actor.outbox().is_none());
//! assert!(actor.following().is_none());
//! assert!(actor.followers().is_none());
//! assert!(actor.liked().is_none());
//! assert!(actor.streams().is_none());
//! assert!(actor.preferred_username().is_none());
//! assert!(actor.endpoints().is_none());
//! assert!(actor.assertion_method().is_none());
//! assert!(actor.public_key().is_none());
//! # }
//! ```
//!
//! For details about the `external_vocab` crate, see the [top-level documentation](crate).
//!
//! `Actor` types also inherit all fields from the [Object](crate::object) type.

mod application;
mod endpoints;
mod group;
mod organization;
mod person;
mod service;

pub use application::*;
pub use endpoints::*;
pub use group::*;
pub use organization::*;
pub use person::*;
pub use service::*;
