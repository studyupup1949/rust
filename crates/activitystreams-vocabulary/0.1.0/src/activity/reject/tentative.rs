use crate::{ActivityType, create_activity};

create_activity! {
    /// A specialization of [Reject](crate::Reject) in which the rejection is considered tentative.
    ///
    ///	# Example
    ///
    ///	```rust
    /// use activitystreams_vocabulary::{Event, Invite, Iri, Name, Person, TentativeReject};
    ///
    /// # fn main() {
    /// let summary = "Sally tentatively rejected an invitation to a party";
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let object_actor = Iri::try_from("http://john.example.org").unwrap();
    /// let object_object_name = Name::try_from("Going-Away Party for Jim").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "TentativeReject",
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
    ///       "name": "{object_object_name}"
    ///     }}
    ///   }}
    /// }}"#);
    ///
    /// let actor = Person::new_inner().with_name(actor_name);
    /// let object_object = Event::new_inner().with_name(object_object_name);
    /// let object = Invite::new_inner()
    ///     .with_actor(object_actor)
    ///     .with_object(object_object);
    /// let reject = TentativeReject::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&reject).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<TentativeReject>(json_str.as_str()).unwrap(),
    ///     reject
    /// );
    /// # }
    ///	```
    TentativeReject {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, Invite, Iri, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally tentatively rejected an invitation to a party";
        let actor_name = Name::try_from("Sally").unwrap();
        let object_actor = Iri::try_from("http://john.example.org").unwrap();
        let object_object_name = Name::try_from("Going-Away Party for Jim").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "TentativeReject",
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
      "name": "{object_object_name}"
    }}
  }}
}}"#
        );

        let actor = Person::new_inner().with_name(actor_name);
        let object_object = Event::new_inner().with_name(object_object_name);
        let object = Invite::new_inner()
            .with_actor(object_actor)
            .with_object(object_object);
        let reject = TentativeReject::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        assert_eq!(serde_json::to_string_pretty(&reject).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<TentativeReject>(json_str.as_str()).unwrap(),
            reject
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<TentativeReject>(json_str).is_err());
    }
}
