use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};

/// Describes the level of approval given by a review.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ReviewVerdict {
    Approve,
    Remark,
    Revise,
}

impl ReviewVerdict {
    /// String representation for the [Approve](Self::Approve) variant.
    pub const APPROVE: &str = "approve";
    /// String representation for the [Remark](Self::Remark) variant.
    pub const REMARK: &str = "remark";
    /// String representation for the [Revise](Self::Revise) variant.
    pub const REVISE: &str = "revise";

    /// Creates a new [ReviewVerdict].
    pub const fn new() -> Self {
        Self::Approve
    }

    /// Gets the [ReviewVerdict] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Approve => Self::APPROVE,
            Self::Remark => Self::REMARK,
            Self::Revise => Self::REVISE,
        }
    }
}

impl_default!(ReviewVerdict);
impl_display!(ReviewVerdict, str);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_review_verdict() {
        [
            (ReviewVerdict::Approve, ReviewVerdict::APPROVE),
            (ReviewVerdict::Remark, ReviewVerdict::REMARK),
            (ReviewVerdict::Revise, ReviewVerdict::REVISE),
        ]
        .into_iter()
        .for_each(|(verdict, verdict_str)| {
            let json_str = format!(r#""{verdict_str}""#);

            assert_eq!(verdict.to_string(), verdict_str);
            assert_eq!(serde_json::to_string(&verdict).unwrap(), json_str);
            assert_eq!(
                serde_json::from_str::<ReviewVerdict>(&json_str).unwrap(),
                verdict
            );
        });
    }
}
