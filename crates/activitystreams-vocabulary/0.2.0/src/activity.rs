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
