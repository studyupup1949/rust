use activitystreams_vocabulary::{LinkItems, create_actor, field_access};

use crate::ActorTypes;

create_actor! {
    /// A service actor that creates resource actors (such as `Repository`), exists to allow access control on creation of remote resources.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{ActorType, Factory, context};
    /// use activitystreams_vocabulary::{
    ///     Iri, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyPublicKey, Name,
    /// };
    ///
    /// # fn main() {
    /// let id = Iri::try_from("https://grape.fr33domlover.site/factories/YGQoZ").unwrap();
    /// let name = Name::try_from("final-factory").unwrap();
    /// let summary = ".";
    ///
    /// let ticket_tracker = ActorType::TicketTracker;
    /// let project = ActorType::Project;
    /// let team = ActorType::Team;
    /// let repository = ActorType::Repository;
    ///
    /// let controller = Iri::try_from("https://grape.fr33domlover.site/actor").unwrap();
    ///
    /// let key_id1 = Iri::try_from("https://grape.fr33domlover.site/actor#akey1").unwrap();
    /// let encoded_multibase1 = "u7QGwDY2Tjn93PVFWWq02piP1NE9_XRlg-c8-jhJiHqKBHw";
    ///
    /// let key_id2 = Iri::try_from("https://grape.fr33domlover.site/actor#akey1").unwrap();
    /// let encoded_multibase2 = "uhiQj9TRPf10ZYPnPPo4SYrANjZOOf3c9UVZarTamDqKBDxE";
    ///
    /// let collaborators = Iri::try_from("https://grape.fr33domlover.site/factories/YGQotiZ/collabs").unwrap();
    /// let followers = Iri::try_from("https://grape.fr33domlover.site/factories/YGQoZ/followers").unwrap();
    /// let inbox = Iri::try_from("https://grape.fr33domlover.site/factories/YGQoZ/inbox").unwrap();
    /// let outbox = Iri::try_from("https://grape.fr33domlover.site/factories/YGQoZ/outbox").unwrap();
    /// let teams = Iri::try_from("https://grape.fr33domlover.site/factories/YGQoZ/teams").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Factory",
    ///   "id": "{id}",
    ///   "name": "{name}",
    ///   "summary": "{summary}",
    ///   "inbox": "{inbox}",
    ///   "outbox": "{outbox}",
    ///   "followers": "{followers}",
    ///   "assertionMethod": [
    ///     {{
    ///       "type": "Multikey",
    ///       "id": "{key_id1}",
    ///       "controller": "{controller}",
    ///       "publicKeyMultibase": "{encoded_multibase1}"
    ///     }},
    ///     {{
    ///       "type": "Multikey",
    ///       "id": "{key_id2}",
    ///       "controller": "{controller}",
    ///       "publicKeyMultibase": "{encoded_multibase2}"
    ///     }}
    ///   ],
    ///   "availableActorTypes": [
    ///     "{ticket_tracker}",
    ///     "{project}",
    ///     "{team}",
    ///     "{repository}"
    ///   ],
    ///   "collaborators": "{collaborators}",
    ///   "teams": "{teams}"
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
    ///
    /// let multibase1 = MultibasePublicKey::new()
    ///     .with_header(MultibaseHeader::Base64UrlNoPad)
    ///     .with_key(MultikeyPublicKey::Ed25519([
    ///         0xb0, 0x0d, 0x8d, 0x93, 0x8e, 0x7f, 0x77, 0x3d, 0x51, 0x56, 0x5a, 0xad, 0x36, 0xa6,
    ///         0x23, 0xf5, 0x34, 0x4f, 0x7f, 0x5d, 0x19, 0x60, 0xf9, 0xcf, 0x3e, 0x8e, 0x12, 0x62,
    ///         0x1e, 0xa2, 0x81, 0x1f,
    ///     ]));
    ///
    /// let multibase2 = MultibasePublicKey::new()
    ///     .with_header(MultibaseHeader::Base64UrlNoPad)
    ///     .with_key(MultikeyPublicKey::Sm2([
    ///         0x23, 0xf5, 0x34, 0x4f, 0x7f, 0x5d, 0x19, 0x60, 0xf9, 0xcf, 0x3e, 0x8e, 0x12, 0x62,
    ///         0xb0, 0x0d, 0x8d, 0x93, 0x8e, 0x7f, 0x77, 0x3d, 0x51, 0x56, 0x5a, 0xad, 0x36, 0xa6,
    ///         0x0e, 0xa2, 0x81, 0x0f, 0x11,
    ///     ]));
    ///
    /// let multikey1 = Multikey::new_inner()
    ///     .with_id(key_id1)
    ///     .with_controller(controller.clone())
    ///     .with_public_key_multibase(multibase1);
    ///
    /// let multikey2 = Multikey::new_inner()
    ///     .with_id(key_id2)
    ///     .with_controller(controller.clone())
    ///     .with_public_key_multibase(multibase2);
    ///
    /// let factory = Factory::new()
    ///     .with_context_property(context)
    ///     .with_id(id)
    ///     .with_name(name)
    ///     .with_summary(summary)
    ///     .with_available_actor_types([ticket_tracker, project, team, repository])
    ///     .with_assertion_method([multikey1, multikey2])
    ///     .with_collaborators(collaborators)
    ///     .with_followers(followers)
    ///     .with_inbox(inbox)
    ///     .with_outbox(outbox)
    ///     .with_teams(teams);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&factory).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Factory>(json_str.as_str()).unwrap(),
    ///     factory
    /// );
    /// # }
    /// ```
    Factory: crate::ActorType::Factory {
        #[serde(skip_serializing_if = "Option::is_none")]
        available_actor_types: Option<ActorTypes>,
        #[serde(skip_serializing_if = "Option::is_none")]
        collaborators: Option<LinkItems>,
        #[serde(skip_serializing_if = "Option::is_none")]
        teams: Option<LinkItems>,
    }
}

