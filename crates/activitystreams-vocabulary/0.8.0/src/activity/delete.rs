use crate::create_activity;

create_activity! {
    /// Indicates that the `actor` has deleted the `object`.
    ///
    /// If specified, the `origin` indicates the context from which the `object` was deleted.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Collection, Delete, Iri, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally deleted a note";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let object = Iri::try_from("http://example.org/notes/1").unwrap();
    ///
    /// let origin_name = Name::try_from("Sally's Notes").unwrap();
    /// let origin = Collection::new_inner().with_name(origin_name.clone());
    ///
    /// let delete = Delete::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object.clone())
    ///     .with_origin(origin);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Delete",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": "{object}",
    ///   "origin": {{
    ///     "type": "Collection",
    ///     "name": "{origin_name}"
    ///   }}
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&delete).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Delete>(json_str.as_str()).unwrap(),
    ///     delete
    /// );
    /// # }
    /// ```
    Delete {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Collection, Iri, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally deleted a note";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let object = Iri::try_from("http://example.org/notes/1").unwrap();

        let origin_name = Name::try_from("Sally's Notes").unwrap();
        let origin = Collection::new_inner().with_name(origin_name.clone());

        let delete = Delete::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object.clone())
            .with_origin(origin);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Delete",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": "{object}",
  "origin": {{
    "type": "Collection",
    "name": "{origin_name}"
  }}
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&delete).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Delete>(json_str.as_str()).unwrap(),
            delete
        );
    }

    #[test]
    fn test_invalid_activity() {
        let id: Iri = "http://www.test.example/object/1".try_into().unwrap();
        let name: Name = "A Simple, non-specific object".try_into().unwrap();

        let json_str = format!(
            r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object","id":"{id}","name":"{name}"}}"#
        );

        assert!(serde_json::from_str::<Delete>(json_str.as_str()).is_err());
    }
}
