use activitystreams_vocabulary::{ObjectItems, create_object, field_access};

mod approval;
mod status;
mod thread;
mod verdict;

pub use approval::Approval;
pub use status::ReviewStatus;
pub use thread::{ReviewThread, ReviewThreads};
pub use verdict::ReviewVerdict;

create_object! {
    /// Describes a review on a merge request.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Review, ReviewThread, ReviewVerdict, context};
    /// use activitystreams_vocabulary::{DateTime, Iri};
    ///
    /// # fn main() {
    /// let object = Iri::try_from("https://example.dev/alice/myrepo/pulls/1/patch/1").unwrap();
    /// let verdict = ReviewVerdict::Approve;
    /// let review_is_binding = true;
    ///
    /// let thread_context =
    ///     Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1").unwrap();
    /// let thread_target =
    ///     Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1/quote/1").unwrap();
    /// let thread_object =
    ///     Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1/comment/1").unwrap();
    /// let thread_is_resolved = true;
    /// let thread_resolved_by = Iri::try_from("https://example.dev/alice").unwrap();
    /// let thread_resolved = "2026-07-10T13:37:42Z";
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Review",
    ///   "object": "{object}",
    ///   "verdict": "{verdict}",
    ///   "reviewIsBinding": {review_is_binding},
    ///   "reviewThreads": [
    ///     {{
    ///       "type": "ReviewThread",
    ///       "context": "{thread_context}",
    ///       "isResolved": {thread_is_resolved},
    ///       "resolvedBy": "{thread_resolved_by}",
    ///       "resolved": "{thread_resolved}",
    ///       "object": "{thread_object}",
    ///       "target": "{thread_target}"
    ///     }}
    ///   ]
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let review_thread = ReviewThread::new_inner()
    ///     .with_context(thread_context)
    ///     .with_object(thread_object)
    ///     .with_target(thread_target)
    ///     .with_is_resolved(thread_is_resolved)
    ///     .with_resolved_by(thread_resolved_by)
    ///     .with_resolved(thread_resolved.parse::<DateTime>().unwrap());
    ///
    /// let review = Review::new()
    ///     .with_context_property(context_property)
    ///     .with_object(object)
    ///     .with_verdict(verdict)
    ///     .with_review_is_binding(review_is_binding)
    ///     .with_review_threads([review_thread]);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&review).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Review>(json_str.as_str()).unwrap(),
    ///     review
    /// );
    /// # }
    /// ```
    Review: crate::ObjectType::Review {
        #[serde(skip_serializing_if = "Option::is_none")]
        object: Option<ObjectItems>,
        #[serde(skip_serializing_if = "Option::is_none")]
        verdict: Option<ReviewVerdict>,
        #[serde(skip_serializing_if = "Option::is_none")]
        review_is_binding: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        review_threads: Option<ReviewThreads>,
    }
}

field_access! {
    Review {
        /// URI of an [OrderedCollection](activitystreams_vocabulary::OrderedCollection) of [Patche](crate::Patch)s, representing the version of the merge request, on which the review was made.
        ///
        /// That collection’s `target` can be used to determine the commit hash, i.e. the repository tree state on which the review was made.
        object: option_ref { ObjectItems },
        /// Zero or more [ReviewThread]s.
        review_threads: option_ref { ReviewThreads },
    }
}

field_access! {
    Review {
        /// The level of approval that the review gives on the merge request.
        verdict: option { ReviewVerdict },
        /// Whether the review is made by someone with `write` or higher access to the [PatchTracker](crate::PatchTracker), i.e. someone with access to apply a merge request.
        review_is_binding: option { bool },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::{DateTime, Iri};

    #[test]
    fn test_review_thread() {
        let object = Iri::try_from("https://example.dev/alice/myrepo/pulls/1/patch/1").unwrap();
        let verdict = ReviewVerdict::Approve;
        let review_is_binding = true;

        let thread_context =
            Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1").unwrap();
        let thread_target =
            Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1/quote/1").unwrap();
        let thread_object =
            Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1/comment/1").unwrap();
        let thread_is_resolved = true;
        let thread_resolved_by = Iri::try_from("https://example.dev/alice").unwrap();
        let thread_resolved = "2026-07-10T13:37:42Z";

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Review",
  "object": "{object}",
  "verdict": "{verdict}",
  "reviewIsBinding": {review_is_binding},
  "reviewThreads": [
    {{
      "type": "ReviewThread",
      "context": "{thread_context}",
      "isResolved": {thread_is_resolved},
      "resolvedBy": "{thread_resolved_by}",
      "resolved": "{thread_resolved}",
      "object": "{thread_object}",
      "target": "{thread_target}"
    }}
  ]
}}"#
        );

        let context_property = context::forgefed_context();

        let review_thread = ReviewThread::new_inner()
            .with_context(thread_context)
            .with_object(thread_object)
            .with_target(thread_target)
            .with_is_resolved(thread_is_resolved)
            .with_resolved_by(thread_resolved_by)
            .with_resolved(thread_resolved.parse::<DateTime>().unwrap());

        let review = Review::new()
            .with_context_property(context_property)
            .with_object(object)
            .with_verdict(verdict)
            .with_review_is_binding(review_is_binding)
            .with_review_threads([review_thread]);

        assert_eq!(serde_json::to_string_pretty(&review).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Review>(json_str.as_str()).unwrap(),
            review
        );
    }
}
