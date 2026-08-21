use crate::create_activity;

create_activity! {
    ///	Indicates that the `actor` has joined the `object`.
    ///
    ///	The `target` and `origin` typically have no defined meaning.
    ///
    ///	# Example
    ///
    ///	```rust
    /// use activitystreams_vocabulary::{Group, Join, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally joined a group";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let object_name = Name::try_from("A Simple Group").unwrap();
    /// let object = Group::new_inner().with_name(object_name.clone());
    ///
    /// let join = Join::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Join",
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
    /// assert_eq!(serde_json::to_string_pretty(&join).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Join>(json_str.as_str()).unwrap(),
    ///     join
    /// );
    /// # }
    ///	```
    Join {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Group, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally joined a group";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let object_name = Name::try_from("A Simple Group").unwrap();
        let object = Group::new_inner().with_name(object_name.clone());

        let join = Join::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Join",
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

        assert_eq!(serde_json::to_string_pretty(&join).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Join>(json_str.as_str()).unwrap(),
            join
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Join>(json_str).is_err());
    }
}
