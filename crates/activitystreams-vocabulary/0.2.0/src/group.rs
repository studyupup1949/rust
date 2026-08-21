use crate::create_actor;

create_actor! {
    /// Represents a formal or informal collective of Actors.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Group, Name};
    ///
    /// # fn main() {
    /// let group_name = Name::try_from("Big Beards of Austin").unwrap();
    /// let group = Group::new().with_name(group_name.clone());
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Group",
    ///   "name": "{group_name}"
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&group).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Group>(json_str.as_str()).unwrap(),
    ///     group
    /// );
    /// # }
    /// ```
    ///
    /// # Example (with `Actor` fields)
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Context, Group, Iri, Name};
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
    ///   "type": "Group",
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
    ///   "liked": "{liked}"
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
    /// let group = Group::new()
    ///     .with_context_property(context)
    ///     .with_name(name)
    ///     .with_summary(summary)
    ///     .with_id(id)
    ///     .with_following(following)
    ///     .with_followers(followers)
    ///     .with_liked(liked)
    ///     .with_inbox(inbox)
    ///     .with_outbox(outbox)
    ///     .with_icon([icon]);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&group).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Group>(json_str.as_str()).unwrap(),
    ///     group
    /// );
    /// # }
    /// ```
    Group {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Context, Iri, Name};

    #[test]
    fn test_valid() {
        let group_name = Name::try_from("Big Beards of Austin").unwrap();
        let group = Group::new().with_name(group_name.clone());

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Group",
  "name": "{group_name}"
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&group).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Group>(json_str.as_str()).unwrap(),
            group
        );
    }

    #[test]
    fn test_invalid() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Group>(json_str).is_err());
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
  "type": "Group",
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
  "liked": "{liked}"
}}"#
        );

        let context_obj = [(context_key, context_val)]
            .map(|(k, v)| (k.to_string(), serde_json::Value::String(v.into())));

        let context = Context::array([
            serde_json::Value::String(context_uri.into()),
            serde_json::Value::Object(context_obj.into_iter().collect()),
        ]);

        let group = Group::new()
            .with_context_property(context)
            .with_name(name)
            .with_summary(summary)
            .with_id(id)
            .with_following(following)
            .with_followers(followers)
            .with_liked(liked)
            .with_inbox(inbox)
            .with_outbox(outbox)
            .with_icon([icon]);

        assert_eq!(serde_json::to_string_pretty(&group).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Group>(json_str.as_str()).unwrap(),
            group
        );
    }
}
