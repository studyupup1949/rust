use crate::create_activity;

create_activity! {
    /// Indicates that the `actor` is offering the `object`.
    ///
    /// If specified, the `target` indicates the entity to which the `object` is being offered.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Name, Object, Offer, Person, VocabularyType};
    ///
    /// # fn main() {
    /// let summary = "Sally offered 50% off to Lewis";
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let object_kind = VocabularyType::iri("http://www.types.example/ProductOffer").unwrap();
    /// let object_name = Name::try_from("50% Off!").unwrap();
    /// let target_name = Name::try_from("Lewis").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Offer",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": {{
    ///     "type": "{object_kind}",
    ///     "name": "{object_name}"
    ///   }},
    ///   "target": {{
    ///     "type": "Person",
    ///     "name": "{target_name}"
    ///   }}
    /// }}"#);
    ///
    /// let actor = Person::new_inner().with_name(actor_name);
    /// let object = Object::new_inner()
    ///     .with_kind(object_kind)
    ///     .with_name(object_name);
    /// let target = Person::new_inner().with_name(target_name);
    /// let offer = Offer::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object)
    ///     .with_target(target);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&offer).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Offer>(json_str.as_str()).unwrap(),
    ///     offer
    /// );
    /// # }
    /// ```
    Offer {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Name, Object, Person, VocabularyType};

    #[test]
    fn test_activity() {
        let summary = "Sally offered 50% off to Lewis";
        let actor_name = Name::try_from("Sally").unwrap();
        let object_kind = VocabularyType::iri("http://www.types.example/ProductOffer").unwrap();
        let object_name = Name::try_from("50% Off!").unwrap();
        let target_name = Name::try_from("Lewis").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Offer",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": {{
    "type": "{object_kind}",
    "name": "{object_name}"
  }},
  "target": {{
    "type": "Person",
    "name": "{target_name}"
  }}
}}"#
        );

        let actor = Person::new_inner().with_name(actor_name);
        let object = Object::new_inner()
            .with_kind(object_kind)
            .with_name(object_name);
        let target = Person::new_inner().with_name(target_name);
        let offer = Offer::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object)
            .with_target(target);

        assert_eq!(serde_json::to_string_pretty(&offer).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Offer>(json_str.as_str()).unwrap(),
            offer
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Offer>(json_str).is_err());
    }
}
