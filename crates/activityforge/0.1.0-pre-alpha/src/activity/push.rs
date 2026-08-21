use activitystreams_vocabulary::create_activity;

create_activity! {
    /// Indicates that new content has been pushed to the [Repository](crate::Repository).
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Commit, Push, Sha1Hash, context};
    /// use activitystreams_vocabulary::{DateTime, Iri, OrderedCollection};
    ///
    /// # fn main() {
    /// let id = Iri::try_from("https://example.dev/aviva/myproject/outbox/reBGo").unwrap();
    /// let actor = Iri::try_from("https://example.dev/aviva/myproject").unwrap();
    /// let attributed_to = Iri::try_from("https://example.dev/aviva").unwrap();
    ///
    /// let to0 = Iri::try_from("https://example.dev/aviva").unwrap();
    /// let to1 = Iri::try_from("https://example.dev/aviva/followers").unwrap();
    /// let to2 = Iri::try_from("https://example.dev/aviva/myproject/followers").unwrap();
    ///
    /// let summary = "<p>Aviva pushed a commit to myproject</p>";
    ///
    /// let commit_id = Iri::try_from("https://example.dev/aviva/myproject/commits/d96596230322716bd6f87a232a648ca9822a1c20").unwrap();
    /// let commit_attributed_to = Iri::try_from("https://example.dev/aviva").unwrap();
    /// let commit_context = Iri::try_from("https://example.dev/aviva/myproject").unwrap();
    /// let commit_hash = Sha1Hash::try_from("d96596230322716bd6f87a232a648ca9822a1c20").unwrap();
    /// let commit_created = "2019-11-03T13:43:59Z";
    /// let commit_summary = "Provide hints in sign-up form fields";
    ///
    /// let target = Iri::try_from("https://example.dev/aviva/myproject/branches/master").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Push",
    ///   "id": "{id}",
    ///   "attributedTo": "{attributed_to}",
    ///   "summary": "{summary}",
    ///   "to": [
    ///     "{to0}",
    ///     "{to1}",
    ///     "{to2}"
    ///   ],
    ///   "actor": "{actor}",
    ///   "object": {{
    ///     "type": "OrderedCollection",
    ///     "orderedItems": [
    ///       {{
    ///         "type": "Commit",
    ///         "id": "{commit_id}",
    ///         "attributedTo": "{commit_attributed_to}",
    ///         "summary": "{commit_summary}",
    ///         "context": "{commit_context}",
    ///         "created": "{commit_created}",
    ///         "hash": "{commit_hash}"
    ///       }}
    ///     ],
    ///     "totalItems": 1
    ///   }},
    ///   "target": "{target}"
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let commit = Commit::new_inner()
    ///     .with_id(commit_id)
    ///     .with_attributed_to(commit_attributed_to)
    ///     .with_context(commit_context)
    ///     .with_hash(commit_hash)
    ///     .with_created(commit_created.parse::<DateTime>().unwrap())
    ///     .with_summary(commit_summary);
    ///
    /// let items = [commit];
    ///
    /// let object = OrderedCollection::new_inner()
    ///     .with_total_items(items.len() as u64)
    ///     .with_ordered_items(items);
    ///
    /// let push = Push::new()
    ///     .with_context_property(context_property)
    ///     .with_id(id)
    ///     .with_actor(actor)
    ///     .with_attributed_to(attributed_to)
    ///     .with_to([to0, to1, to2])
    ///     .with_summary(summary)
    ///     .with_object(object)
    ///     .with_target(target);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&push).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Push>(json_str.as_str()).unwrap(),
    ///     push
    /// );
    /// # }
    /// ```
    Push: crate::ActivityType::Push {}
}

#[cfg(test)]
mod tests {
    use activitystreams_vocabulary::{DateTime, Iri, OrderedCollection};

    use super::*;
    use crate::{Commit, Sha1Hash, context};

    #[test]
    fn test_push() {
        let id = Iri::try_from("https://example.dev/aviva/myproject/outbox/reBGo").unwrap();
        let actor = Iri::try_from("https://example.dev/aviva/myproject").unwrap();
        let attributed_to = Iri::try_from("https://example.dev/aviva").unwrap();

        let to0 = Iri::try_from("https://example.dev/aviva").unwrap();
        let to1 = Iri::try_from("https://example.dev/aviva/followers").unwrap();
        let to2 = Iri::try_from("https://example.dev/aviva/myproject/followers").unwrap();

        let summary = "<p>Aviva pushed a commit to myproject</p>";

        let commit_id = Iri::try_from(
            "https://example.dev/aviva/myproject/commits/d96596230322716bd6f87a232a648ca9822a1c20",
        )
        .unwrap();
        let commit_attributed_to = Iri::try_from("https://example.dev/aviva").unwrap();
        let commit_context = Iri::try_from("https://example.dev/aviva/myproject").unwrap();
        let commit_hash = Sha1Hash::try_from("d96596230322716bd6f87a232a648ca9822a1c20").unwrap();
        let commit_created = "2019-11-03T13:43:59Z";
        let commit_summary = "Provide hints in sign-up form fields";

        let target = Iri::try_from("https://example.dev/aviva/myproject/branches/master").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Push",
  "id": "{id}",
  "attributedTo": "{attributed_to}",
  "summary": "{summary}",
  "to": [
    "{to0}",
    "{to1}",
    "{to2}"
  ],
  "actor": "{actor}",
  "object": {{
    "type": "OrderedCollection",
    "orderedItems": [
      {{
        "type": "Commit",
        "id": "{commit_id}",
        "attributedTo": "{commit_attributed_to}",
        "summary": "{commit_summary}",
        "context": "{commit_context}",
        "created": "{commit_created}",
        "hash": "{commit_hash}"
      }}
    ],
    "totalItems": 1
  }},
  "target": "{target}"
}}"#
        );

        let context_property = context::forgefed_context();

        let commit = Commit::new_inner()
            .with_id(commit_id)
            .with_attributed_to(commit_attributed_to)
            .with_context(commit_context)
            .with_hash(commit_hash)
            .with_created(commit_created.parse::<DateTime>().unwrap())
            .with_summary(commit_summary);

        let items = [commit];

        let object = OrderedCollection::new_inner()
            .with_total_items(items.len() as u64)
            .with_ordered_items(items);

        let push = Push::new()
            .with_context_property(context_property)
            .with_id(id)
            .with_actor(actor)
            .with_attributed_to(attributed_to)
            .with_to([to0, to1, to2])
            .with_summary(summary)
            .with_object(object)
            .with_target(target);

        assert_eq!(serde_json::to_string_pretty(&push).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Push>(json_str.as_str()).unwrap(),
            push
        );
    }
}
