use crate::{ActivityType, create_activity};

create_activity! {
    /// A specialization of [Accept](crate::Accept) indicating that the acceptance is tentative.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Event, Invite, Iri, Name, Person, TentativeAccept};
    ///
    /// # fn main() {
    /// let summary = "Sally tentatively accepted an invitation to a party";
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
    /// let accept = TentativeAccept::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "TentativeAccept",
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
    ///     serde_json::from_str::<TentativeAccept>(json_str.as_str()).unwrap(),
    ///     accept
    /// );
    /// # }
    TentativeAccept {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, Invite, Iri, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally tentatively accepted an invitation to a party";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let event_name = Name::try_from("Going-Away Party for Jim").unwrap();
        let event = Event::new_inner().with_name(event_name.clone());

        let object_actor = Iri::try_from("http://john.example.org").unwrap();
        let object = Invite::new_inner()
            .with_actor(object_actor.clone())
            .with_object(event.clone());

        let accept = TentativeAccept::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "TentativeAccept",
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
            serde_json::from_str::<TentativeAccept>(json_str.as_str()).unwrap(),
            accept
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<TentativeAccept>(json_str).is_err());
    }
}
