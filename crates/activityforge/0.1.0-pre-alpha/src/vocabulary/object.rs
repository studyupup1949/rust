use activitystreams_vocabulary::{
    impl_activity_vocabulary, impl_default, impl_display, impl_into_vocabulary,
};
use serde::{Deserialize, Serialize};

/// Represents the [ForgeFed](https://forgefed.org/ns) `Object` type variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum ObjectType {
    CapabilityUsage,
    Role,
    Branch,
    Commit,
    Patch,
    TicketDependency,
    Ticket,
    Enum,
    EnumValue,
    Field,
    FieldType,
    FieldValue,
    Milestone,
    Release,
    ReviewVerdict,
    ReviewStatus,
    ReviewThread,
    Suggestion,
    CodeQuote,
    Approval,
    DiffSide,
    Review,
    SshPublicKey,
}

impl ObjectType {
    /// String representation of the [CapabilityUsage](Self::CapabilityUsage) variant.
    pub const CAPABILITY_USAGE: &str = "CapabilityUsage";
    /// String representation of the [Role](Self::Role) variant.
    pub const ROLE: &str = "Role";
    /// String representation of the [Branch](Self::Branch) variant.
    pub const BRANCH: &str = "Branch";
    /// String representation of the [Commit](Self::Commit) variant.
    pub const COMMIT: &str = "Commit";
    /// String representation of the [Patch](Self::Patch) variant.
    pub const PATCH: &str = "Patch";
    /// String representation of the [TicketDependency](Self::TicketDependency) variant.
    pub const TICKET_DEPENDENCY: &str = "TicketDependency";
    /// String representation of the [Ticket](Self::Ticket) variant.
    pub const TICKET: &str = "Ticket";
    /// String representation of the [Enum](Self::Enum) variant.
    pub const ENUM: &str = "Enum";
    /// String representation of the [EnumValue](Self::EnumValue) variant.
    pub const ENUM_VALUE: &str = "EnumValue";
    /// String representation of the [Field](Self::Field) variant.
    pub const FIELD: &str = "Field";
    /// String representation of the [FieldType](Self::FieldType) variant.
    pub const FIELD_TYPE: &str = "FieldType";
    /// String representation of the [FieldValue](Self::FieldValue) variant.
    pub const FIELD_VALUE: &str = "FieldValue";
    /// String representation of the [Milestone](Self::Milestone) variant.
    pub const MILESTONE: &str = "Milestone";
    /// String representation of the [Release](Self::Release) variant.
    pub const RELEASE: &str = "Release";
    /// String representation of the [ReviewVerdict](Self::ReviewVerdict) variant.
    pub const REVIEW_VERDICT: &str = "ReviewVerdict";
    /// String representation of the [ReviewStatus](Self::ReviewStatus) variant.
    pub const REVIEW_STATUS: &str = "ReviewStatus";
    /// String representation of the [ReviewThread](Self::ReviewThread) variant.
    pub const REVIEW_THREAD: &str = "ReviewThread";
    /// String representation of the [Suggestion](Self::Suggestion) variant.
    pub const SUGGESTION: &str = "Suggestion";
    /// String representation of the [CodeQuote](Self::CodeQuote) variant.
    pub const CODE_QUOTE: &str = "CodeQuote";
    /// String representation of the [Approval](Self::Approval) variant.
    pub const APPROVAL: &str = "Approval";
    /// String representation of the [DiffSide](Self::DiffSide) variant.
    pub const DIFF_SIDE: &str = "DiffSide";
    /// String representation of the [Review](Self::Review) variant.
    pub const REVIEW: &str = "Review";
    /// String representation of the [SshPublicKey](Self::SshPublicKey) variant.
    pub const SSH_PUBLIC_KEY: &str = "SshPublicKey";

    /// Creates a new [ForgeAc
    pub const fn new() -> Self {
        Self::CapabilityUsage
    }

