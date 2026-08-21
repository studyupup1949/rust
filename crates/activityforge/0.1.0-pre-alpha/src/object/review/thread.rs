use activitystreams_vocabulary::{DateTime, ObjectItems, create_list, create_object, field_access};

create_object! {
    /// A comment on a code change and the discussion on it.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{ReviewThread, context};
    /// use activitystreams_vocabulary::{DateTime, Iri};
    ///
    /// # fn main() {
    /// let context = Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1").unwrap();
    /// let target =
    ///     Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1/quote/1").unwrap();
    /// let object =
    ///     Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1/comment/1").unwrap();
    /// let is_resolved = true;
    /// let resolved_by = Iri::try_from("https://example.dev/alice").unwrap();
    /// let resolved = "2026-07-10T13:37:42Z";
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "ReviewThread",
    ///   "context": "{context}",
    ///   "isResolved": {is_resolved},
    ///   "resolvedBy": "{resolved_by}",
    ///   "resolved": "{resolved}",
    ///   "object": "{object}",
    ///   "target": "{target}"
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let review_thread = ReviewThread::new()
    ///     .with_context_property(context_property)
    ///     .with_context(context)
    ///     .with_object(object)
    ///     .with_target(target)
    ///     .with_is_resolved(is_resolved)
    ///     .with_resolved_by(resolved_by)
    ///     .with_resolved(resolved.parse::<DateTime>().unwrap());
    ///
    /// assert_eq!(
    ///     serde_json::to_string_pretty(&review_thread).unwrap(),
    ///     json_str
    /// );
    /// assert_eq!(
    ///     serde_json::from_str::<ReviewThread>(json_str.as_str()).unwrap(),
    ///     review_thread
    /// );
    /// # }
    /// ```
    ReviewThread: crate::ObjectType::ReviewThread {
        #[serde(skip_serializing_if = "Option::is_none")]
        is_resolved: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        resolved_by: Option<ObjectItems>,
        #[serde(skip_serializing_if = "Option::is_none")]
        resolved: Option<DateTime>,
        #[serde(skip_serializing_if = "Option::is_none")]
        object: Option<ObjectItems>,
        #[serde(skip_serializing_if = "Option::is_none")]
        target: Option<ObjectItems>,
    }
}

field_access! {
    ReviewThread {
        /// Specifies whether the [ReviewThread] is closed.
        is_resolved: option { bool },
    }
}

field_access! {
    ReviewThread {
        /// Identifies the Actor who has resolved the [ReviewThread], or the activity that has resolved the [ReviewThread].
        resolved_by: option_ref { ObjectItems },
        /// For a resolved [ReviewThread], specifies the time the [ReviewThread] has been resolved.
        resolved: option_ref { DateTime },
        /// The [Note](activitystreams_vocabulary::Note) that is the top comment of this thread.
        object: option_ref { ObjectItems },
        /// The [CodeQuote](crate::CodeQuote) being commented on.
        target: option_ref { ObjectItems },
    }
}

create_list! {
    /// Represents a singular or list variant of [ReviewThread]s.
    ReviewThreads: boxed { ReviewThread },
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::{DateTime, Iri};

    #[test]
    fn test_review_thread() {
        let context = Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1").unwrap();
        let target =
            Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1/quote/1").unwrap();
        let object =
            Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1/comment/1").unwrap();
        let is_resolved = true;
        let resolved_by = Iri::try_from("https://example.dev/alice").unwrap();
        let resolved = "2026-07-10T13:37:42Z";

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "ReviewThread",
  "context": "{context}",
  "isResolved": {is_resolved},
  "resolvedBy": "{resolved_by}",
  "resolved": "{resolved}",
  "object": "{object}",
  "target": "{target}"
}}"#
        );

        let context_property = context::forgefed_context();

        let review_thread = ReviewThread::new()
            .with_context_property(context_property)
            .with_context(context)
            .with_object(object)
            .with_target(target)
            .with_is_resolved(is_resolved)
            .with_resolved_by(resolved_by)
            .with_resolved(resolved.parse::<DateTime>().unwrap());

        assert_eq!(
            serde_json::to_string_pretty(&review_thread).unwrap(),
            json_str
        );
        assert_eq!(
            serde_json::from_str::<ReviewThread>(json_str.as_str()).unwrap(),
            review_thread
        );
    }
}
