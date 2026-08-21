use activitystreams_vocabulary::{Iri, create_actor, create_item};

create_actor! {
    /// Represents a tracker of merge requests, i.e. a project managing a list of patches or branches submitted as proposed changes to a given [Repository](crate::Repository).
    ///
    /// # Example (single vocabulary type)
    ///
    /// ```rust
    /// use activityforge::{PatchTracker, context};
    /// use activitystreams_vocabulary::{
    ///     Iri, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyPublicKey, Name,
    /// };
    ///
    /// # fn main() {
    /// let id = Iri::try_from("https://dev.example/aviva/treesim").unwrap();
    /// let name = Name::try_from("Tree Growth 3D Simulation").unwrap();
    /// let summary = "<p>Tree growth 3D simulator for my nature exploration game</p>";
    /// let inbox = Iri::try_from("https://dev.example/aviva/treesim/inbox").unwrap();
    /// let outbox = Iri::try_from("https://dev.example/aviva/treesim/outbox").unwrap();
    /// let followers = Iri::try_from("https://dev.example/aviva/treesim/followers").unwrap();
    ///
    /// let key_id = Iri::try_from("https://dev.example/aviva/treesim#main-key").unwrap();
    /// let controller = Iri::try_from("https://dev.example/aviva/treesim").unwrap();
    /// let encoded_multibase = "u7QGwDY2Tjn93PVFWWq02piP1NE9_XRlg-c8-jhJiHqKBHw";
    ///
    /// let team = Iri::try_from("https://dev.example/aviva/treesim/team").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "PatchTracker",
    ///   "id": "{id}",
    ///   "name": "{name}",
    ///   "summary": "{summary}",
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
    ///   ]
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
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
    /// let repository = PatchTracker::new()
    ///     .with_context_property(context)
    ///     .with_id(id)
    ///     .with_name(name)
    ///     .with_summary(summary)
    ///     .with_inbox(inbox)
    ///     .with_outbox(outbox)
    ///     .with_followers(followers)
    ///     .with_assertion_method([multikey]);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&repository).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<PatchTracker>(json_str.as_str()).unwrap(),
    ///     repository
    /// );
    /// # }
    /// ```
    ///
    /// # Example (list of vocabulary types)
    ///
    /// ```rust
    /// use activityforge::{ActorType, PatchTracker, VocabularyTypes, context};
    /// use activitystreams_vocabulary::{
    ///     Iri, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyPublicKey, Name,
    /// };
    ///
    /// # fn main() {
    /// let id = Iri::try_from("https://dev.example/aviva/treesim").unwrap();
    /// let name = Name::try_from("Tree Growth 3D Simulation").unwrap();
    /// let summary = "<p>Tree growth 3D simulator for my nature exploration game</p>";
    /// let inbox = Iri::try_from("https://dev.example/aviva/treesim/inbox").unwrap();
    /// let outbox = Iri::try_from("https://dev.example/aviva/treesim/outbox").unwrap();
    /// let followers = Iri::try_from("https://dev.example/aviva/treesim/followers").unwrap();
    ///
    /// let key_id = Iri::try_from("https://dev.example/aviva/treesim#main-key").unwrap();
    /// let controller = Iri::try_from("https://dev.example/aviva/treesim").unwrap();
    /// let encoded_multibase = "u7QGwDY2Tjn93PVFWWq02piP1NE9_XRlg-c8-jhJiHqKBHw";
    ///
    /// let team = Iri::try_from("https://dev.example/aviva/treesim/team").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": [
    ///     "Repository",
    ///     "TicketTracker",
    ///     "PatchTracker"
    ///   ],
    ///   "id": "{id}",
    ///   "name": "{name}",
    ///   "summary": "{summary}",
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
    ///   ]
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
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
    /// let repository = PatchTracker::new()
    ///     .with_context_property(context)
    ///     .with_kind(VocabularyTypes::list([
    ///         ActorType::Repository,
    ///         ActorType::TicketTracker,
    ///         ActorType::PatchTracker,
    ///     ]))
    ///     .with_id(id)
    ///     .with_name(name)
    ///     .with_summary(summary)
    ///     .with_inbox(inbox)
    ///     .with_outbox(outbox)
    ///     .with_followers(followers)
    ///     .with_assertion_method([multikey]);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&repository).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<PatchTracker>(json_str.as_str()).unwrap(),
    ///     repository
    /// );
    /// # }
    /// ```
    PatchTracker: crate::ActorType::PatchTracker {}
}

create_item! {
    /// Represents a field range that can either be a [PatchTracker] or [Iri].
    PatchTrackerItem
        boxed
        default: Self::PatchTracker(Box::default()),
    {
        PatchTracker(PatchTracker),
        Iri(Iri),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ActorType, VocabularyTypes, context};
    use activitystreams_vocabulary::{
        Iri, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyPublicKey, Name,
    };

    #[test]
    fn test_valid() {
        let id = Iri::try_from("https://dev.example/aviva/treesim").unwrap();
        let name = Name::try_from("Tree Growth 3D Simulation").unwrap();
        let summary = "<p>Tree growth 3D simulator for my nature exploration game</p>";
        let inbox = Iri::try_from("https://dev.example/aviva/treesim/inbox").unwrap();
        let outbox = Iri::try_from("https://dev.example/aviva/treesim/outbox").unwrap();
        let followers = Iri::try_from("https://dev.example/aviva/treesim/followers").unwrap();

        let key_id = Iri::try_from("https://dev.example/aviva/treesim#main-key").unwrap();
        let controller = Iri::try_from("https://dev.example/aviva/treesim").unwrap();
        let encoded_multibase = "u7QGwDY2Tjn93PVFWWq02piP1NE9_XRlg-c8-jhJiHqKBHw";

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": [
    "Repository",
    "TicketTracker",
    "PatchTracker"
  ],
  "id": "{id}",
  "name": "{name}",
  "summary": "{summary}",
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
  ]
}}"#
        );

        let context = context::forgefed_context();

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

        let repository = PatchTracker::new()
            .with_context_property(context)
            .with_kind(VocabularyTypes::list([
                ActorType::Repository,
                ActorType::TicketTracker,
                ActorType::PatchTracker,
            ]))
            .with_id(id)
            .with_name(name)
            .with_summary(summary)
            .with_inbox(inbox)
            .with_outbox(outbox)
            .with_followers(followers)
            .with_assertion_method([multikey]);

        assert_eq!(serde_json::to_string_pretty(&repository).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<PatchTracker>(json_str.as_str()).unwrap(),
            repository
        );
    }
}
