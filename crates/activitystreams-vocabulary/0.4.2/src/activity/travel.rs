use crate::create_intransitive_activity;

create_intransitive_activity! {
    ///	Indicates that the `actor` is traveling to `target` from `origin`.
    ///
    ///	Travel is an [IntransitiveActivity](crate::IntransitiveActivity) whose `actor` specifies the direct `object`.
    ///
    ///	If the `target` or `origin` are not specified, either can be determined by context.
    ///
    ///	# Example
    ///
    ///	```rust
    /// use activitystreams_vocabulary::{Name, Person, Place, Travel};
    ///
    /// # fn main() {
    /// let summary = "Sally went home from work";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let target_name = Name::try_from("Home").unwrap();
    /// let target = Place::new_inner().with_name(target_name.clone());
    ///
    /// let origin_name = Name::try_from("Work").unwrap();
    /// let origin = Place::new_inner().with_name(origin_name.clone());
    ///
    /// let activity = Travel::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_target(target)
    ///     .with_origin(origin);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Travel",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "target": {{
    ///     "type": "Place",
    ///     "name": "{target_name}"
    ///   }},
    ///   "origin": {{
    ///     "type": "Place",
    ///     "name": "{origin_name}"
    ///   }}
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&activity).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Travel>(json_str.as_str()).unwrap(),
    ///     activity
    /// );
    /// # }
    ///	```
    Travel {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Person, Place};

    #[test]
    fn test_activity() {
        let summary = "Sally went home from work";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let target_name = Name::try_from("Home").unwrap();
        let target = Place::new_inner().with_name(target_name.clone());

        let origin_name = Name::try_from("Work").unwrap();
        let origin = Place::new_inner().with_name(origin_name.clone());

        let activity = Travel::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_target(target)
            .with_origin(origin);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Travel",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "target": {{
    "type": "Place",
    "name": "{target_name}"
  }},
  "origin": {{
    "type": "Place",
    "name": "{origin_name}"
  }}
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&activity).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Travel>(json_str.as_str()).unwrap(),
            activity
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Travel>(json_str).is_err());
    }
}
