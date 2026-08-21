use crate::create_activity;

create_activity! {
    /// Indicates that the `actor` has listened to the `object`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Iri, Listen, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally listened to a piece of music";
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let object = Iri::try_from("http://example.org/music.mp3").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Listen",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": "{object}"
    /// }}"#);
    ///
    /// let actor = Person::new_inner().with_name(actor_name);
    /// let listen = Listen::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&listen).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Listen>(json_str.as_str()).unwrap(),
    ///     listen
    /// );
    /// # }
    /// ```
    Listen {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Iri, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally listened to a piece of music";
        let actor_name = Name::try_from("Sally").unwrap();
        let object = Iri::try_from("http://example.org/music.mp3").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Listen",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": "{object}"
}}"#
        );

        let actor = Person::new_inner().with_name(actor_name);

        let listen = Listen::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        assert_eq!(serde_json::to_string_pretty(&listen).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Listen>(json_str.as_str()).unwrap(),
            listen
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Listen>(json_str).is_err());
    }
}
