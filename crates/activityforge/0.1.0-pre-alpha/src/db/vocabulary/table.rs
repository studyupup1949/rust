use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

/// Represents the table type for a [TableEntry](crate::db::TableEntry).
///
/// Used to select the table in the database.
#[derive(
    Clone, Copy, Debug, Eq, Ord, PartialOrd, PartialEq, Deserialize, Serialize, sqlx::Type,
)]
#[serde(rename_all = "snake_case")]
#[sqlx(type_name = "table_type")]
pub enum TableType {
    #[sqlx(rename = "inbox")]
    Inbox,
    #[sqlx(rename = "outbox")]
    Outbox,
    #[sqlx(rename = "collaborator")]
    Collaborator,
    #[sqlx(rename = "follower")]
    Follower,
    #[sqlx(rename = "key")]
    Key,
    #[sqlx(rename = "role_grant")]
    Grant,
    #[sqlx(rename = "like_activity")]
    Like,
    #[sqlx(rename = "team")]
    Team,
    #[sqlx(rename = "factory")]
    Factory,
    #[sqlx(rename = "patch_tracker")]
    PatchTracker,
    #[sqlx(rename = "ticket_tracker")]
    TicketTracker,
    #[sqlx(rename = "activity")]
    Activity,
    #[sqlx(rename = "object")]
    Object,
    #[sqlx(rename = "repository")]
    Repository,
    #[sqlx(rename = "person")]
    Person,
}

impl TableType {
    /// String representation of the [Inbox](Self::Inbox) variant.
    pub const INBOX: &str = "inbox";
    /// String representation of the [Outbox](Self::Outbox) variant.
    pub const OUTBOX: &str = "outbox";
    /// String representation of the [Collaborator](Self::Collaborator) variant.
    pub const COLLABORATOR: &str = "collaborator";
    /// String representation of the [Follower](Self::Follower) variant.
    pub const FOLLOWER: &str = "follower";
    /// String representation of the [Key](Self::Key) variant.
    pub const KEY: &str = "key";
    /// String representation of the [Grant](Self::Grant) variant.
    pub const GRANT: &str = "role_grant";
    /// String representation of the [Like](Self::Like) variant.
    pub const LIKE: &str = "like_activity";
    /// String representation of the [Team](Self::Team) variant.
    pub const TEAM: &str = "team";
    /// String representation of the [Factory](Self::Factory) variant.
    pub const FACTORY: &str = "factory";
    /// String representation of the [PatchTracker](Self::PatchTracker) variant.
    pub const PATCH_TRACKER: &str = "patch_tracker";
    /// String representation of the [TicketTracker](Self::TicketTracker) variant.
    pub const TICKET_TRACKER: &str = "ticket_tracker";
    /// String representation of the [Activity](Self::Activity) variant.
    pub const ACTIVITY: &str = "activity";
    /// String representation of the [Object](Self::Object) variant.
    pub const OBJECT: &str = "object";
    /// String representation of the [Repository](Self::Repository) variant.
    pub const REPOSITORY: &str = "repository";
    /// String representation of the [Person](Self::Person) variant.
    pub const PERSON: &str = "person";

    /// Creates a new [TableType].
    pub const fn new() -> Self {
        Self::Inbox
    }

    /// Gets [TableType] the string representation.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Inbox => Self::INBOX,
            Self::Outbox => Self::OUTBOX,
            Self::Collaborator => Self::COLLABORATOR,
            Self::Follower => Self::FOLLOWER,
            Self::Key => Self::KEY,
            Self::Like => Self::LIKE,
            Self::Grant => Self::GRANT,
            Self::Team => Self::TEAM,
            Self::Factory => Self::FACTORY,
            Self::PatchTracker => Self::PATCH_TRACKER,
            Self::TicketTracker => Self::TICKET_TRACKER,
            Self::Activity => Self::ACTIVITY,
            Self::Object => Self::OBJECT,
            Self::Repository => Self::REPOSITORY,
            Self::Person => Self::PERSON,
        }
    }

    /// Gets whether the table is for `Object` records.
    pub const fn is_object(&self) -> bool {
        matches!(
            self,
            Self::Inbox | Self::Outbox | Self::Collaborator | Self::Key | Self::Object
        )
    }

    /// Checks if the table is for `Object` records, returning [Error] if not.
    pub fn check_object(&self) -> Result<()> {
        if self.is_object() {
            Ok(())
        } else {
            Err(Error::sql("table: is not for an `Object`"))
        }
    }

    /// Gets whether the table is for `Activity` records.
    pub const fn is_activity(&self) -> bool {
        matches!(self, Self::Grant | Self::Like | Self::Activity)
    }

    /// Checks if the table is for `Activity` records, returning [Error] if not.
    pub fn check_activity(&self) -> Result<()> {
        if self.is_activity() {
            Ok(())
        } else {
            Err(Error::sql("table: is not for an `Activity`"))
        }
    }

    /// Gets whether the table is for `Actor` records.
    pub const fn is_actor(&self) -> bool {
        matches!(
            self,
            Self::Factory
                | Self::Follower
                | Self::Person
                | Self::Repository
                | Self::PatchTracker
                | Self::TicketTracker
                | Self::Team
        )
    }

    /// Checks if the table is for `Actor` records, returning [Error] if not.
    pub fn check_actor(&self) -> Result<()> {
        if self.is_actor() {
            Ok(())
        } else {
            Err(Error::sql("table: is not for an `Actor`"))
        }
    }
}

impl_default!(TableType);
impl_display!(TableType, str);
