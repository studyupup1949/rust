use crate::{ActivityType, create_activity};

create_activity! {
    /// Indicates that the `actor` has moved `object` from `origin` to `target`.
    ///
    /// If the `origin` or `target` are not specified, either can be determined by context.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activitystreams_vocabulary::{Collection, Iri, Move, Name, Person};
    ///
    /// # fn main() {
    /// let summary = "Sally moved a post from List A to List B";
    /// let actor_name = Name::try_from("Sally").unwrap();
    /// let object = Iri::try_from("http://example.org/posts/1").unwrap();
    /// let target_name = Name::try_from("List B").unwrap();
    /// let origin_name = Name::try_from("List A").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": "https://www.w3.org/ns/activitystreams",
    ///   "type": "Move",
    ///   "summary": "{summary}",
    ///   "actor": {{
    ///     "type": "Person",
    ///     "name": "{actor_name}"
    ///   }},
    ///   "object": "{object}",
    ///   "origin": {{
    ///     "type": "Collection",
    ///     "name": "{origin_name}"
    ///   }},
    ///   "target": {{
    ///     "type": "Collection",
    ///     "name": "{target_name}"
    ///   }}
    /// }}"#);
    ///
    /// let actor = Person::new_inner().with_name(actor_name);
    /// let target = Collection::new_inner().with_name(target_name);
    /// let origin = Collection::new_inner().with_name(origin_name);
    ///
    /// let move_act = Move::new()
    ///     .with_summary(summary)
    ///     .with_actor(actor)
    ///     .with_object(object)
    ///     .with_origin(origin)
    ///     .with_target(target);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&move_act).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Move>(json_str.as_str()).unwrap(),
    ///     move_act
    /// );
    /// # }
    /// ```
    Move {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Collection, Iri, Name, Person};

    #[test]
    fn test_activity() {
        let summary = "Sally moved a post from List A to List B";
        let actor_name = Name::try_from("Sally").unwrap();
        let object = Iri::try_from("http://example.org/posts/1").unwrap();
        let target_name = Name::try_from("List B").unwrap();
        let origin_name = Name::try_from("List A").unwrap();

        let json_str = format!(
            r#"{{
  "@context": "https://www.w3.org/ns/activitystreams",
  "type": "Move",
  "summary": "{summary}",
  "actor": {{
    "type": "Person",
    "name": "{actor_name}"
  }},
  "object": "{object}",
  "origin": {{
    "type": "Collection",
    "name": "{origin_name}"
  }},
  "target": {{
    "type": "Collection",
    "name": "{target_name}"
  }}
}}"#
        );

        let actor = Person::new_inner().with_name(actor_name);
        let target = Collection::new_inner().with_name(target_name);
        let origin = Collection::new_inner().with_name(origin_name);

        let move_act = Move::new()
            .with_summary(summary)
            .with_actor(actor)
            .with_object(object)
            .with_origin(origin)
            .with_target(target);

        assert_eq!(serde_json::to_string_pretty(&move_act).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Move>(json_str.as_str()).unwrap(),
            move_act
        );
    }

    #[test]
    fn test_invalid_activity() {
        let json_str = r#"{{"@context":"https://www.w3.org/ns/activitystreams","type":"Object"}}"#;

        assert!(serde_json::from_str::<Move>(json_str).is_err());
    }
}
