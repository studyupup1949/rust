use activitystreams_vocabulary::{
    impl_activity_vocabulary, impl_default, impl_display, impl_into_vocabulary,
};
use serde::{Deserialize, Serialize};

/// Represents the [ForgeFed](https://forgefed.org/ns) `Activity` type variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum ActivityType {
    Edit,
    Grant,
    RoleFilter,
    Revoke,
    Push,
    Assign,
    Resolve,
    Apply,
}

impl ActivityType {
    /// String representation of the [Edit](Self::Edit) variant.
    pub const EDIT: &str = "Edit";
    /// String representation of the [Grant](Self::Grant) variant.
    pub const GRANT: &str = "Grant";
    /// String representation of the [RoleFilter](Self::RoleFilter) variant.
    pub const ROLE_FILTER: &str = "RoleFilter";
    /// String representation of the [Revoke](Self::Revoke) variant.
    pub const REVOKE: &str = "Revoke";
    /// String representation of the [Push](Self::Push) variant.
    pub const PUSH: &str = "Push";
    /// String representation of the [Assign](Self::Assign) variant.
    pub const ASSIGN: &str = "Assign";
    /// String representation of the [Resolve](Self::Resolve) variant.
    pub const RESOLVE: &str = "Resolve";
    /// String representation of the [Apply](Self::Apply) variant.
    pub const APPLY: &str = "Apply";

    /// Creates a new [ForgeAc
    pub const fn new() -> Self {
        Self::Edit
    }

    /// Gets the [ActivityType] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Edit => Self::EDIT,
            Self::Grant => Self::GRANT,
            Self::RoleFilter => Self::ROLE_FILTER,
            Self::Revoke => Self::REVOKE,
            Self::Push => Self::PUSH,
            Self::Assign => Self::ASSIGN,
            Self::Resolve => Self::RESOLVE,
            Self::Apply => Self::APPLY,
        }
    }
}

impl_default!(ActivityType);
impl_display!(ActivityType, str);
impl_activity_vocabulary!(ActivityType);
impl_into_vocabulary!(ActivityType);

#[cfg(test)]
mod tests {
    use activitystreams_vocabulary::ActivityVocabulary;

    use super::*;
    use crate::tests::TestType;

    #[test]
    fn test_activity() {
        [
            (ActivityType::Edit, ActivityType::EDIT),
            (ActivityType::Grant, ActivityType::GRANT),
            (ActivityType::RoleFilter, ActivityType::ROLE_FILTER),
            (ActivityType::Revoke, ActivityType::REVOKE),
            (ActivityType::Push, ActivityType::PUSH),
            (ActivityType::Assign, ActivityType::ASSIGN),
            (ActivityType::Resolve, ActivityType::RESOLVE),
            (ActivityType::Apply, ActivityType::APPLY),
        ]
        .into_iter()
        .for_each(|(ty, ty_str)| {
            assert_eq!(ty.as_str(), ty_str);
            assert_eq!(ty.kind(), ty_str);
            assert_eq!(ty.as_type(), Ok(ty));

            let json_str = format!(r#""{ty_str}""#);
            assert_eq!(serde_json::to_string(&ty).unwrap(), json_str);
            assert_eq!(
                serde_json::from_str::<ActivityType>(json_str.as_str()).unwrap(),
                ty
            );

            let test_ty =
                serde_json::from_str::<TestType<ActivityType>>(json_str.as_str()).unwrap();
            assert_eq!(test_ty.as_type().unwrap(), ty);
        });
    }
}
