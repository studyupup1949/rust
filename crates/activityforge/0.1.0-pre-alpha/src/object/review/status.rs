use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};

/// Describes the status of the review.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReviewStatus {
    Approve,
    Remark,
    Revise,
    Awaiting,
    Dismissed,
    DismissedAwaiting,
}

impl ReviewStatus {
    /// String representation for the [Approve](Self::Approve) variant.
    pub const APPROVE: &str = "approve";
    /// String representation for the [Remark](Self::Remark) variant.
    pub const REMARK: &str = "remark";
    /// String representation for the [Revise](Self::Revise) variant.
    pub const REVISE: &str = "revise";
    /// String representation for the [Awaiting](Self::Awaiting) variant.
    pub const AWAITING: &str = "awaiting";
    /// String representation for the [Dismissed](Self::Dismissed) variant.
    pub const DISMISSED: &str = "dismissed";
    /// String representation for the [DismissedAwaiting](Self::DismissedAwaiting) variant.
    pub const DISMISSED_AWAITING: &str = "dismissedAwaiting";

    /// Creates a new [ReviewStatus].
    pub const fn new() -> Self {
        Self::Approve
    }

    /// Gets the [ReviewStatus] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Approve => Self::APPROVE,
            Self::Remark => Self::REMARK,
            Self::Revise => Self::REVISE,
            Self::Awaiting => Self::AWAITING,
            Self::Dismissed => Self::DISMISSED,
            Self::DismissedAwaiting => Self::DISMISSED_AWAITING,
        }
    }
}

impl_default!(ReviewStatus);
impl_display!(ReviewStatus, str);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_verdict() {
        [
            (ReviewStatus::Approve, ReviewStatus::APPROVE),
            (ReviewStatus::Remark, ReviewStatus::REMARK),
            (ReviewStatus::Revise, ReviewStatus::REVISE),
            (ReviewStatus::Awaiting, ReviewStatus::AWAITING),
            (ReviewStatus::Dismissed, ReviewStatus::DISMISSED),
            (
                ReviewStatus::DismissedAwaiting,
                ReviewStatus::DISMISSED_AWAITING,
            ),
        ]
        .into_iter()
        .for_each(|(verdict, verdict_str)| {
            let json_str = format!(r#""{verdict_str}""#);

            assert_eq!(verdict.to_string(), verdict_str);
            assert_eq!(serde_json::to_string(&verdict).unwrap(), json_str);
            assert_eq!(
                serde_json::from_str::<ReviewStatus>(&json_str).unwrap(),
                verdict
            );
        });
    }
}
