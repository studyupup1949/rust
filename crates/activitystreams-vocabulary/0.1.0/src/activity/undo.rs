use crate::{ActivityType, create_activity};

create_activity! {
    /// Indicates that the `actor` is undoing the `object`.
    ///
    /// In most cases, the `object` will be an [Activity](crate::Activity) describing some previously performed action (for instance,
    /// a person may have previously "liked" an article but, for whatever reason,
    /// might choose to undo that like at some later point in time).
    ///
    /// The `target` and `origin` typically have no defined meaning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, Name, Offer, Undo};
    ///
    /// # fn main() {
    /// let summary = "Sally retracted her offer to John";
    /// let actor = Iri::try_from("http://sally.example.org").unwrap();
    /// let object_actor = Iri::try_from("http://sally.example.org").unwrap();
    /// let object_object = Iri::try_from("http://example.org/posts/1").unwrap();
    /// let object_target = Iri::try_from("http://john.example.org").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Undo",
    ///   "summary": "{summary}",
    ///   "actor": "{actor}",
    ///   "object": {{
    ///     "type": "Offer",
    ///     "actor": "{object_actor}",
    ///     "object": "{object_object}",
    ///     "target": "{object_target}"
    ///   }}
    /// }}"#);
    ///
    /// let object = Offer::new_inner()
    ///     .with_actor(object_actor)
    ///     .with_object(object_object)
    ///     .with_target(object_target);
    ///
    /// let undo = Undo::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&undo).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Undo>(json_str.as_str()).unwrap(),
    ///     undo
    /// );
    /// # }
    /// ```
    Undo {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name, Offer};

    #[test]
    fn test_activity() {
        let summary = "Sally retracted her offer to John";
        let actor = Iri::try_from("http://sally.example.org").unwrap();
        let object_actor = Iri::try_from("http://sally.example.org").unwrap();
        let object_object = Iri::try_from("http://example.org/posts/1").unwrap();
        let object_target = Iri::try_from("http://john.example.org").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Undo",
  "summary": "{summary}",
  "actor": "{actor}",
  "object": {{
    "type": "Offer",
    "actor": "{object_actor}",
    "object": "{object_object}",
    "target": "{object_target}"
  }}
}}"#
        );

        let object = Offer::new_inner()
            .with_actor(object_actor)
            .with_object(object_object)
            .with_target(object_target);

        let undo = Undo::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        assert_eq!(serde_json::to_string_pretty(&undo).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Undo>(json_str.as_str()).unwrap(),
            undo
        );
    }

    #[test]
    fn test_invalid_activity() {
        let id: Iri = "http://www.test.example/object/1".try_into().unwrap();
        let name: Name = "A Simple, non-specific object".try_into().unwrap();

        let json_str = format!(
            r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object","id":"{id}","name":"{name}"}}"#
        );

        assert!(serde_json::from_str::<Undo>(json_str.as_str()).is_err());
    }
}
