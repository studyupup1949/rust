use crate::create_activity;

mod tentative;

pub use tentative::TentativeAccept;

create_activity! {
    /// Indicates that the `actor` accepts the `object`.
    ///
    /// The target property can be used in certain circumstances to indicate the context into which the object has been accepted.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Accept, Event, Invite, Iri, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally accepted an invitation to a party";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let event_name = Name::try_from("Going-Away Party for Jim").unwrap();
    /// let event = Event::new_inner().with_name(event_name.clone());
    ///
    /// let object_actor = Iri::try_from("http://john.example.org").unwrap();
    /// let object = Invite::new_inner()
    ///     .with_actor(object_actor.clone())
    ///     .with_object(event.clone());
    ///
    /// let accept = Accept::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor.clone())
    ///     .with_object(object.clone());
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Accept",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": {{
    ///     "type": "Invite",
    ///     "actor": "{object_actor}",
    ///     "object": {{
    ///       "type": "Event",
    ///       "name": "{event_name}"
    ///     }}
    ///   }}
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&accept).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Accept>(json_str.as_str()).unwrap(),
    ///     accept
    /// );
    /// # }
    /// ```
    Accept {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, Invite, Iri, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally accepted an invitation to a party";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let event_name = Name::try_from("Going-Away Party for Jim").unwrap();
        let event = Event::new_inner().with_name(event_name.clone());

        let object_actor = Iri::try_from("http://john.example.org").unwrap();
        let object = Invite::new_inner()
            .with_actor(object_actor.clone())
            .with_object(event.clone());

        let accept = Accept::new()
            .with_summary(summary)
            .with_actor(actor.clone())
            .with_object(object.clone());

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Accept",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": {{
    "type": "Invite",
    "actor": "{object_actor}",
    "object": {{
      "type": "Event",
      "name": "{event_name}"
    }}
  }}
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&accept).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Accept>(json_str.as_str()).unwrap(),
            accept
        );
    }

    #[test]
    fn test_invalid_activity() {
        let id: Iri = "http://www.test.example/object/1".try_into().unwrap();
        let name: Name = "A Simple, non-specific object".try_into().unwrap();

        let json_str = format!(
            r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object","id":"{id}","name":"{name}"}}"#
        );

        assert!(serde_json::from_str::<Accept>(json_str.as_str()).is_err());
    }
}
