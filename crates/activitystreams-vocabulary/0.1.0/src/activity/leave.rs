use crate::{ActivityType, create_activity};

create_activity! {
    /// Indicates that the `actor` has left the `object`.
    ///
    /// The `target` and `origin` typically have no meaning.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Group, Leave, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally left a group";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let object_name = Name::try_from("A Simple Group").unwrap();
    /// let object = Group::new_inner().with_name(object_name.clone());
    ///
    /// let leave = Leave::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Leave",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": {{
    ///     "type": "Group",
    ///     "name": "{object_name}"
    ///   }}
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&leave).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Leave>(json_str.as_str()).unwrap(),
    ///     leave
    /// );
    /// # }
    /// ```
    Leave {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Group, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally left a group";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let object_name = Name::try_from("A Simple Group").unwrap();
        let object = Group::new_inner().with_name(object_name.clone());

        let leave = Leave::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Leave",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": {{
    "type": "Group",
    "name": "{object_name}"
  }}
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&leave).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Leave>(json_str.as_str()).unwrap(),
            leave
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Leave>(json_str).is_err());
    }
}
