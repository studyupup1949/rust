use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};

use crate::{ActorType as VocabActorType, Error, Result};

/// Represents the `Actor` type variants.
///
/// - [ForgeFed](crate::ActorType)
/// - [ActivityStreams](activitystreams_vocabulary::ActorType)
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, sqlx::Type,
)]
#[sqlx(type_name = "actor_type")]
pub enum ActorType {
    Factory,
    Repository,
    PatchTracker,
    ReleaseTracker,
    Roadmap,
    TicketTracker,
    Project,
    Team,
    Workflow,
    Application,
    Group,
    Organization,
    Person,
    Service,
}

impl ActorType {
    pub const FACTORY: &str = "factory";
    pub const REPOSITORY: &str = "repository";
    pub const PATCH_TRACKER: &str = "patch_tracker";
    pub const RELEASE_TRACKER: &str = "release_tracker";
    pub const ROADMAP: &str = "roadmap";
    pub const TICKET_TRACKER: &str = "ticket_tracker";
    pub const PROJECT: &str = "project";
    pub const TEAM: &str = "team";
    pub const WORKFLOW: &str = "workflow";
    pub const APPLICATION: &str = "application";
    pub const GROUP: &str = "group";
    pub const ORGANIZATION: &str = "organization";
    pub const PERSON: &str = "person";
    pub const SERVICE: &str = "service";

    /// Creates a new [ActorType].
    pub const fn new() -> Self {
        ActorType::Factory
    }

    /// Gets the string representation of the [ActorType].
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Factory => Self::FACTORY,
            Self::Repository => Self::REPOSITORY,
            Self::PatchTracker => Self::PATCH_TRACKER,
            Self::ReleaseTracker => Self::RELEASE_TRACKER,
            Self::Roadmap => Self::ROADMAP,
            Self::TicketTracker => Self::TICKET_TRACKER,
            Self::Project => Self::PROJECT,
            Self::Team => Self::TEAM,
            Self::Workflow => Self::WORKFLOW,
            Self::Application => Self::APPLICATION,
            Self::Group => Self::GROUP,
            Self::Organization => Self::ORGANIZATION,
            Self::Person => Self::PERSON,
            Self::Service => Self::SERVICE,
        }
    }
}

impl_default!(ActorType);
impl_display!(ActorType, str);

impl From<VocabActorType> for ActorType {
    fn from(val: VocabActorType) -> Self {
        (&val).into()
    }
}

impl From<&VocabActorType> for ActorType {
    fn from(val: &VocabActorType) -> Self {
        match val {
            VocabActorType::Factory => Self::Factory,
            VocabActorType::Repository => Self::Repository,
            VocabActorType::PatchTracker => Self::PatchTracker,
            VocabActorType::ReleaseTracker => Self::ReleaseTracker,
            VocabActorType::Roadmap => Self::Roadmap,
            VocabActorType::TicketTracker => Self::TicketTracker,
            VocabActorType::Project => Self::Project,
            VocabActorType::Team => Self::Team,
            VocabActorType::Workflow => Self::Workflow,
            VocabActorType::Application => Self::Application,
            VocabActorType::Person => Self::Person,
        }
    }
}

impl TryFrom<ActorType> for VocabActorType {
    type Error = Error;

    fn try_from(val: ActorType) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&ActorType> for VocabActorType {
    type Error = Error;

    fn try_from(val: &ActorType) -> Result<Self> {
        match val {
            ActorType::Factory => Ok(Self::Factory),
            ActorType::Repository => Ok(Self::Repository),
            ActorType::PatchTracker => Ok(Self::PatchTracker),
            ActorType::ReleaseTracker => Ok(Self::ReleaseTracker),
            ActorType::Roadmap => Ok(Self::Roadmap),
            ActorType::TicketTracker => Ok(Self::TicketTracker),
            ActorType::Project => Ok(Self::Project),
            ActorType::Team => Ok(Self::Team),
            ActorType::Workflow => Ok(Self::Workflow),
            _ => Err(Error::db("actor_type: unsupported actor type: {val}")),
        }
    }
}
