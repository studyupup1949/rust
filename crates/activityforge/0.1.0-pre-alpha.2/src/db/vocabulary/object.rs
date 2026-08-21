use serde::{Deserialize, Serialize};

/// Represents the `Object` type variants.
///
/// - [ForgeFed](crate::ObjectType)
/// - [ActivityStreams](activitystreams_vocabulary::ObjectType)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, sqlx::Type)]
#[sqlx(type_name = "object_type")]
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
    Article,
    Audio,
    Document,
    Event,
    Image,
    Note,
    Page,
    Place,
    Profile,
    Relationship,
    Tombstone,
    Video,
}

