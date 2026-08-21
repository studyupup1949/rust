use crate::{ActivityType, create_activity};

create_activity! {
    /// Indicates that the `actor` is removing the `object`.
    ///
    /// If specified, the `origin` indicates the context from which the `object` is being removed.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Group, Name, Object, Person, Remove, VocabularyType};
    ///
    /// # fn main() {
    /// let summary = "The moderator removed Sally from a group";
    ///
    /// let actor_type = VocabularyType::iri("http://example.org/Role").unwrap();
    /// let actor_name = Name::try_from("The Moderator").unwrap();
    ///
    /// let object_name = Name::try_from("Sally").unwrap();
    /// let origin_name = Name::try_from("A Simple Group").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Remove",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "{actor_type}",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": {{
    ///     "type": "Person",
    ///     "name": "{object_name}"
    ///   }},
    ///   "origin": {{
    ///     "type": "Group",
    ///     "name": "{origin_name}"
    ///   }}
    /// }}"#);
    ///
    /// let actor = Object::new_inner().with_kind(actor_type).with_name(actor_name);
    /// let object = Person::new_inner().with_name(object_name);
    /// let origin = Group::new_inner().with_name(origin_name);
    /// let remove = Remove::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object)
    ///     .with_origin(origin);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&remove).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Remove>(json_str.as_str()).unwrap(),
    ///     remove
    /// );
    /// # }
    /// ```
    Remove {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Group, Name, Object, Person, VocabularyType};

    #[test]
    fn test_activity() {
        let summary = "The moderator removed Sally from a group";

        let actor_type = VocabularyType::iri("http://example.org/Role").unwrap();
        let actor_name = Name::try_from("The Moderator").unwrap();

        let object_name = Name::try_from("Sally").unwrap();
        let origin_name = Name::try_from("A Simple Group").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Remove",
  "summary": "{summary}",
  "actor": {{
    "type": "{actor_type}",
    "name": "{actor_name}"
  }},
  "object": {{
    "type": "Person",
    "name": "{object_name}"
  }},
  "origin": {{
    "type": "Group",
    "name": "{origin_name}"
  }}
}}"#
        );

        let actor = Object::new_inner()
            .with_kind(actor_type)
            .with_name(actor_name);
        let object = Person::new_inner().with_name(object_name);
        let origin = Group::new_inner().with_name(origin_name);
        let remove = Remove::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object)
            .with_origin(origin);

        assert_eq!(serde_json::to_string_pretty(&remove).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Remove>(json_str.as_str()).unwrap(),
            remove
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Remove>(json_str).is_err());
    }
}
