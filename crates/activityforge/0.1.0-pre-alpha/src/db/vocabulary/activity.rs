use serde::{Deserialize, Serialize};

/// Represents the `Activity` type variants.
///
/// - [ForgeFed](crate::ActivityType)
/// - [ActivityStreams](activitystreams_vocabulary::ActivityType)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "activity_type")]
pub enum ActivityType {
    Edit,
    Grant,
    RoleFilter,
    Revoke,
    Push,
    Assign,
    Resolve,
    Apply,
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
