use crate::create_activity;

create_activity! {
    /// Indicates that the `actor` has created the `object`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Create, Name, Note, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally created a note";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let object_name = Name::try_from("A Simple Note").unwrap();
    /// let object_content = "This is a simple note";
    /// let object = Note::new_inner()
    ///     .with_name(object_name.clone())
    ///     .with_content(object_content);
    ///
    /// let create = Create::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Create",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": {{
    ///     "type": "Note",
    ///     "name": "{object_name}",
    ///     "content": "{object_content}"
    ///   }}
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&create).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Create>(json_str.as_str()).unwrap(),
    ///     create
    /// );
    /// # }
    /// ```
    Create {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Note, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally created a note";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let object_name = Name::try_from("A Simple Note").unwrap();
        let object_content = "This is a simple note";
        let object = Note::new_inner()
            .with_name(object_name.clone())
            .with_content(object_content);

        let create = Create::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Create",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": {{
    "type": "Note",
    "name": "{object_name}",
    "content": "{object_content}"
  }}
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&create).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Create>(json_str.as_str()).unwrap(),
            create
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Create>(json_str).is_err());
    }
}
