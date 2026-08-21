use activitystreams_vocabulary::{
    CollectionItem, Iri, Iris, LinkItem, OrderedCollectionItem, create_actor, create_item,
    field_access,
};

use crate::{PatchTrackerItem, TeamItem, TicketTrackerItem};

create_actor! {
    /// Represents a version control system repository.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Repository, context};
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
    ///   "type": "Repository",
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
    ///   ],
    ///   "team": "{team}"
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
    /// let repository = Repository::new()
    ///     .with_context_property(context)
    ///     .with_id(id)
    ///     .with_name(name)
    ///     .with_summary(summary)
    ///     .with_inbox(inbox)
    ///     .with_outbox(outbox)
    ///     .with_followers(followers)
    ///     .with_assertion_method([multikey])
    ///     .with_team(team);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&repository).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Repository>(json_str.as_str()).unwrap(),
    ///     repository
    /// );
    /// # }
    /// ```
    Repository: crate::ActorType::Repository {
        #[serde(skip_serializing_if = "Option::is_none")]
        clone_uri: Option<Iris>,
        #[serde(skip_serializing_if = "Option::is_none")]
        push_uri: Option<Iris>,
        #[serde(skip_serializing_if = "Option::is_none")]
        forks: Option<OrderedCollectionItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        tickets_tracked_by: Option<TicketTrackerItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        send_patches_to: Option<PatchTrackerItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        likes: Option<CollectionItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        is_archived: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        moved_to: Option<LinkItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        mirrors: Option<RepositoryItem>,
        #[serde(skip_serializing_if = "Option::is_none")]
        team: Option<TeamItem>,
    }
}

create_item! {
    /// Represents a [Repository], or an [Iri] referencing a [Repository].
    RepositoryItem
        boxed
        default: Self::Repository(Box::default()),
    {
        Repository(Repository),
        Iri(Iri),
    }
}

field_access! {
    Repository {
        /// The endpoint from which the content of the repository can be obtained via the native protocol (Git, Hg, etc.)
        clone_uri: option_ref { Iris },
        /// The endpoint through which repository commits can be pushed via the native protocol.
        push_uri: option_ref { Iris },
        /// Represents repositories that are forks of this repository.
        forks: option_ref { OrderedCollectionItem },
        /// Represents the [Like](crate::Like) activities, i.e. stars, that the repo received.
        likes: option_ref { CollectionItem },
        /// Represents a new location the [Repository] moved to, if it is archived.
        moved_to: option_ref { LinkItem },
        /// Identifies the [Repository] which this [Repository] copies content from (i.e. what this repository is a "pull mirror" of).
        mirrors: option_ref { RepositoryItem },
        /// Represents the optional [TicketTracker](crate::TicketTracker) used to track issues.
        tickets_tracked_by: option_ref { TicketTrackerItem },
        /// Represents the optional [PatchTracker](crate::PatchTracker) used to track patches.
        send_patches_to: option_ref { PatchTrackerItem },
        /// Represents people who have push/edit access, the "collaborators" of the repository.
        ///
        /// Specifies a [Collection](activitystreams_vocabulary::Collection) of actors who are working on the object, or responsible for it, or managing or administrating it, or having edit access to it.
        team: option_ref { TeamItem },
    }
}

field_access! {
    Repository {
        /// Represents whether the repo is in read-only mode.
        is_archived: option { bool },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;
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

        let team = Iri::try_from("https://dev.example/aviva/treesim/team").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Repository",
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
  ],
  "team": "{team}"
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

        let repository = Repository::new()
            .with_context_property(context)
            .with_id(id)
            .with_name(name)
            .with_summary(summary)
            .with_inbox(inbox)
            .with_outbox(outbox)
            .with_followers(followers)
            .with_assertion_method([multikey])
            .with_team(team);

        assert_eq!(serde_json::to_string_pretty(&repository).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Repository>(json_str.as_str()).unwrap(),
            repository
        );
    }
}
