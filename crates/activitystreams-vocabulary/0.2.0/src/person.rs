use crate::create_actor;

create_actor! {
    /// Represents an individual person.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Name, Person};
    ///
    /// # fn main() {
    /// let name = Name::try_from("Sally Smith").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Person",
    ///   "name": "{name}"
    /// }}"#
    ///     );
    ///
    /// let person = Person::new().with_name(name);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&person).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Person>(json_str.as_str()).unwrap(),
    ///     person
    /// );
    /// # }
    /// ```
    ///
    /// # Example (with `Actor` fields)
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Context, Iri, Name, Person};
    ///
    /// # fn main() {
    /// let context_uri = Context::URI;
    /// let context_key = "@language";
    /// let context_val = "ja";
    ///
    /// let id = Iri::try_from("https://kenzoishii.example.com/").unwrap();
    /// let following = Iri::try_from("https://kenzoishii.example.com/following.json").unwrap();
    /// let followers = Iri::try_from("https://kenzoishii.example.com/followers.json").unwrap();
    /// let liked = Iri::try_from("https://kenzoishii.example.com/liked.json").unwrap();
    /// let inbox = Iri::try_from("https://kenzoishii.example.com/inbox.json").unwrap();
    /// let outbox = Iri::try_from("https://kenzoishii.example.com/feed.json").unwrap();
    /// let preferred_username = Name::try_from("kenzoishii").unwrap();
    /// let name = Name::try_from("石井健蔵").unwrap();
    /// let summary = "この方はただの例です";
    /// let icon = Iri::try_from("https://kenzoishii.example.com/image/165987aklre4").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "{context_uri}",
    ///     {{
    ///       "{context_key}": "{context_val}"
    ///     }}
    ///   ],
    ///   "type": "Person",
    ///   "id": "{id}",
    ///   "name": "{name}",
    ///   "summary": "{summary}",
    ///   "icon": [
    ///     "{icon}"
    ///   ],
    ///   "inbox": "{inbox}",
    ///   "outbox": "{outbox}",
    ///   "following": "{following}",
    ///   "followers": "{followers}",
    ///   "liked": "{liked}",
    ///   "preferredUsername": "{preferred_username}"
    /// }}"#);
    ///
    /// let context_obj = [(context_key, context_val)]
    ///     .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.into())));
    ///
    /// let context = Context::array([
    ///     serde_json::Value::String(context_uri.into()),
    ///     serde_json::Value::Object(context_obj.into_iter().collect()),
    /// ]);
    ///
    /// let person = Person::new()
    ///     .with_context_property(context)
    ///     .with_name(name)
    ///     .with_summary(summary)
    ///     .with_id(id)
    ///     .with_following(following)
    ///     .with_followers(followers)
    ///     .with_liked(liked)
    ///     .with_inbox(inbox)
    ///     .with_outbox(outbox)
    ///     .with_preferred_username(preferred_username)
    ///     .with_icon([icon]);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&person).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Person>(json_str.as_str()).unwrap(),
    ///     person
    /// );
    /// # }
    /// ```
    ///
    /// # Example (with `assertionMethod` field)
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Context, Iri, MultibaseHeader, MultibasePublicKey, MultikeyPublicKey, Multikey, Person};
    ///
    /// # fn main() {
    /// let streams_context = "https://www.w3.org/ns/activitystreams";
    /// let cid_context = "https://www.w3.org/ns/cid/v1";
    /// let id = Iri::try_from("https://server.example/users/alice").unwrap();
    /// let inbox = Iri::try_from("https://server.example/users/alice/inbox").unwrap();
    /// let outbox = Iri::try_from("https://server.example/users/alice/outbox").unwrap();
    /// let key_id = Iri::try_from("https://server.example/users/alice#ed25519-key").unwrap();
    /// let controller = Iri::try_from("https://server.example/users/alice").unwrap();
    /// let encoded_multibase = "z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2";
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "{streams_context}",
    ///     "{cid_context}"
    ///   ],
    ///   "type": "Person",
    ///   "id": "{id}",
    ///   "inbox": "{inbox}",
    ///   "outbox": "{outbox}",
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
    /// let context = Context::array([
    ///     serde_json::Value::String(streams_context.into()),
    ///     serde_json::Value::String(cid_context.into()),
    /// ]);
    ///
    /// let multibase = MultibasePublicKey::new()
    ///     .with_header(MultibaseHeader::Base58Btc)
    ///     .with_key(MultikeyPublicKey::Ed25519([
    ///         0xb0, 0xd, 0x8d, 0x93, 0x8e, 0x7f, 0x77, 0x3d, 0x51, 0x56, 0x5a, 0xad, 0x36, 0xa6,
    ///         0x23, 0xf5, 0x34, 0x4f, 0x7f, 0x5d, 0x19, 0x60, 0xf9, 0xcf, 0x3e, 0x8e, 0x12, 0x62,
    ///         0xe, 0xa2, 0x81, 0xf,
    ///     ]));
    ///
    /// let multikey = Multikey::new_inner()
    ///     .with_id(key_id)
    ///     .with_controller(controller)
    ///     .with_public_key_multibase(multibase);
    ///
    /// let person = Person::new()
    ///     .with_context_property(context)
    ///     .with_id(id)
    ///     .with_inbox(inbox)
    ///     .with_outbox(outbox)
    ///     .with_assertion_method([multikey]);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&person).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Person>(json_str.as_str()).unwrap(),
    ///     person
    /// );
    /// # }
    /// ```
    Person {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Context, Iri, MultibaseHeader, MultibasePublicKey, Multikey, MultikeyPublicKey, Name,
    };

    #[test]
    fn test_valid() {
        let name = Name::try_from("Sally Smith").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Person",
  "name": "{name}"
}}"#
        );

        let person = Person::new().with_name(name);

        assert_eq!(serde_json::to_string_pretty(&person).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Person>(json_str.as_str()).unwrap(),
            person
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Person>(json_str).is_err());
    }

    #[test]
    fn test_actor_fields() {
        let context_uri = Context::URI;
        let context_key = "@language";
        let context_val = "ja";

        let id = Iri::try_from("https://kenzoishii.example.com/").unwrap();
        let following = Iri::try_from("https://kenzoishii.example.com/following.json").unwrap();
        let followers = Iri::try_from("https://kenzoishii.example.com/followers.json").unwrap();
        let liked = Iri::try_from("https://kenzoishii.example.com/liked.json").unwrap();
        let inbox = Iri::try_from("https://kenzoishii.example.com/inbox.json").unwrap();
        let outbox = Iri::try_from("https://kenzoishii.example.com/feed.json").unwrap();
        let preferred_username = Name::try_from("kenzoishii").unwrap();
        let name = Name::try_from("石井健蔵").unwrap();
        let summary = "この方はただの例です";
        let icon = Iri::try_from("https://kenzoishii.example.com/image/165987aklre4").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "{context_uri}",
    {{
      "{context_key}": "{context_val}"
    }}
  ],
  "type": "Person",
  "id": "{id}",
  "name": "{name}",
  "summary": "{summary}",
  "icon": [
    "{icon}"
  ],
  "inbox": "{inbox}",
  "outbox": "{outbox}",
  "following": "{following}",
  "followers": "{followers}",
  "liked": "{liked}",
  "preferredUsername": "{preferred_username}"
}}"#
        );

        let context_obj = [(context_key, context_val)]
            .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.into())));

        let context = Context::array([
            serde_json::Value::String(context_uri.into()),
            serde_json::Value::Object(context_obj.into_iter().collect()),
        ]);

        let person = Person::new()
            .with_context_property(context)
            .with_name(name)
            .with_summary(summary)
            .with_id(id)
            .with_following(following)
            .with_followers(followers)
            .with_liked(liked)
            .with_inbox(inbox)
            .with_outbox(outbox)
            .with_preferred_username(preferred_username)
            .with_icon([icon]);

        assert_eq!(serde_json::to_string_pretty(&person).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Person>(json_str.as_str()).unwrap(),
            person
        );
    }

    #[test]
    fn test_assertion_method() {
        let streams_context = "https://www.w3.org/ns/activitystreams";
        let cid_context = "https://www.w3.org/ns/cid/v1";
        let id = Iri::try_from("https://server.example/users/alice").unwrap();
        let inbox = Iri::try_from("https://server.example/users/alice/inbox").unwrap();
        let outbox = Iri::try_from("https://server.example/users/alice/outbox").unwrap();
        let key_id = Iri::try_from("https://server.example/users/alice#ed25519-key").unwrap();
        let controller = Iri::try_from("https://server.example/users/alice").unwrap();
        let encoded_multibase = "z6MkrJVnaZkeFzdQyMZu1cgjg7k1pZZ6pvBQ7XJPt4swbTQ2";

        let json_str = format!(
            r#"{{
  "@context": [
    "{streams_context}",
    "{cid_context}"
  ],
  "type": "Person",
  "id": "{id}",
  "inbox": "{inbox}",
  "outbox": "{outbox}",
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
        let context = Context::array([
            serde_json::Value::String(streams_context.into()),
            serde_json::Value::String(cid_context.into()),
        ]);

        let multibase = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base58Btc)
            .with_key(MultikeyPublicKey::Ed25519([
                0xb0, 0xd, 0x8d, 0x93, 0x8e, 0x7f, 0x77, 0x3d, 0x51, 0x56, 0x5a, 0xad, 0x36, 0xa6,
                0x23, 0xf5, 0x34, 0x4f, 0x7f, 0x5d, 0x19, 0x60, 0xf9, 0xcf, 0x3e, 0x8e, 0x12, 0x62,
                0xe, 0xa2, 0x81, 0xf,
            ]));

        let multikey = Multikey::new_inner()
            .with_id(key_id)
            .with_controller(controller)
            .with_public_key_multibase(multibase);

        let person = Person::new()
            .with_context_property(context)
            .with_id(id)
            .with_inbox(inbox)
            .with_outbox(outbox)
            .with_assertion_method([multikey]);

        assert_eq!(serde_json::to_string_pretty(&person).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Person>(json_str.as_str()).unwrap(),
            person
        );
    }
}
