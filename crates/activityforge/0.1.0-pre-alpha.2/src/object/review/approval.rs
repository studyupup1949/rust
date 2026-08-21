use activitystreams_vocabulary::{create_object, field_access};

use crate::ReviewStatus;

create_object! {
    /// Refers to either the old or new version of a file being changed.
    ///
    /// # Example (individual approval)
    ///
    /// ```rust
    /// use activityforge::{Approval, ReviewStatus, context};
    /// use activitystreams_vocabulary::{DateTime, Iri};
    ///
    /// # fn main() {
    /// let attributed_to = Iri::try_from("https://example.dev/alice").unwrap();
    /// let published = "2026-04-20T13:37:42Z";
    /// let approval_status = ReviewStatus::Approve;
    /// let review_is_requested = false;
    /// let review_is_by_code_owner = true;
    /// let review_counts_in_rule = true;
    /// let new_review_in_progress = false;
    /// let attachment =
    ///     Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Approval",
    ///   "attachment": "{attachment}",
    ///   "attributedTo": "{attributed_to}",
    ///   "published": "{published}",
    ///   "approvalStatus": "{approval_status}",
    ///   "reviewIsRequested": {review_is_requested},
    ///   "reviewIsByCodeOwner": {review_is_by_code_owner},
    ///   "reviewCountsInRule": {review_counts_in_rule},
    ///   "newReviewInProgress": {new_review_in_progress}
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let approval = Approval::new()
    ///     .with_context_property(context_property)
    ///     .with_attributed_to(attributed_to)
    ///     .with_published(published.parse::<DateTime>().unwrap())
    ///     .with_approval_status(approval_status)
    ///     .with_review_is_requested(review_is_requested)
    ///     .with_review_is_by_code_owner(review_is_by_code_owner)
    ///     .with_review_counts_in_rule(review_counts_in_rule)
    ///     .with_new_review_in_progress(new_review_in_progress)
    ///     .with_attachment(attachment);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&approval).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Approval>(json_str.as_str()).unwrap(),
    ///     approval
    /// );
    /// # }
    /// ```
    ///
    /// # Example (team approval)
    ///
    /// ```rust
    /// use activityforge::{Approval, ReviewStatus, context};
    /// use activitystreams_vocabulary::{DateTime, Iri};
    ///
    /// # fn main() {
    /// let attributed_to = Iri::try_from("https://example.dev/alice_team").unwrap();
    /// let published = "2026-04-20T13:37:42Z";
    /// let approval_status = ReviewStatus::Approve;
    /// let review_is_requested = false;
    /// let review_is_by_code_owner = true;
    /// let review_counts_in_rule = true;
    /// let approval_id =
    ///     Iri::try_from("https://example.dev/alice/myrepo/pulls/1/approval/1").unwrap();
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Approval",
    ///   "attachment": [
    ///     {{
    ///       "type": "Approval",
    ///       "id": "{approval_id}"
    ///     }}
    ///   ],
    ///   "attributedTo": "{attributed_to}",
    ///   "published": "{published}",
    ///   "approvalStatus": "{approval_status}",
    ///   "reviewIsRequested": {review_is_requested},
    ///   "reviewIsByCodeOwner": {review_is_by_code_owner},
    ///   "reviewCountsInRule": {review_counts_in_rule}
    /// }}"#
    ///         );
    ///
    /// let context_property = context::forgefed_context();
    ///
    /// let individual_approval = Approval::new_inner().with_id(approval_id);
    /// let attachment = [individual_approval];
    ///
    /// let approval = Approval::new()
    ///     .with_context_property(context_property)
    ///     .with_attributed_to(attributed_to)
    ///     .with_published(published.parse::<DateTime>().unwrap())
    ///     .with_approval_status(approval_status)
    ///     .with_review_is_requested(review_is_requested)
    ///     .with_review_is_by_code_owner(review_is_by_code_owner)
    ///     .with_review_counts_in_rule(review_counts_in_rule)
    ///     .with_attachment(attachment);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&approval).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Approval>(json_str.as_str()).unwrap(),
    ///     approval
    /// );
    /// # }
    /// ```
    Approval: crate::ObjectType::Approval {
        #[serde(skip_serializing_if = "Option::is_none")]
        approval_status: Option<ReviewStatus>,
        #[serde(skip_serializing_if = "Option::is_none")]
        review_is_requested: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        review_is_by_code_owner: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        review_counts_in_rule: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        new_review_in_progress: Option<bool>,
    }
}

