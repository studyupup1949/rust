use serde::{Deserialize, Serialize};

use crate::{
    ActivityVocabulary, Error, Result, VocabularyType, VocabularyTypes, impl_default, impl_display,
};

/// Represents the ActivityStream vocabulary type variants for "activities".
///
/// All Activity Types inherit the properties of the base Activity type.
///
/// Some specific Activity Types are subtypes or specializations of more generalized Activity Types
/// (for instance, the [Invite](Self::Invite) Activity Type is a more specific form of the [Offer](Self::Offer) Activity Type).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize)]
pub enum ActivityType {
    Accept,
    Add,
    Announce,
    Arrive,
    Block,
    Create,
    Delete,
    Dislike,
    Flag,
    Follow,
    Ignore,
    Invite,
    Join,
    Leave,
    Like,
    Listen,
    Move,
    Offer,
    Question,
    Reject,
    Read,
    Remove,
    TentativeReject,
    TentativeAccept,
    Travel,
    Undo,
    Update,
    View,
}

impl ActivityType {
    /// Represents the string for the [Accept](Self::Accept) type.
    pub const ACCEPT: &str = "Accept";
    /// Represents the string for the [Add](Self::Add) type.
    pub const ADD: &str = "Add";
    /// Represents the string for the [Announce](Self::Announce) type.
    pub const ANNOUNCE: &str = "Announce";
    /// Represents the string for the [Arrive](Self::Arrive) type.
    pub const ARRIVE: &str = "Arrive";
    /// Represents the string for the [Block](Self::Block) type.
    pub const BLOCK: &str = "Block";
    /// Represents the string for the [Create](Self::Create) type.
    pub const CREATE: &str = "Create";
    /// Represents the string for the [Delete](Self::Delete) type.
    pub const DELETE: &str = "Delete";
    /// Represents the string for the [Dislike](Self::Dislike) type.
    pub const DISLIKE: &str = "Dislike";
    /// Represents the string for the [Flag](Self::Flag) type.
    pub const FLAG: &str = "Flag";
    /// Represents the string for the [Follow](Self::Follow) type.
    pub const FOLLOW: &str = "Follow";
    /// Represents the string for the [Ignore](Self::Ignore) type.
    pub const IGNORE: &str = "Ignore";
    /// Represents the string for the [Invite](Self::Invite) type.
    pub const INVITE: &str = "Invite";
    /// Represents the string for the [Join](Self::Join) type.
    pub const JOIN: &str = "Join";
    /// Represents the string for the [Leave](Self::Leave) type.
    pub const LEAVE: &str = "Leave";
    /// Represents the string for the [Like](Self::Like) type.
    pub const LIKE: &str = "Like";
    /// Represents the string for the [Listen](Self::Listen) type.
    pub const LISTEN: &str = "Listen";
    /// Represents the string for the [Move](Self::Move) type.
    pub const MOVE: &str = "Move";
    /// Represents the string for the [Offer](Self::Offer) type.
    pub const OFFER: &str = "Offer";
    /// Represents the string for the [Question](Self::Question) type.
    pub const QUESTION: &str = "Question";
    /// Represents the string for the [Reject](Self::Reject) type.
    pub const REJECT: &str = "Reject";
    /// Represents the string for the [Read](Self::Read) type.
    pub const READ: &str = "Read";
    /// Represents the string for the [Remove](Self::Remove) type.
    pub const REMOVE: &str = "Remove";
    /// Represents the string for the [TentativeReject](Self::TentativeReject) type.
    pub const TENTATIVE_REJECT: &str = "TentativeReject";
    /// Represents the string for the [TentativeAccept](Self::TentativeAccept) type.
    pub const TENTATIVE_ACCEPT: &str = "TentativeAccept";
    /// Represents the string for the [Travel](Self::Travel) type.
    pub const TRAVEL: &str = "Travel";
    /// Represents the string for the [Undo](Self::Undo) type.
    pub const UNDO: &str = "Undo";
    /// Represents the string for the [Update](Self::Update) type.
    pub const UPDATE: &str = "Update";
    /// Represents the string for the [View](Self::View) type.
    pub const VIEW: &str = "View";

    /// Creates a new [ActivityType].
    pub const fn new() -> Self {
        Self::Accept
    }

