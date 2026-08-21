use activitystreams_vocabulary::{Collection, create_actor, field_access};

create_actor! {
    /// Represents a project, a planned endeavor that involves usage of tools related to the software development lifecycle.
    ///
    /// It may be a software project, but may also be totally unrelated to software development.
    ///
    /// For example, it may be a book that is being written using Markdown files kept in a Git repository.
    ///
    /// A [Project] object is a way to collect forge related components together under one title.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Project, context};
    /// use activitystreams_vocabulary::{
    ///     Collection, Iri, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyPublicKey, Name,
    /// };
    ///
    /// # fn main() {
    /// let id = Iri::try_from("https://dev.example/projects/wanderer").unwrap();
    /// let name = Name::try_from("Wanderer").unwrap();
    /// let summary = "3D nature exploration game";
    /// let inbox = Iri::try_from("https://dev.example/projects/wanderer/inbox").unwrap();
    /// let outbox = Iri::try_from("https://dev.example/projects/wanderer/outbox").unwrap();
    /// let followers = Iri::try_from("https://dev.example/projects/wanderer/followers").unwrap();
    ///
    /// let key_id = Iri::try_from("https://dev.example/aviva/treesim#main-key").unwrap();
    /// let controller = Iri::try_from("https://dev.example/aviva/treesim").unwrap();
    /// let encoded_multibase = "u7QGwDY2Tjn93PVFWWq02piP1NE9_XRlg-c8-jhJiHqKBHw";
    ///
    /// let item0 = Iri::try_from("https://dev.example/repos/opengl-vegetation").unwrap();
    /// let item1 = Iri::try_from("https://dev.example/repos/opengl-vegetation/patch-tracker").unwrap();
    /// let item2 = Iri::try_from("https://dev.example/repos/treesim").unwrap();
    /// let item3 = Iri::try_from("https://dev.example/repos/treesim/patch-tracker").unwrap();
    /// let item4 = Iri::try_from("https://dev.example/repos/wanderer").unwrap();
    /// let item5 = Iri::try_from("https://dev.example/repos/wanderer/patch-tracker").unwrap();
    /// let item6 = Iri::try_from("https://dev.example/issue-trackers/wanderer").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Project",
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
    ///   "components": {{
    ///     "type": "Collection",
    ///     "totalItems": 7,
    ///     "items": [
    ///       "{item0}",
    ///       "{item1}",
    ///       "{item2}",
    ///       "{item3}",
    ///       "{item4}",
    ///       "{item5}",
    ///       "{item6}"
    ///     ]
    ///   }}
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
    /// let items = [item0, item1, item2, item3, item4, item5, item6];
    /// let components = Collection::new_inner()
    ///     .with_total_items(items.len() as u64)
    ///     .with_items(items);
    ///
    /// let repository = Project::new()
    ///     .with_context_property(context)
    ///     .with_id(id)
    ///     .with_name(name)
    ///     .with_summary(summary)
    ///     .with_inbox(inbox)
    ///     .with_outbox(outbox)
    ///     .with_followers(followers)
    ///     .with_assertion_method([multikey])
    ///     .with_components(components);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&repository).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Project>(json_str.as_str()).unwrap(),
    ///     repository
    /// );
    /// # }
    /// ```
    Project: crate::ActorType::Project {
        components: Option<Collection>,
    }
}

field_access! {
    Project {
        /// Identifies a [Collection](activitystreams_vocabulary::Collection) listing actors whose services and resources are considered to be components of this project.
        ///
        /// The collection items are Relationship objects whose relationship is hasComponent and whose instrument is the maximal role allowed for delegation, specified when the component was added.
        components: option_ref { Collection },
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
        let id = Iri::try_from("https://dev.example/projects/wanderer").unwrap();
        let name = Name::try_from("Wanderer").unwrap();
        let summary = "3D nature exploration game";
        let inbox = Iri::try_from("https://dev.example/projects/wanderer/inbox").unwrap();
        let outbox = Iri::try_from("https://dev.example/projects/wanderer/outbox").unwrap();
        let followers = Iri::try_from("https://dev.example/projects/wanderer/followers").unwrap();

        let key_id = Iri::try_from("https://dev.example/aviva/treesim#main-key").unwrap();
        let controller = Iri::try_from("https://dev.example/aviva/treesim").unwrap();
        let encoded_multibase = "u7QGwDY2Tjn93PVFWWq02piP1NE9_XRlg-c8-jhJiHqKBHw";

        let item0 = Iri::try_from("https://dev.example/repos/opengl-vegetation").unwrap();
        let item1 =
            Iri::try_from("https://dev.example/repos/opengl-vegetation/patch-tracker").unwrap();
        let item2 = Iri::try_from("https://dev.example/repos/treesim").unwrap();
        let item3 = Iri::try_from("https://dev.example/repos/treesim/patch-tracker").unwrap();
        let item4 = Iri::try_from("https://dev.example/repos/wanderer").unwrap();
        let item5 = Iri::try_from("https://dev.example/repos/wanderer/patch-tracker").unwrap();
        let item6 = Iri::try_from("https://dev.example/issue-trackers/wanderer").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Project",
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
  "components": {{
    "type": "Collection",
    "totalItems": 7,
    "items": [
      "{item0}",
      "{item1}",
      "{item2}",
      "{item3}",
      "{item4}",
      "{item5}",
      "{item6}"
    ]
  }}
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

        let items = [item0, item1, item2, item3, item4, item5, item6];
        let components = Collection::new_inner()
            .with_total_items(items.len() as u64)
            .with_items(items);

        let repository = Project::new()
            .with_context_property(context)
            .with_id(id)
            .with_name(name)
            .with_summary(summary)
            .with_inbox(inbox)
            .with_outbox(outbox)
            .with_followers(followers)
            .with_assertion_method([multikey])
            .with_components(components);

        assert_eq!(serde_json::to_string_pretty(&repository).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Project>(json_str.as_str()).unwrap(),
            repository
        );
    }
}
