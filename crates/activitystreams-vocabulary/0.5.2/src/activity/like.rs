use crate::create_activity;

create_activity! {
    /// Indicates that the `actor` likes, recommends or endorses the `object`.
    ///
    /// The `target` and `origin` typically have no defined meaning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, Like, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally liked a note";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let object = Iri::try_from("http://example.org/notes/1").unwrap();
    ///
    /// let like = Like::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object.clone());
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Like",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": "{object}"
    /// }}"#);
    ///
    ///
    /// assert_eq!(serde_json::to_string_pretty(&like).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Like>(json_str.as_str()).unwrap(),
    ///     like
    /// );
    /// # }
    /// ```
    Like {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally liked a note";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let object = Iri::try_from("http://example.org/notes/1").unwrap();

        let like = Like::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object.clone());

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Like",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": "{object}"
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&like).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Like>(json_str.as_str()).unwrap(),
            like
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Like>(json_str).is_err());
    }
}
