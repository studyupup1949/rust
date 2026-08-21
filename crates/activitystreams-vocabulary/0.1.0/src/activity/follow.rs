use crate::{ActivityType, create_activity};

create_activity! {
    /// Indicates that the `actor` is "following" the `object`.
    ///
    /// Following is defined in the sense typically used within Social systems in which the actor is interested in any activity performed by or on the object.
    ///
    /// The `target` and origin typically have no defined meaning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Follow, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally followed John";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let object_name = Name::try_from("John").unwrap();
    /// let object = Person::new_inner().with_name(object_name.clone());
    ///
    /// let follow = Follow::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Follow",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": {{
    ///     "type": "Person",
    ///     "name": "{object_name}"
    ///   }}
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&follow).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Follow>(json_str.as_str()).unwrap(),
    ///     follow
    /// );
    /// # }
    /// ```
    Follow {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally followed John";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let object_name = Name::try_from("John").unwrap();
        let object = Person::new_inner().with_name(object_name.clone());

        let follow = Follow::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Follow",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": {{
    "type": "Person",
    "name": "{object_name}"
  }}
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&follow).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Follow>(json_str.as_str()).unwrap(),
            follow
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Follow>(json_str).is_err());
    }
}
