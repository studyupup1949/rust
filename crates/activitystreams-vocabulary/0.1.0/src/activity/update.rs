use crate::{ActivityType, create_activity};

create_activity! {
    /// Indicates that the `actor` has updated the `object`.
    ///
    /// Note, however, that this vocabulary does not define a mechanism for describing the actual set of modifications made to object.
    ///
    ///
    /// The `target` and `origin` typically have no defined meaning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, Name, Person, Update};
    ///
    /// # fn main() {
    /// let summary = "Sally updated her note";
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let object = Iri::try_from("http://example.org/notes/1").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Update",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": "{object}"
    /// }}"#);
    ///
    /// let actor = Person::new_inner().with_name(actor_name);
    /// let update = Update::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&update).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Update>(json_str.as_str()).unwrap(),
    ///     update
    /// );
    /// # }
    /// ```
    Update {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally updated her note";
        let actor_name = Name::try_from("Sally").unwrap();
        let object = Iri::try_from("http://example.org/notes/1").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Update",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": "{object}"
}}"#
        );

        let actor = Person::new_inner().with_name(actor_name);
        let update = Update::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        assert_eq!(serde_json::to_string_pretty(&update).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Update>(json_str.as_str()).unwrap(),
            update
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Update>(json_str).is_err());
    }
}
