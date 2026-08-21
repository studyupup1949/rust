use activitystreams_vocabulary::create_activity;

create_activity! {
    /// Indicates that a [Ticket](crate::Ticket) is being assigned to a [Person](activitystreams_vocabulary::Person) or [Team](crate::Team).
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Assign, context};
    /// use activitystreams_vocabulary::Iri;
    ///
    /// # fn main() {
    /// let id = Iri::try_from("https://example.dev/aviva/myproject/outbox/reBGo").unwrap();
    /// let object = Iri::try_from("https://example.dev/aviva/myproject/issues/1").unwrap();
    /// let to0 = Iri::try_from("https://example.dev/bob").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Assign",
    ///   "id": "{id}",
    ///   "to": [
    ///     "{to0}"
    ///   ],
    ///   "object": "{object}"
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let assign = Assign::new()
    ///     .with_context_property(context_property)
    ///     .with_id(id)
    ///     .with_to([to0])
    ///     .with_object(object);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&assign).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Assign>(json_str.as_str()).unwrap(),
    ///     assign
    /// );
    /// # }
    /// ```
    Assign: crate::ActivityType::Assign {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::Iri;

    #[test]
    fn test_assign() {
        let id = Iri::try_from("https://example.dev/aviva/myproject/outbox/reBGo").unwrap();
        let object = Iri::try_from("https://example.dev/aviva/myproject/issues/1").unwrap();
        let to0 = Iri::try_from("https://example.dev/bob").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Assign",
  "id": "{id}",
  "to": [
    "{to0}"
  ],
  "object": "{object}"
}}"#
        );

        let context_property = context::forgefed_context();

        let assign = Assign::new()
            .with_context_property(context_property)
            .with_id(id)
            .with_to([to0])
            .with_object(object);

        assert_eq!(serde_json::to_string_pretty(&assign).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Assign>(json_str.as_str()).unwrap(),
            assign
        );
    }
}
