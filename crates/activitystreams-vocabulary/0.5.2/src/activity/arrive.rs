use crate::create_intransitive_activity;

create_intransitive_activity! {
    /// An [IntransitiveActivity](crate::IntransitiveActivity) that indicates that the actor has arrived at the `location`.
    ///
    /// The `origin` can be used to identify the context from which the actor originated.
    ///
    /// The target typically has no defined meaning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Arrive, Name, Person, Place};
    ///
    /// # fn main() {
    /// let summary = "Sally arrived at work";
    ///
    /// let actor_name =  Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let location_name =  Name::try_from("Work").unwrap();
    /// let location = Place::new_inner().with_name(location_name.clone());
    ///
    /// let origin_name =  Name::try_from("Home").unwrap();
    /// let origin = Place::new_inner().with_name(origin_name.clone());
    ///
    /// let arrive = Arrive::new()
    ///     .with_summary(summary)
    ///     .with_location(location)
    ///     .with_actor(actor)
    ///     .with_origin(origin);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Arrive",
    ///   "summary": "{summary}",
    ///   "location": {{
    ///     "type": "Place",
    ///     "name": "{location_name}"
    ///   }},
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "origin": {{
    ///     "type": "Place",
    ///     "name": "{origin_name}"
    ///   }}
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&arrive).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Arrive>(json_str.as_str()).unwrap(),
    ///     arrive
    /// );
    /// # }
    Arrive {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Person, Place};

    #[test]
    fn test_activity() {
        let summary = "Sally arrived at work";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let location_name = Name::try_from("Work").unwrap();
        let location = Place::new_inner().with_name(location_name.clone());

        let origin_name = Name::try_from("Home").unwrap();
        let origin = Place::new_inner().with_name(origin_name.clone());

        let arrive = Arrive::new()
            .with_summary(summary)
            .with_location(location)
            .with_actor(actor)
            .with_origin(origin);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Arrive",
  "summary": "{summary}",
  "location": {{
    "type": "Place",
    "name": "{location_name}"
  }},
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "origin": {{
    "type": "Place",
    "name": "{origin_name}"
  }}
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&arrive).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Arrive>(json_str.as_str()).unwrap(),
            arrive
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Arrive>(json_str).is_err());
    }
}