field_access! {
    Approval {
        /// The [ReviewStatus] of this approval - note that values `remark` and `revise` **MUST NOT** be used.
        approval_status: option { ReviewStatus },
        /// Whether a review from this team was specifically personally requested/invited, or was is given without a personal request.
        review_is_requested: option { bool },
        /// Whether this review/request is here because (perhaps among other reasons) this team is a Code Owner of some of the files being changed by the merge request.
        review_is_by_code_owner: option { bool },
        /// Whether this approval count towards required amount of approvals under some rule.
        review_counts_in_rule: option { bool },
        /// Whether the person has indicated they’re in the process of making a new review.
        new_review_in_progress: option { bool },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::{DateTime, Iri};

    #[test]
    fn test_approval_individual() {
        let attributed_to = Iri::try_from("https://example.dev/alice").unwrap();
        let published = "2026-04-20T13:37:42Z";
        let approval_status = ReviewStatus::Approve;
        let review_is_requested = false;
        let review_is_by_code_owner = true;
        let review_counts_in_rule = true;
        let new_review_in_progress = false;
        let attachment =
            Iri::try_from("https://example.dev/alice/myrepo/pulls/1/review/1").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Approval",
  "attachment": "{attachment}",
  "attributedTo": "{attributed_to}",
  "published": "{published}",
  "approvalStatus": "{approval_status}",
  "reviewIsRequested": {review_is_requested},
  "reviewIsByCodeOwner": {review_is_by_code_owner},
  "reviewCountsInRule": {review_counts_in_rule},
  "newReviewInProgress": {new_review_in_progress}
}}"#
        );

        let context_property = context::forgefed_context();

        let approval = Approval::new()
            .with_context_property(context_property)
            .with_attributed_to(attributed_to)
            .with_published(published.parse::<DateTime>().unwrap())
            .with_approval_status(approval_status)
            .with_review_is_requested(review_is_requested)
            .with_review_is_by_code_owner(review_is_by_code_owner)
            .with_review_counts_in_rule(review_counts_in_rule)
            .with_new_review_in_progress(new_review_in_progress)
            .with_attachment(attachment);

        assert_eq!(serde_json::to_string_pretty(&approval).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Approval>(json_str.as_str()).unwrap(),
            approval
        );
    }

    #[test]
    fn test_approval_team() {
        let attributed_to = Iri::try_from("https://example.dev/alice_team").unwrap();
        let published = "2026-04-20T13:37:42Z";
        let approval_status = ReviewStatus::Approve;
        let review_is_requested = false;
        let review_is_by_code_owner = true;
        let review_counts_in_rule = true;
        let approval_id =
            Iri::try_from("https://example.dev/alice/myrepo/pulls/1/approval/1").unwrap();

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Approval",
  "attachment": [
    {{
      "type": "Approval",
      "id": "{approval_id}"
    }}
  ],
  "attributedTo": "{attributed_to}",
  "published": "{published}",
  "approvalStatus": "{approval_status}",
  "reviewIsRequested": {review_is_requested},
  "reviewIsByCodeOwner": {review_is_by_code_owner},
  "reviewCountsInRule": {review_counts_in_rule}
}}"#
        );

        let context_property = context::forgefed_context();

        let individual_approval = Approval::new_inner().with_id(approval_id);
        let attachment = [individual_approval];

        let approval = Approval::new()
            .with_context_property(context_property)
            .with_attributed_to(attributed_to)
            .with_published(published.parse::<DateTime>().unwrap())
            .with_approval_status(approval_status)
            .with_review_is_requested(review_is_requested)
            .with_review_is_by_code_owner(review_is_by_code_owner)
            .with_review_counts_in_rule(review_counts_in_rule)
            .with_attachment(attachment);

        assert_eq!(serde_json::to_string_pretty(&approval).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Approval>(json_str.as_str()).unwrap(),
            approval
        );
    }
}
