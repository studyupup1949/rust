use activitystreams_vocabulary::create_activity;

create_activity! {
    /// Indicates that the `actor` is canceling `target`’s access to a resource specified by `context` under the role specified by instrument, making the [Grant](crate::Grant) activities specified by `object` unusable anymore in other activities' `capability` field.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Revoke, context};
    /// use activitystreams_vocabulary::Iri;
    ///
    /// # fn main() {
    /// let id = Iri::try_from("https://example.dev/myproject/outbox/nlTxb").unwrap();
    /// let actor = Iri::try_from("http://example.dev/myproject").unwrap();
    ///
    /// let object = Iri::try_from("https://example.dev/myproject/outbox/reBGo").unwrap();
    /// let context = Iri::try_from("https://example.dev/myproject").unwrap();
    /// let instrument = Iri::try_from("https://example.dev/roles/developer").unwrap();
    /// let target = Iri::try_from("https://example.dev/users/aviva").unwrap();
    ///
    /// let to0 = Iri::try_from("https://example.dev/myproject/followers").unwrap();
    /// let to1 = Iri::try_from("https://example.dev/users/aviva").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Revoke",
    ///   "id": "{id}",
    ///   "context": "{context}",
    ///   "to": [
    ///     "{to0}",
    ///     "{to1}"
    ///   ],
    ///   "actor": "{actor}",
    ///   "object": "{object}",
    ///   "target": "{target}",
    ///   "instrument": "{instrument}"
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let to = [to0, to1];
    ///
    /// let revoke = Revoke::new()
    ///     .with_context_property(context_property)
    ///     .with_id(id)
    ///     .with_actor(actor)
    ///     .with_object(object)
    ///     .with_instrument(instrument)
    ///     .with_context(context)
    ///     .with_target(target)
    ///     .with_to(to);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&revoke).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Revoke>(json_str.as_str()).unwrap(),
    ///     revoke
    /// );
    /// # }
    Revoke: crate::ActivityType::Revoke {}
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;
    use activitystreams_vocabulary::Iri;

    #[test]
    fn test_revoke() {
        let id = Iri::try_from("https://example.dev/myproject/outbox/nlTxb").unwrap();
        let actor = Iri::try_from("http://example.dev/myproject").unwrap();

        let object = Iri::try_from("https://example.dev/myproject/outbox/reBGo").unwrap();
        let context = Iri::try_from("https://example.dev/myproject").unwrap();
        let instrument = Iri::try_from("https://example.dev/roles/developer").unwrap();
        let target = Iri::try_from("https://example.dev/users/aviva").unwrap();

        let to0 = Iri::try_from("https://example.dev/myproject/followers").unwrap();
        let to1 = Iri::try_from("https://example.dev/users/aviva").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Revoke",
  "id": "{id}",
  "context": "{context}",
  "to": [
    "{to0}",
    "{to1}"
  ],
  "actor": "{actor}",
  "object": "{object}",
  "target": "{target}",
  "instrument": "{instrument}"
}}"#
        );

        let context_property = context::forgefed_context();

        let to = [to0, to1];

        let revoke = Revoke::new()
            .with_context_property(context_property)
            .with_id(id)
            .with_actor(actor)
            .with_object(object)
            .with_instrument(instrument)
            .with_context(context)
            .with_target(target)
            .with_to(to);

        assert_eq!(serde_json::to_string_pretty(&revoke).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Revoke>(json_str.as_str()).unwrap(),
            revoke
        );
    }
}
