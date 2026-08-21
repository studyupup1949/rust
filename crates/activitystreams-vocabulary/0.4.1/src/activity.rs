//! Collection of `Activity` vocabulary types.
//!
//! ## Creating a Custom Activity
//!
//! ```rust
//! use activitystreams_vocabulary::{create_activity, field_access};
//!
//! // Create a custom `Activity` type that inherits all of the base `Activity` + `Object` properties.
//! create_activity! {
//!     /// Externally created activity.
//!     ExternalActivity: external_vocab::ExternalType::TestActivity {
//!         custom_field1: Option<usize>,
//!         custom_field2: Option<u8>,
//!         string_field: Option<String>,
//!         vec_field: Option<Vec<u8>>,
//!    }
//! }
//!
//! // Field access definitions need to be grouped based on the access type, e.g. `option`, `option_deref`, etc.
//! field_access! {
//!     ExternalActivity<Vocab> {
//!         custom_field1: option { usize },
//!         custom_field2: option { u8 },
//!     }
//! }
//!
//! // `option_deref` uses the `Option::as_deref` function to get a reference to the `Deref` type, e.g. `Option<&str>` for `Option<String>`.
//! field_access! {
//!     ExternalActivity<Vocab> {
//!         string_field: option_deref { &str, String },
//!         vec_field: option_deref { &[u8], Vec<u8> },
//!     }
//! }
//!
//! # use activitystreams_vocabulary::Context;
//! # use external_vocab::ExternalType;
//! # fn main() {
//! let activity = ExternalActivity::<ExternalType>::new();
//!
//! // all Activity types have the following fields
//! //   (along with `set_`, `with_`, and `unset_` access functions)
//! assert_eq!(activity.context_property(), Some(&Context::new()));
//! assert_eq!(activity.kind(), &ExternalType::TestActivity);
//! assert!(activity.actor().is_none());
//! assert!(activity.object().is_none());
//! assert!(activity.origin().is_none());
//! assert!(activity.target().is_none());
//! assert!(activity.instrument().is_none());
//! assert!(activity.result().is_none());
//! # }
//! ```
//!
//! For details about the `external_vocab` crate, see the [top-level documentation](crate).
//!
//! `Activity` types also inherit all fields from the [Object](crate::object) type.

use crate::create_activity;

mod accept;
mod add;
mod announce;
mod arrive;
mod block;
mod create;
mod delete;
mod dislike;
mod flag;
mod follow;
mod ignore;
mod intransitive;
mod invite;
mod join;
mod leave;
mod like;
mod listen;
mod move_mod;
mod offer;
mod question;
mod read;
mod reject;
mod remove;
mod travel;
mod undo;
mod update;
mod view;

pub use accept::{Accept, TentativeAccept};
pub use add::Add;
pub use announce::Announce;
pub use arrive::Arrive;
pub use block::Block;
pub use create::Create;
pub use delete::Delete;
pub use dislike::Dislike;
pub use flag::Flag;
pub use follow::Follow;
pub use ignore::Ignore;
pub use intransitive::IntransitiveActivity;
pub use invite::Invite;
pub use join::Join;
pub use leave::Leave;
pub use like::Like;
pub use listen::Listen;
pub use move_mod::Move;
pub use offer::Offer;
pub use question::{Closed, Question};
pub use read::Read;
pub use reject::{Reject, TentativeReject};
pub use remove::Remove;
pub use travel::Travel;
pub use undo::Undo;
pub use update::Update;
pub use view::View;

create_activity! {
    /// Represents a activity of any kind.
    ///
    /// An [Activity] is a subtype of [Object](crate::Object) that describes some form of action that may happen, is currently happening, or has already happened.
    ///
    /// The [Activity] type itself serves as an abstract base type for all types of activities.
    ///
    /// It is important to note that the Activity type itself does not carry any specific semantics about the kind of action being taken.
    Activity: CoreType {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Note, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally did something to a note";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());
        let actor_kind = actor.kind();

        let object_name = Name::try_from("A Note").unwrap();
        let object = Note::new_inner().with_name(object_name.clone());
        let object_kind = object.kind();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Activity",
  "summary": "{summary}",
  "actor": {{
    "type": "{actor_kind}",
    "name": "{actor_name}"
  }},
  "object": {{
    "type": "{object_kind}",
    "name": "{object_name}"
  }}
}}"#
        );

        let document = Activity::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        assert_eq!(serde_json::to_string_pretty(&document).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Activity>(json_str.as_str()).unwrap(),
            document
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Activity>(json_str).is_err());
    }
}
