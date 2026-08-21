use crate::{ActivityType, create_activity};

create_activity! {
    /// Indicates that the `actor` has read the `object`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, Name, Person, Read};
    ///
    /// # fn main() {
    /// let summary = "Sally read a blog post";
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let object = Iri::try_from("http://example.org/posts/1").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Read",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": "{object}"
    /// }}"#);
    ///
    /// let actor = Person::new_inner().with_name(actor_name);
    /// let read = Read::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&read).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Read>(json_str.as_str()).unwrap(),
    ///     read
    /// );
    /// # }
    /// ```
    Read {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally read a blog post";
        let actor_name = Name::try_from("Sally").unwrap();
        let object = Iri::try_from("http://example.org/posts/1").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Read",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": "{object}"
}}"#
        );

        let actor = Person::new_inner().with_name(actor_name);
        let read = Read::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        assert_eq!(serde_json::to_string_pretty(&read).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Read>(json_str.as_str()).unwrap(),
            read
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Read>(json_str).is_err());
    }
}
