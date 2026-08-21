use activitystreams_vocabulary::{Item, create_object, field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};

/// Represents a relationship for a member or collaborator.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, sqlx::Type,
)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "collab_relationship")]
pub enum CollabRelationship {
    #[sqlx(rename = "hasCollaborator")]
    HasCollaborator,
    #[sqlx(rename = "hasMember")]
    HasMember,
}

impl CollabRelationship {
    /// Represents the string for the [HasCollaborator](Self::HasCollaborator) variant.
    pub const HAS_COLLABORATOR: &str = "hasCollaborator";
    /// Represents the string for the [HasMember](Self::HasMember) variant.
    pub const HAS_MEMBER: &str = "hasMember";

    /// Creates a new [CollabRelationship].
    pub const fn new() -> Self {
        Self::HasCollaborator
    }

    /// Gets the string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::HasCollaborator => Self::HAS_COLLABORATOR,
            Self::HasMember => Self::HAS_MEMBER,
        }
    }
}

impl_default!(CollabRelationship);
impl_display!(CollabRelationship, str);

create_object! {
    /// Represents a collaborator actor that has direct-grant access to a resource,
    /// e.g. a `Project`, `Team` or `Repository`.
    Collaborator: activitystreams_vocabulary::ObjectType::Relationship {
        #[serde(skip_serializing_if = "Option::is_none")]
        subject: Option<Item>,
        #[serde(skip_serializing_if = "Option::is_none")]
        relationship: Option<CollabRelationship>,
        #[serde(skip_serializing_if = "Option::is_none")]
        object: Option<Item>,
    }
}

field_access! {
    Collaborator {
        /// Represents the `subject` on which the `object` is collaborating.
        subject: option_ref { Item },
        /// Represents the `object` collaborating on the `subject`.
        object: option_ref { Item },
    }
}

field_access! {
    Collaborator {
        /// Represents the collaboration relationship.
        relationship: option { CollabRelationship },
    }
}
