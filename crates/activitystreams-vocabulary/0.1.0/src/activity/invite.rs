use crate::{ActivityType, create_activity};

create_activity! {
    ///	A specialization of [Offer](crate::Offer) in which the `actor` is extending an invitation for the `object` to the `target`.
    ///
    ///	# Example
    ///
    ///	```rust
    /// use activitystreams_vocabulary::{Event, Invite, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally invited John and Lisa to a party";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let object_name = Name::try_from("A Party").unwrap();
    /// let object = Event::new_inner().with_name(object_name.clone());
    ///
    /// let target_john_name = Name::try_from("John").unwrap();
    /// let target_john = Person::new_inner().with_name(target_john_name.clone());
    ///
    /// let target_lisa_name = Name::try_from("Lisa").unwrap();
    /// let target_lisa = Person::new_inner().with_name(target_lisa_name.clone());
    ///
    /// let invite = Invite::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object)
    ///     .with_target([target_john, target_lisa]);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Invite",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": {{
    ///     "type": "Event",
    ///     "name": "{object_name}"
    ///   }},
    ///   "target": [
    ///     {{
    ///       "type": "Person",
    ///       "name": "{target_john_name}"
    ///     }},
    ///     {{
    ///       "type": "Person",
    ///       "name": "{target_lisa_name}"
    ///     }}
    ///   ]
    /// }}"#
    /// );
    ///
    /// assert_eq!(serde_json::to_string_pretty(&invite).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Invite>(json_str.as_str()).unwrap(),
    ///     invite
    /// );
    /// # }
    ///	```
    Invite {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Event, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally invited John and Lisa to a party";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let object_name = Name::try_from("A Party").unwrap();
        let object = Event::new_inner().with_name(object_name.clone());

        let target_john_name = Name::try_from("John").unwrap();
        let target_john = Person::new_inner().with_name(target_john_name.clone());

        let target_lisa_name = Name::try_from("Lisa").unwrap();
        let target_lisa = Person::new_inner().with_name(target_lisa_name.clone());

        let invite = Invite::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object)
            .with_target([target_john, target_lisa]);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Invite",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": {{
    "type": "Event",
    "name": "{object_name}"
  }},
  "target": [
    {{
      "type": "Person",
      "name": "{target_john_name}"
    }},
    {{
      "type": "Person",
      "name": "{target_lisa_name}"
    }}
  ]
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&invite).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Invite>(json_str.as_str()).unwrap(),
            invite
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Invite>(json_str).is_err());
    }
}