    /// Gets the [ObjectType] string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::CapabilityUsage => Self::CAPABILITY_USAGE,
            Self::Role => Self::ROLE,
            Self::Branch => Self::BRANCH,
            Self::Commit => Self::COMMIT,
            Self::Patch => Self::PATCH,
            Self::TicketDependency => Self::TICKET_DEPENDENCY,
            Self::Ticket => Self::TICKET,
            Self::Enum => Self::ENUM,
            Self::EnumValue => Self::ENUM_VALUE,
            Self::Field => Self::FIELD,
            Self::FieldType => Self::FIELD_TYPE,
            Self::FieldValue => Self::FIELD_VALUE,
            Self::Milestone => Self::MILESTONE,
            Self::Release => Self::RELEASE,
            Self::ReviewVerdict => Self::REVIEW_VERDICT,
            Self::ReviewStatus => Self::REVIEW_STATUS,
            Self::ReviewThread => Self::REVIEW_THREAD,
            Self::Suggestion => Self::SUGGESTION,
            Self::CodeQuote => Self::CODE_QUOTE,
            Self::Approval => Self::APPROVAL,
            Self::DiffSide => Self::DIFF_SIDE,
            Self::Review => Self::REVIEW,
            Self::SshPublicKey => Self::SSH_PUBLIC_KEY,
        }
    }
}

impl_default!(ObjectType);
impl_display!(ObjectType, str);
impl_activity_vocabulary!(ObjectType);
impl_into_vocabulary!(ObjectType);

#[cfg(test)]
mod tests {
    use activitystreams_vocabulary::ActivityVocabulary;

    use super::*;
    use crate::tests::TestType;

    #[test]
    fn test_object() {
        [
            (ObjectType::CapabilityUsage, ObjectType::CAPABILITY_USAGE),
            (ObjectType::Role, ObjectType::ROLE),
            (ObjectType::Branch, ObjectType::BRANCH),
            (ObjectType::Commit, ObjectType::COMMIT),
            (ObjectType::Patch, ObjectType::PATCH),
            (ObjectType::TicketDependency, ObjectType::TICKET_DEPENDENCY),
            (ObjectType::Ticket, ObjectType::TICKET),
            (ObjectType::Enum, ObjectType::ENUM),
            (ObjectType::EnumValue, ObjectType::ENUM_VALUE),
            (ObjectType::Field, ObjectType::FIELD),
            (ObjectType::FieldType, ObjectType::FIELD_TYPE),
            (ObjectType::FieldValue, ObjectType::FIELD_VALUE),
            (ObjectType::Milestone, ObjectType::MILESTONE),
            (ObjectType::Release, ObjectType::RELEASE),
            (ObjectType::ReviewVerdict, ObjectType::REVIEW_VERDICT),
            (ObjectType::ReviewStatus, ObjectType::REVIEW_STATUS),
            (ObjectType::ReviewThread, ObjectType::REVIEW_THREAD),
            (ObjectType::Suggestion, ObjectType::SUGGESTION),
            (ObjectType::CodeQuote, ObjectType::CODE_QUOTE),
            (ObjectType::Approval, ObjectType::APPROVAL),
            (ObjectType::DiffSide, ObjectType::DIFF_SIDE),
            (ObjectType::Review, ObjectType::REVIEW),
            (ObjectType::SshPublicKey, ObjectType::SSH_PUBLIC_KEY),
        ]
        .into_iter()
        .for_each(|(ty, ty_str)| {
            assert_eq!(ty.as_str(), ty_str);
            assert_eq!(ty.kind(), ty_str);
            assert_eq!(ty.as_type(), Ok(ty));

            let json_str = format!(r#""{ty_str}""#);
            assert_eq!(serde_json::to_string(&ty).unwrap(), json_str);
            assert_eq!(
                serde_json::from_str::<ObjectType>(json_str.as_str()).unwrap(),
                ty
            );

            let test_ty = serde_json::from_str::<TestType<ObjectType>>(json_str.as_str()).unwrap();
            assert_eq!(test_ty.as_type().unwrap(), ty);
        });
    }
}
