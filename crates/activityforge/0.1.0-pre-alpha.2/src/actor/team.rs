use activitystreams_vocabulary::{Collection, Iri, create_actor, create_item, field_access};

mod role_filter;

pub use role_filter::{FilterKey, RoleFilter};

create_actor! {
    /// Represents a group of people working together, collaborating on shared resources.
    ///
    /// Each member [Person](activitystreams_vocabulary::Person) in the team has a defined role,
    /// affecting the level of access they have to the team’s shared resources and to managing the team itself.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Team, context};
    /// use activitystreams_vocabulary::{
    ///     Collection, Iri, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyPublicKey, Name, Relationship,
    /// };
    /// # fn main() {
    /// let id = Iri::try_from("https://dev.example/teams/mobilizon-dev-team").unwrap();
    /// let name = Name::try_from("Mobilizon Development Team").unwrap();
    /// let context = Iri::try_from("https://dev.example/teams/framasoft-developers").unwrap();
    /// let summary = "We're creating a federated tool for organizing events!";
    /// let inbox = Iri::try_from("https://dev.example/teams/mobilizon-dev-team/inbox").unwrap();
    /// let outbox = Iri::try_from("https://dev.example/teams/mobilizon-dev-team/outbox").unwrap();
    /// let followers =
    ///     Iri::try_from("https://dev.example/teams/mobilizon-dev-team/followers").unwrap();
    ///
    /// let key_id = Iri::try_from("https://dev.example/aviva/treesim#main-key").unwrap();
    /// let controller = Iri::try_from("https://dev.example/aviva/treesim").unwrap();
    /// let encoded_multibase = "u7QGwDY2Tjn93PVFWWq02piP1NE9_XRlg-c8-jhJiHqKBHw";
    ///
    /// let member0_subject =
    ///     Iri::try_from("https://dev.example/teams/mobilizon-dev-team").unwrap();
    /// let member0_relationship = Iri::try_from("hasMember").unwrap();
    /// let member0_object = Iri::try_from("https://dev.example/people/alice").unwrap();
    /// let member0_tag = Iri::try_from("https://roles.example/admin").unwrap();
    ///
    /// let member1_subject =
    ///     Iri::try_from("https://dev.example/teams/mobilizon-dev-team").unwrap();
    /// let member1_relationship = Iri::try_from("hasMember").unwrap();
    /// let member1_object = Iri::try_from("https://dev.example/people/bob").unwrap();
    /// let member1_tag = Iri::try_from("maintain").unwrap();
    ///
    /// let member2_subject =
    ///     Iri::try_from("https://dev.example/teams/mobilizon-dev-team").unwrap();
    /// let member2_relationship = Iri::try_from("hasMember").unwrap();
    /// let member2_object = Iri::try_from("https://dev.example/people/celine").unwrap();
    /// let member2_tag = Iri::try_from("develop").unwrap();
    ///
    /// let subteam0 = Iri::try_from("https://dev.example/teams/mobilizon-backend-team").unwrap();
    /// let subteam1 = Iri::try_from("https://dev.example/teams/mobilizon-frontend-team").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Team",
    ///   "id": "{id}",
    ///   "name": "{name}",
    ///   "summary": "{summary}",
    ///   "context": "{context}",
    ///   "inbox": "{inbox}",
    ///   "outbox": "{outbox}",
    ///   "followers": "{followers}",
    ///   "assertionMethod": [
    ///     {{
    ///       "type": "Multikey",
    ///       "id": "{key_id}",
    ///       "controller": "{controller}",
    ///       "publicKeyMultibase": "{encoded_multibase}"
    ///     }}
    ///   ],
    ///   "members": {{
    ///     "type": "Collection",
    ///     "totalItems": 3,
    ///     "items": [
    ///       {{
    ///         "type": "Relationship",
    ///         "tag": "{member0_tag}",
    ///         "object": "{member0_object}",
    ///         "relationship": "{member0_relationship}",
    ///         "subject": "{member0_subject}"
    ///       }},
    ///       {{
    ///         "type": "Relationship",
    ///         "tag": "{member1_tag}",
    ///         "object": "{member1_object}",
    ///         "relationship": "{member1_relationship}",
    ///         "subject": "{member1_subject}"
    ///       }},
    ///       {{
    ///         "type": "Relationship",
    ///         "tag": "{member2_tag}",
    ///         "object": "{member2_object}",
    ///         "relationship": "{member2_relationship}",
    ///         "subject": "{member2_subject}"
    ///       }}
    ///     ]
    ///   }},
    ///   "subteams": {{
    ///     "type": "Collection",
    ///     "totalItems": 2,
    ///     "items": [
    ///       "{subteam0}",
    ///       "{subteam1}"
    ///     ]
    ///   }}
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let multibase = MultibasePublicKey::new()
    ///     .with_header(MultibaseHeader::Base64UrlNoPad)
    ///     .with_key(MultikeyPublicKey::Ed25519([
    ///         0xb0, 0x0d, 0x8d, 0x93, 0x8e, 0x7f, 0x77, 0x3d, 0x51, 0x56, 0x5a, 0xad, 0x36, 0xa6,
    ///         0x23, 0xf5, 0x34, 0x4f, 0x7f, 0x5d, 0x19, 0x60, 0xf9, 0xcf, 0x3e, 0x8e, 0x12, 0x62,
    ///         0x1e, 0xa2, 0x81, 0x1f,
    ///     ]));
    ///
    /// let multikey = Multikey::new_inner()
    ///     .with_id(key_id)
    ///     .with_controller(controller.clone())
    ///     .with_public_key_multibase(multibase);
    ///
    /// let member0 = Relationship::new_inner()
    ///     .with_subject(member0_subject)
    ///     .with_relationship(member0_relationship)
    ///     .with_object(member0_object)
    ///     .with_tag(member0_tag);
    ///
    /// let member1 = Relationship::new_inner()
    ///     .with_subject(member1_subject)
    ///     .with_relationship(member1_relationship)
    ///     .with_object(member1_object)
    ///     .with_tag(member1_tag);
    ///
    /// let member2 = Relationship::new_inner()
    ///     .with_subject(member2_subject)
    ///     .with_relationship(member2_relationship)
    ///     .with_object(member2_object)
    ///     .with_tag(member2_tag);
    ///
    /// let member_items = [member0, member1, member2];
    /// let members = Collection::new_inner()
    ///     .with_total_items(member_items.len() as u64)
    ///     .with_items(member_items);
    ///
    /// let subteam_items = [subteam0, subteam1];
    /// let subteams = Collection::new_inner()
    ///     .with_total_items(subteam_items.len() as u64)
    ///     .with_items(subteam_items);
    ///
    /// let team = Team::new()
    ///     .with_context_property(context_property)
    ///     .with_id(id)
    ///     .with_name(name)
    ///     .with_summary(summary)
    ///     .with_context(context)
    ///     .with_inbox(inbox)
    ///     .with_outbox(outbox)
    ///     .with_followers(followers)
    ///     .with_assertion_method([multikey])
    ///     .with_members(members)
    ///     .with_subteams(subteams);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&team).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Team>(json_str.as_str()).unwrap(),
    ///     team
    /// );
    /// # }
    /// ```
    Team: crate::ActorType::Team {
        #[serde(skip_serializing_if = "Option::is_none")]
        members: Option<Collection>,
        #[serde(skip_serializing_if = "Option::is_none")]
        subteams: Option<Collection>,
        #[serde(skip_serializing_if = "Option::is_none")]
        oversees: Option<Collection>,
        #[serde(skip_serializing_if = "Option::is_none")]
        overseen_by: Option<Collection>,
        #[serde(skip_serializing_if = "Option::is_none")]
        role_filter: Option<RoleFilter>,
    }
}

