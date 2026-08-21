use crate::{ActivityType, create_activity};

create_activity! {
    /// Indicates that the `actor` is ignoring the `object`.
    ///
    /// The `target` and `origin` typically have no defined meaning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Ignore, Iri, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally ignored a note";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let object = Iri::try_from("http://example.org/notes/1").unwrap();
    ///
    /// let ignore = Ignore::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object.clone());
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Ignore",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": "{object}"
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&ignore).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Ignore>(json_str.as_str()).unwrap(),
    ///     ignore
    /// );
    /// # }
    /// ```
    Ignore {}

}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally ignored a note";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let object = Iri::try_from("http://example.org/notes/1").unwrap();

        let ignore = Ignore::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object.clone());

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Ignore",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": "{object}"
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&ignore).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Ignore>(json_str.as_str()).unwrap(),
            ignore
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Ignore>(json_str).is_err());
    }
}
