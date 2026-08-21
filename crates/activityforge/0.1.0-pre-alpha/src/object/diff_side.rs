use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};

/// Refers to either the old or new version of a file being changed.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum DiffSide {
    #[serde(rename = "diffSideNew")]
    New,
    #[serde(rename = "diffSideOld")]
    Old,
}

impl DiffSide {
    /// String representation of [New](Self::New) variant.
    pub const NEW: &str = "diffSideNew";
    /// String representation of [Old](Self::Old) variant.
    pub const OLD: &str = "diffSideOld";

    /// Creates a new [DiffSide].
    pub const fn new() -> Self {
        Self::New
    }

    /// Gets the [DiffSide] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::New => Self::NEW,
            Self::Old => Self::OLD,
        }
    }
}

impl_default!(DiffSide);
impl_display!(DiffSide, str);

impl From<DiffSide> for &'static str {
    fn from(val: DiffSide) -> Self {
        val.as_str()
    }
}

impl From<&DiffSide> for &'static str {
    fn from(val: &DiffSide) -> Self {
        val.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diff_side() {
        [
            (DiffSide::New, DiffSide::NEW),
            (DiffSide::Old, DiffSide::OLD),
        ]
        .into_iter()
        .for_each(|(diff, diff_str)| {
            let json_str = format!(r#""{diff_str}""#);

            assert_eq!(diff.to_string(), diff_str);
            assert_eq!(serde_json::to_string(&diff).unwrap(), json_str);
            assert_eq!(serde_json::from_str::<DiffSide>(&json_str).unwrap(), diff);
        });
    }
}
