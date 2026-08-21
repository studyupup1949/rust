use crate::{CoreType, create_intransitive_activity};

create_intransitive_activity! {
    /// Instances of [IntransitiveActivity] are a subtype of [Activity](crate::Activity) representing intransitive actions.
    ///
    /// The `object` property is therefore inappropriate for these activities.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{IntransitiveActivity, Name, Person, Place};
    ///
    /// # fn main() {
    /// let summary = "Sally went to work";
    ///
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let actor = Person::new_inner().with_name(actor_name.clone());
    ///
    /// let target_name = Name::try_from("Work").unwrap();
    /// let target = Place::new_inner().with_name(target_name.clone());
    ///
    /// let activity = IntransitiveActivity::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_target(target);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "IntransitiveActivity",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "target": {{
    ///     "type": "Place",
    ///     "name": "{target_name}"
    ///   }}
    /// }}"#);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&activity).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<IntransitiveActivity>(json_str.as_str()).unwrap(),
    ///     activity
    /// );
    /// # }
    /// ```
    IntransitiveActivity: CoreType {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Person, Place};

    #[test]
    fn test_activity() {
        let summary = "Sally went to work";

        let actor_name = Name::try_from("Sally").unwrap();
        let actor = Person::new_inner().with_name(actor_name.clone());

        let target_name = Name::try_from("Work").unwrap();
        let target = Place::new_inner().with_name(target_name.clone());

        let activity = IntransitiveActivity::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_target(target);

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "IntransitiveActivity",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "target": {{
    "type": "Place",
    "name": "{target_name}"
  }}
}}"#
        );

        assert_eq!(serde_json::to_string_pretty(&activity).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<IntransitiveActivity>(json_str.as_str()).unwrap(),
            activity
        );
    }

    #[test]
    fn test_invalid_intransitive_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<IntransitiveActivity>(json_str).is_err());
    }
}