field_access! {
    Factory {
        /// Specifies the actor types that can be created via this [Factory].
        available_actor_types: option_ref { ActorTypes },
        /// A collection of the direct collaborators of the actor, i.e. actors that have been given a direct-Grant to it.
        collaborators: option_ref { LinkItems },
        /// Identifies a collection of the resources this `Team` has access to, and that it is delegating to its members and subteams.
        teams: option_ref { LinkItems },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActorType, context};
    use activitystreams_vocabulary::{
        Iri, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyPublicKey, Name,
    };

    #[test]
    fn test_valid() {
        let id = Iri::try_from("https://grape.fr33domlover.site/factories/YGQoZ").unwrap();
        let name = Name::try_from("final-factory").unwrap();
        let summary = ".";

        let ticket_tracker = ActorType::TicketTracker;
        let project = ActorType::Project;
        let team = ActorType::Team;
        let repository = ActorType::Repository;

        let controller = Iri::try_from("https://grape.fr33domlover.site/actor").unwrap();

        let key_id1 = Iri::try_from("https://grape.fr33domlover.site/actor#akey1").unwrap();
        let encoded_multibase1 = "u7QGwDY2Tjn93PVFWWq02piP1NE9_XRlg-c8-jhJiHqKBHw";

        let key_id2 = Iri::try_from("https://grape.fr33domlover.site/actor#akey1").unwrap();
        let encoded_multibase2 = "uhiQj9TRPf10ZYPnPPo4SYrANjZOOf3c9UVZarTamDqKBDxE";

        let collaborators =
            Iri::try_from("https://grape.fr33domlover.site/factories/YGQoZ/collabs").unwrap();
        let followers =
            Iri::try_from("https://grape.fr33domlover.site/factories/YGQoZ/followers").unwrap();
        let inbox = Iri::try_from("https://grape.fr33domlover.site/factories/YGQoZ/inbox").unwrap();
        let outbox =
            Iri::try_from("https://grape.fr33domlover.site/factories/YGQoZ/outbox").unwrap();
        let teams = Iri::try_from("https://grape.fr33domlover.site/factories/YGQoZ/teams").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Factory",
  "id": "{id}",
  "name": "{name}",
  "summary": "{summary}",
  "inbox": "{inbox}",
  "outbox": "{outbox}",
  "followers": "{followers}",
  "assertionMethod": [
    {{
      "type": "Multikey",
      "id": "{key_id1}",
      "controller": "{controller}",
      "publicKeyMultibase": "{encoded_multibase1}"
    }},
    {{
      "type": "Multikey",
      "id": "{key_id2}",
      "controller": "{controller}",
      "publicKeyMultibase": "{encoded_multibase2}"
    }}
  ],
  "availableActorTypes": [
    "{ticket_tracker}",
    "{project}",
    "{team}",
    "{repository}"
  ],
  "collaborators": "{collaborators}",
  "teams": "{teams}"
}}"#
        );

        let context = context::forgefed_context();

        let multibase1 = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base64UrlNoPad)
            .with_key(MultikeyPublicKey::Ed25519([
                0xb0, 0x0d, 0x8d, 0x93, 0x8e, 0x7f, 0x77, 0x3d, 0x51, 0x56, 0x5a, 0xad, 0x36, 0xa6,
                0x23, 0xf5, 0x34, 0x4f, 0x7f, 0x5d, 0x19, 0x60, 0xf9, 0xcf, 0x3e, 0x8e, 0x12, 0x62,
                0x1e, 0xa2, 0x81, 0x1f,
            ]));

        let multibase2 = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base64UrlNoPad)
            .with_key(MultikeyPublicKey::Sm2([
                0x23, 0xf5, 0x34, 0x4f, 0x7f, 0x5d, 0x19, 0x60, 0xf9, 0xcf, 0x3e, 0x8e, 0x12, 0x62,
                0xb0, 0x0d, 0x8d, 0x93, 0x8e, 0x7f, 0x77, 0x3d, 0x51, 0x56, 0x5a, 0xad, 0x36, 0xa6,
                0x0e, 0xa2, 0x81, 0x0f, 0x11,
            ]));

        let multikey1 = Multikey::new_inner()
            .with_id(key_id1)
            .with_controller(controller.clone())
            .with_public_key_multibase(multibase1);

        let multikey2 = Multikey::new_inner()
            .with_id(key_id2)
            .with_controller(controller.clone())
            .with_public_key_multibase(multibase2);

        let factory = Factory::new()
            .with_context_property(context)
            .with_id(id)
            .with_name(name)
            .with_summary(summary)
            .with_available_actor_types([ticket_tracker, project, team, repository])
            .with_assertion_method([multikey1, multikey2])
            .with_collaborators(collaborators)
            .with_followers(followers)
            .with_inbox(inbox)
            .with_outbox(outbox)
            .with_teams(teams);

        assert_eq!(serde_json::to_string_pretty(&factory).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Factory>(json_str.as_str()).unwrap(),
            factory
        );
    }
}
