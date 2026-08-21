use activitystreams_vocabulary::impl_default;
use serde::{Deserialize, Serialize};

/// Represents the `Actor` type variants.
///
/// - [ForgeFed](crate::ActorType)
/// - [ActivityStreams](activitystreams_vocabulary::ActorType)
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize, sqlx::Type)]
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
    /// Creates a new [ActorType].
    pub const fn new() -> Self {
        ActorType::Factory
    }
}

impl_default!(ActorType);