field_access! {
    Team {
        /// [Collection](activitystreams_vocabulary::Collection) of [Relationship](activitystreams_vocabulary::Relationship)s whose relationship is `hasMember`, and whose subject is this `Organization`.
        members: option_ref { Collection },
        /// Identifies a collection of the subteams of this [Team].
        ///
        /// Teams whose members inherit the access that this team’s members have to projects, and to project components (such as [Repository](crate::Repository)s).
        ///
        /// The collection items are Relationship objects whose relationship is hasChild and whose instrument is the maximal role allowed for delegation, specified when the child was added.
        subteams: option_ref { Collection },
        /// Represents the list of [Team]s this [Team] oversees.
        oversees: option_ref { Collection },
        /// Represents the list of [Team]s overseen by this [Team].
        overseen_by: option_ref { Collection },
        /// Represents the list of [RoleFilter] entries for [Team] members.
        ///
        /// Allows for fine-grained access control for individual [Team] members to shared resources.
        role_filter: option_ref { RoleFilter },
    }
}

create_item! {
    TeamItem
        boxed
        default: Self::Team(Box::default()),
    {
        Team(Team),
        Iri(Iri),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;
    use activitystreams_vocabulary::{
        MultibaseHeader, MultibasePublicKey, Multikey, MultikeyPublicKey, Name, Relationship,
    };

    #[test]
    fn test_valid() {
        let id = Iri::try_from("https://dev.example/teams/mobilizon-dev-team").unwrap();
        let name = Name::try_from("Mobilizon Development Team").unwrap();
        let context = Iri::try_from("https://dev.example/teams/framasoft-developers").unwrap();
        let summary = "We're creating a federated tool for organizing events!";
        let inbox = Iri::try_from("https://dev.example/teams/mobilizon-dev-team/inbox").unwrap();
        let outbox = Iri::try_from("https://dev.example/teams/mobilizon-dev-team/outbox").unwrap();
        let followers =
            Iri::try_from("https://dev.example/teams/mobilizon-dev-team/followers").unwrap();

        let key_id = Iri::try_from("https://dev.example/aviva/treesim#main-key").unwrap();
        let controller = Iri::try_from("https://dev.example/aviva/treesim").unwrap();
        let encoded_multibase = "u7QGwDY2Tjn93PVFWWq02piP1NE9_XRlg-c8-jhJiHqKBHw";

        let member0_subject =
            Iri::try_from("https://dev.example/teams/mobilizon-dev-team").unwrap();
        let member0_relationship = Iri::try_from("hasMember").unwrap();
        let member0_object = Iri::try_from("https://dev.example/people/alice").unwrap();
        let member0_tag = Iri::try_from("https://roles.example/admin").unwrap();

        let member1_subject =
            Iri::try_from("https://dev.example/teams/mobilizon-dev-team").unwrap();
        let member1_relationship = Iri::try_from("hasMember").unwrap();
        let member1_object = Iri::try_from("https://dev.example/people/bob").unwrap();
        let member1_tag = Iri::try_from("maintain").unwrap();

        let member2_subject =
            Iri::try_from("https://dev.example/teams/mobilizon-dev-team").unwrap();
        let member2_relationship = Iri::try_from("hasMember").unwrap();
        let member2_object = Iri::try_from("https://dev.example/people/celine").unwrap();
        let member2_tag = Iri::try_from("develop").unwrap();

        let subteam0 = Iri::try_from("https://dev.example/teams/mobilizon-backend-team").unwrap();
        let subteam1 = Iri::try_from("https://dev.example/teams/mobilizon-frontend-team").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Team",
  "id": "{id}",
  "name": "{name}",
  "summary": "{summary}",
  "context": "{context}",
  "inbox": "{inbox}",
  "outbox": "{outbox}",
  "followers": "{followers}",
  "assertionMethod": [
    {{
      "type": "Multikey",
      "id": "{key_id}",
      "controller": "{controller}",
      "publicKeyMultibase": "{encoded_multibase}"
    }}
  ],
  "members": {{
    "type": "Collection",
    "totalItems": 3,
    "items": [
      {{
        "type": "Relationship",
        "tag": "{member0_tag}",
        "object": "{member0_object}",
        "relationship": "{member0_relationship}",
        "subject": "{member0_subject}"
      }},
      {{
        "type": "Relationship",
        "tag": "{member1_tag}",
        "object": "{member1_object}",
        "relationship": "{member1_relationship}",
        "subject": "{member1_subject}"
      }},
      {{
        "type": "Relationship",
        "tag": "{member2_tag}",
        "object": "{member2_object}",
        "relationship": "{member2_relationship}",
        "subject": "{member2_subject}"
      }}
    ]
  }},
  "subteams": {{
    "type": "Collection",
    "totalItems": 2,
    "items": [
      "{subteam0}",
      "{subteam1}"
    ]
  }}
}}"#
        );

        let context_property = context::forgefed_context();

        let multibase = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base64UrlNoPad)
            .with_key(MultikeyPublicKey::Ed25519([
                0xb0, 0x0d, 0x8d, 0x93, 0x8e, 0x7f, 0x77, 0x3d, 0x51, 0x56, 0x5a, 0xad, 0x36, 0xa6,
                0x23, 0xf5, 0x34, 0x4f, 0x7f, 0x5d, 0x19, 0x60, 0xf9, 0xcf, 0x3e, 0x8e, 0x12, 0x62,
                0x1e, 0xa2, 0x81, 0x1f,
            ]));

        let multikey = Multikey::new_inner()
            .with_id(key_id)
            .with_controller(controller.clone())
            .with_public_key_multibase(multibase);

        let member0 = Relationship::new_inner()
            .with_subject(member0_subject)
            .with_relationship(member0_relationship)
            .with_object(member0_object)
            .with_tag(member0_tag);

        let member1 = Relationship::new_inner()
            .with_subject(member1_subject)
            .with_relationship(member1_relationship)
            .with_object(member1_object)
            .with_tag(member1_tag);

        let member2 = Relationship::new_inner()
            .with_subject(member2_subject)
            .with_relationship(member2_relationship)
            .with_object(member2_object)
            .with_tag(member2_tag);

        let member_items = [member0, member1, member2];
        let members = Collection::new_inner()
            .with_total_items(member_items.len() as u64)
            .with_items(member_items);

        let subteam_items = [subteam0, subteam1];
        let subteams = Collection::new_inner()
            .with_total_items(subteam_items.len() as u64)
            .with_items(subteam_items);

        let repository = Team::new()
            .with_context_property(context_property)
            .with_id(id)
            .with_name(name)
            .with_summary(summary)
            .with_context(context)
            .with_inbox(inbox)
            .with_outbox(outbox)
            .with_followers(followers)
            .with_assertion_method([multikey])
            .with_members(members)
            .with_subteams(subteams);

        assert_eq!(serde_json::to_string_pretty(&repository).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Team>(json_str.as_str()).unwrap(),
            repository
        );
    }
}
