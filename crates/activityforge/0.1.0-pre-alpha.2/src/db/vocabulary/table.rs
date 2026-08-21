use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sha3::TurboShake256;
use sha3::digest::{ExtendableOutput, Update, XofReader};

use crate::db::{Iri, Name, Uuid};
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
    #[sqlx(rename = "accept_activity")]
    Accept,
    #[sqlx(rename = "create_activity")]
    Create,
    #[sqlx(rename = "follow_activity")]
    Follow,
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
    #[sqlx(rename = "role_filter")]
    RoleFilter,
    #[sqlx(rename = "application")]
    Application,
    #[sqlx(rename = "person")]
    Person,
    #[sqlx(rename = "oauth_grant")]
    OAuthGrant,
    #[sqlx(rename = "oauth_token")]
    OAuthToken,
    #[sqlx(rename = "oauth_client")]
    OAuthClient,
}

impl TableType {
    /// String representation of the [Inbox](Self::Inbox) variant.
    pub const INBOX: &str = "inbox";
    /// Path string of the [Inbox](Self::Inbox) variant.
    pub const INBOX_PATH: &str = "/api/v1/inboxes";
    /// String representation of the [Outbox](Self::Outbox) variant.
    pub const OUTBOX: &str = "outbox";
    /// Path string of the [Outbox](Self::Outbox) variant.
    pub const OUTBOX_PATH: &str = "/api/v1/outboxes";
    /// String representation of the [Collaborator](Self::Collaborator) variant.
    pub const COLLABORATOR: &str = "collaborator";
    /// Path string of the [Collaborator](Self::Collaborator) variant.
    pub const COLLABORATOR_PATH: &str = "/api/v1/collaborators";
    /// String representation of the [Follow](Self::Follow) variant.
    pub const FOLLOW: &str = "follow_activity";
    /// Path string of the [Follow](Self::Follow) variant.
    pub const FOLLOW_PATH: &str = "/api/v1/follow";
    /// String representation of the [Follower](Self::Follower) variant.
    pub const FOLLOWER: &str = "follower";
    /// Path string of the [Follower](Self::Follower) variant.
    pub const FOLLOWER_PATH: &str = "/api/v1/followers";
    /// String representation of the [Key](Self::Key) variant.
    pub const KEY: &str = "key";
    /// Path string of the [Key](Self::Key) variant.
    pub const KEY_PATH: &str = "/api/v1/keys";
    /// String representation of the [Accept](Self::Accept) variant.
    pub const ACCEPT: &str = "accept_activity";
    /// Path string of the [Accept](Self::Accept) variant.
    pub const ACCEPT_PATH: &str = "/api/v1/accepts";
    /// String representation of the [Create](Self::Create) variant.
    pub const CREATE: &str = "create_activity";
    /// Path string of the [Create](Self::Create) variant.
    pub const CREATE_PATH: &str = "/api/v1/creates";
    /// String representation of the [Grant](Self::Grant) variant.
    pub const GRANT: &str = "role_grant";
    /// Path string of the [Grant](Self::Grant) variant.
    pub const GRANT_PATH: &str = "/api/v1/grants";
    /// String representation of the [Like](Self::Like) variant.
    pub const LIKE: &str = "like_activity";
    /// Path string of the [Like](Self::Like) variant.
    pub const LIKE_PATH: &str = "/api/v1/likes";
    /// String representation of the [Team](Self::Team) variant.
    pub const TEAM: &str = "team";
    /// Path string of the [Team](Self::Team) variant.
    pub const TEAM_PATH: &str = "/api/v1/teams";
    /// String representation of the [Factory](Self::Factory) variant.
    pub const FACTORY: &str = "factory";
    /// Path string of the [Factory](Self::Factory) variant.
    pub const FACTORY_PATH: &str = "/api/v1/factories";
    /// String representation of the [PatchTracker](Self::PatchTracker) variant.
    pub const PATCH_TRACKER: &str = "patch_tracker";
    /// Path string of the [PatchTracker](Self::PatchTracker) variant.
    pub const PATCH_TRACKER_PATH: &str = "/api/v1/patch_trackers";
    /// String representation of the [TicketTracker](Self::TicketTracker) variant.
    pub const TICKET_TRACKER: &str = "ticket_tracker";
    /// Path string of the [TicketTracker](Self::TicketTracker) variant.
    pub const TICKET_TRACKER_PATH: &str = "/api/v1/ticket_trackers";
    /// String representation of the [Activity](Self::Activity) variant.
    pub const ACTIVITY: &str = "activity";
    /// Path string of the [Activity](Self::Activity) variant.
    pub const ACTIVITY_PATH: &str = "/api/v1/activities";
    /// String representation of the [Object](Self::Object) variant.
    pub const OBJECT: &str = "object";
    /// Path string of the [Object](Self::Object) variant.
    pub const OBJECT_PATH: &str = "/api/v1/objects";
    /// String representation of the [Repository](Self::Repository) variant.
    pub const REPOSITORY: &str = "repository";
    /// Path string of the [Repository](Self::Repository) variant.
    pub const REPOSITORY_PATH: &str = "/api/v1/repositories";
    /// String representation of the [RoleFilter](Self::RoleFilter) variant.
    pub const ROLE_FILTER: &str = "role_filter";
    /// Path string of the [RoleFilter](Self::RoleFilter) variant.
    pub const ROLE_FILTER_PATH: &str = "/api/v1/role_filters";
    /// String representation of the [Person](Self::Person) variant.
    pub const PERSON: &str = "person";
    /// Path string of the [Person](Self::Person) variant.
    pub const PERSON_PATH: &str = "/api/v1/persons";
    /// String representation of the [Application](Self::Application) variant.
    pub const APPLICATION: &str = "application";
    /// Path string of the [Application](Self::Application) variant.
    pub const APPLICATION_PATH: &str = "/api/v1/applications";
    /// String representation of the [OAuthGrant](Self::OAuthGrant) variant.
    pub const OAUTH_GRANT: &str = "oauth_grant";
    /// Path string of the [OAuthGrant](Self::OAuthGrant) variant.
    pub const OAUTH_GRANT_PATH: &str = "/api/v1/oauth_grants";
    /// String representation of the [OAuthToken](Self::OAuthToken) variant.
    pub const OAUTH_TOKEN: &str = "oauth_token";
    /// Path string of the [OAuthToken](Self::OAuthToken) variant.
    pub const OAUTH_TOKEN_PATH: &str = "/api/v1/oauth_tokens";
    /// String representation of the [OAuthClient](Self::OAuthClient) variant.
    pub const OAUTH_CLIENT: &str = "oauth_client";
    /// Path string of the [OAuthClient](Self::OAuthClient) variant.
    pub const OAUTH_CLIENT_PATH: &str = "/api/v1/oauth_clients";
    pub const HASH_CONTEXT: u8 = 6;

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
            Self::Follow => Self::FOLLOW,
            Self::Follower => Self::FOLLOWER,
            Self::Key => Self::KEY,
            Self::Accept => Self::ACCEPT,
            Self::Create => Self::CREATE,
            Self::Like => Self::LIKE,
            Self::Grant => Self::GRANT,
            Self::Team => Self::TEAM,
            Self::Factory => Self::FACTORY,
            Self::PatchTracker => Self::PATCH_TRACKER,
            Self::TicketTracker => Self::TICKET_TRACKER,
            Self::Activity => Self::ACTIVITY,
            Self::Object => Self::OBJECT,
            Self::Repository => Self::REPOSITORY,
            Self::RoleFilter => Self::ROLE_FILTER,
            Self::Person => Self::PERSON,
            Self::Application => Self::APPLICATION,
            Self::OAuthGrant => Self::OAUTH_GRANT,
            Self::OAuthToken => Self::OAUTH_TOKEN,
            Self::OAuthClient => Self::OAUTH_CLIENT,
        }
    }

    /// Gets the [TableType] path string.
    pub const fn path(&self) -> &'static str {
        match self {
            Self::Inbox => Self::INBOX_PATH,
            Self::Outbox => Self::OUTBOX_PATH,
            Self::Collaborator => Self::COLLABORATOR_PATH,
            Self::Follow => Self::FOLLOW_PATH,
            Self::Follower => Self::FOLLOWER_PATH,
            Self::Key => Self::KEY_PATH,
            Self::Accept => Self::ACCEPT_PATH,
            Self::Create => Self::CREATE_PATH,
            Self::Like => Self::LIKE_PATH,
            Self::Grant => Self::GRANT_PATH,
            Self::Team => Self::TEAM_PATH,
            Self::Factory => Self::FACTORY_PATH,
            Self::PatchTracker => Self::PATCH_TRACKER_PATH,
            Self::TicketTracker => Self::TICKET_TRACKER_PATH,
            Self::Activity => Self::ACTIVITY_PATH,
            Self::Object => Self::OBJECT_PATH,
            Self::Repository => Self::REPOSITORY_PATH,
            Self::RoleFilter => Self::ROLE_FILTER_PATH,
            Self::Person => Self::PERSON_PATH,
            Self::Application => Self::APPLICATION_PATH,
            Self::OAuthGrant => Self::OAUTH_GRANT_PATH,
            Self::OAuthToken => Self::OAUTH_TOKEN_PATH,
            Self::OAuthClient => Self::OAUTH_CLIENT_PATH,
        }
    }

    /// Attempts to construct a new ID from an existing [Iri] and [Uuid].
    ///
    /// Convenience function to consistently construct new IDs.
    pub fn id_from_uuid(&self, id: &Iri, uuid: Uuid) -> Result<Iri> {
        let base_iri = id.base_iri()?;
        let path = self.path();

        Iri::try_from(format!("{base_iri}{path}/{uuid}"))
    }

    /// Attempts to construct a new ID from an existing [Iri] and [Name].
    ///
    /// Returns the constructed [Uuid] and [Iri].
    ///
    /// Convenience function to consistently construct new IDs.
    ///
    /// ## NOTES
    ///
    /// - should only be used for local records
    /// - `TableType`-`iri`-`name` tuple **MUST** be unique
    pub fn id_from_name(&self, iri: &Iri, name: &Name) -> Result<(Uuid, Iri)> {
        let base_iri = iri.base_iri()?;
        let path = self.path();

        let mut hasher = TurboShake256::from_core(sha3::TurboShake256Core::new(Self::HASH_CONTEXT));

        hasher.update(base_iri.as_str().as_bytes());
        hasher.update(path.as_bytes());
        hasher.update(name.as_str().as_bytes());

        let mut uuid_bytes = [0u8; 16];
        let mut reader = hasher.finalize_xof();
        reader.read(uuid_bytes.as_mut());

        let uuid = Uuid::from_bytes(uuid_bytes);
        Iri::try_from(format!("{base_iri}{path}/{uuid}")).map(|id| (uuid, id))
    }

    /// Creates a UUID based on unqiue identifiers of the record referenced by `id`.
    pub fn uuid_from_id<I: Into<Iri>>(&self, id: I) -> Uuid {
        let mut hasher = TurboShake256::from_core(sha3::TurboShake256Core::new(Self::HASH_CONTEXT));

        hasher.update(self.as_str().as_bytes());
        hasher.update(id.into().as_str().as_bytes());

        let mut uuid_bytes = [0u8; 16];
        let mut reader = hasher.finalize_xof();
        reader.read(uuid_bytes.as_mut());

        Uuid::from_bytes(uuid_bytes)
    }

    /// Creates a UUID based on unqiue identifiers.
    ///
    /// **NOTE**: `ids` should be a unique list of IDs for the [TableType].
    pub fn uuid_from_ids<'a, I>(&self, ids: I) -> Uuid
    where
        I: IntoIterator<Item = &'a Iri>,
    {
        let mut hasher = TurboShake256::from_core(sha3::TurboShake256Core::new(Self::HASH_CONTEXT));

        hasher.update(self.as_str().as_bytes());
        ids.into_iter()
            .for_each(|id| hasher.update(id.as_str().as_bytes()));

        let mut uuid_bytes = [0u8; 16];
        let mut reader = hasher.finalize_xof();
        reader.read(uuid_bytes.as_mut());

        Uuid::from_bytes(uuid_bytes)
    }

    /// Gets whether the table is for `Object` records.
    pub const fn is_object(&self) -> bool {
        matches!(
            self,
            Self::Inbox
                | Self::Outbox
                | Self::Collaborator
                | Self::Key
                | Self::Object
                | Self::RoleFilter
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
        matches!(
            self,
            Self::Accept | Self::Create | Self::Follow | Self::Grant | Self::Like | Self::Activity
        )
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
            Self::Application
                | Self::Factory
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
