use crate::{ActivityType, create_activity};

create_activity! {
    ///  Indicates that the `actor` is calling the target's attention the `object`.
    ///
    /// The `origin` typically has no defined meaning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Announce, Arrive, Iri, Name, Person, Place};
    ///
    /// # fn main() {
    /// let summary = "Sally announced that she had arrived at work";
    ///
    /// let actor_id = Iri::try_from("http://sally.example.org").unwrap();
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner()
    ///     .with_id(actor_id.clone())
    ///     .with_name(actor_name.clone());
    ///
    /// let object_actor = Iri::try_from("http://sally.example.org").unwrap();
    /// let location_name = Name::try_from("Work").unwrap();
    /// let object_location = Place::new_inner().with_name(location_name.clone());
    /// let object = Arrive::new_inner()
    ///     .with_actor(object_actor.clone())
    ///     .with_location(object_location);
    ///
    /// let announce = Announce::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Announce",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "id": "{actor_id}",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": {{
    ///     "type": "Arrive",
    ///     "location": {{
    ///       "type": "Place",
    ///       "name": "{location_name}"
    ///     }},
    ///     "actor": "{object_actor}"
    ///   }}
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&announce).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Announce>(json_str.as_str()).unwrap(),
    ///     announce
    /// );
    /// # }
    Announce {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Arrive, Iri, Name, Person, Place};

    #[test]
    fn test_activity() {
        let summary = "Sally announced that she had arrived at work";

        let actor_id = Iri::try_from("http://sally.example.org").unwrap();
        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner()
            .with_id(actor_id.clone())
            .with_name(actor_name.clone());

        let object_actor = Iri::try_from("http://sally.example.org").unwrap();
        let location_name = Name::try_from("Work").unwrap();
        let object_location = Place::new_inner().with_name(location_name.clone());
        let object = Arrive::new_inner()
            .with_actor(object_actor.clone())
            .with_location(object_location);

        let announce = Announce::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Announce",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "id": "{actor_id}",
    "name": "{actor_name}"
  }},
  "object": {{
    "type": "Arrive",
    "location": {{
      "type": "Place",
      "name": "{location_name}"
    }},
    "actor": "{object_actor}"
  }}
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&announce).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Announce>(json_str.as_str()).unwrap(),
            announce
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object""}}"#;

        assert!(serde_json::from_str::<Announce>(json_str).is_err());
    }
}