    /// Gets the string representation of the [ActivityType].
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Accept => Self::ACCEPT,
            Self::Add => Self::ADD,
            Self::Announce => Self::ANNOUNCE,
            Self::Arrive => Self::ARRIVE,
            Self::Block => Self::BLOCK,
            Self::Create => Self::CREATE,
            Self::Delete => Self::DELETE,
            Self::Dislike => Self::DISLIKE,
            Self::Flag => Self::FLAG,
            Self::Follow => Self::FOLLOW,
            Self::Ignore => Self::IGNORE,
            Self::Invite => Self::INVITE,
            Self::Join => Self::JOIN,
            Self::Leave => Self::LEAVE,
            Self::Like => Self::LIKE,
            Self::Listen => Self::LISTEN,
            Self::Move => Self::MOVE,
            Self::Offer => Self::OFFER,
            Self::Question => Self::QUESTION,
            Self::Reject => Self::REJECT,
            Self::Read => Self::READ,
            Self::Remove => Self::REMOVE,
            Self::TentativeReject => Self::TENTATIVE_REJECT,
            Self::TentativeAccept => Self::TENTATIVE_ACCEPT,
            Self::Travel => Self::TRAVEL,
            Self::Undo => Self::UNDO,
            Self::Update => Self::UPDATE,
            Self::View => Self::VIEW,
        }
    }

    /// Converts the [ActivityType] to a [VocabularyType].
    #[inline]
    pub const fn to_vocabulary(self) -> VocabularyType {
        VocabularyType::Activity(self)
    }

    /// Converts the [ActivityType] to a [VocabularyTypes].
    #[inline]
    pub const fn to_vocabulary_types(self) -> VocabularyTypes {
        VocabularyTypes::Single(self.to_vocabulary())
    }
}

impl_default!(ActivityType);
impl_display!(ActivityType, str);

impl ActivityVocabulary for ActivityType {
    type Type = ActivityType;

    fn kind(&self) -> String {
        self.to_string()
    }

    fn contains(&self, kind: &str) -> bool {
        self.as_str() == kind
    }
}

impl From<ActivityType> for &'static str {
    fn from(val: ActivityType) -> Self {
        val.as_str()
    }
}

impl From<ActivityType> for VocabularyType {
    fn from(val: ActivityType) -> Self {
        val.to_vocabulary()
    }
}

impl TryFrom<VocabularyType> for ActivityType {
    type Error = Error;

    fn try_from(val: VocabularyType) -> Result<Self> {
        val.to_activity()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::TestType;

    #[test]
    fn test_activity() {
        [
            (ActivityType::Accept, ActivityType::ACCEPT),
            (ActivityType::Add, ActivityType::ADD),
            (ActivityType::Announce, ActivityType::ANNOUNCE),
            (ActivityType::Arrive, ActivityType::ARRIVE),
            (ActivityType::Block, ActivityType::BLOCK),
            (ActivityType::Create, ActivityType::CREATE),
            (ActivityType::Delete, ActivityType::DELETE),
            (ActivityType::Dislike, ActivityType::DISLIKE),
            (ActivityType::Flag, ActivityType::FLAG),
            (ActivityType::Follow, ActivityType::FOLLOW),
            (ActivityType::Ignore, ActivityType::IGNORE),
            (ActivityType::Invite, ActivityType::INVITE),
            (ActivityType::Join, ActivityType::JOIN),
            (ActivityType::Leave, ActivityType::LEAVE),
            (ActivityType::Like, ActivityType::LIKE),
            (ActivityType::Listen, ActivityType::LISTEN),
            (ActivityType::Move, ActivityType::MOVE),
            (ActivityType::Offer, ActivityType::OFFER),
            (ActivityType::Question, ActivityType::QUESTION),
            (ActivityType::Reject, ActivityType::REJECT),
            (ActivityType::Read, ActivityType::READ),
            (ActivityType::Remove, ActivityType::REMOVE),
            (
                ActivityType::TentativeReject,
                ActivityType::TENTATIVE_REJECT,
            ),
            (
                ActivityType::TentativeAccept,
                ActivityType::TENTATIVE_ACCEPT,
            ),
            (ActivityType::Travel, ActivityType::TRAVEL),
            (ActivityType::Undo, ActivityType::UNDO),
            (ActivityType::Update, ActivityType::UPDATE),
            (ActivityType::View, ActivityType::VIEW),
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
