use activitystreams_vocabulary::{
    ActorType as VocabActorType, Application, Iri, Item, Items, Person, create_item,
};

use crate::db::{TableType, Uuid};
use crate::{ActorType, Error, Result};

mod factory;
mod patch_tracker;
mod project;
mod release_tracker;
mod repository;
mod roadmap;
mod team;
mod ticket_tracker;
mod workflow;

pub use factory::Factory;
pub use patch_tracker::{PatchTracker, PatchTrackerItem};
pub use project::{Project, ProjectItem};
pub use release_tracker::ReleaseTracker;
pub use repository::{Repository, RepositoryItem};
pub use roadmap::Roadmap;
pub use team::{FilterKey, RoleFilter, Team, TeamItem};
pub use ticket_tracker::{TicketTracker, TicketTrackerItem};
pub use workflow::Workflow;

create_item! {
    /// Represents the `Actor` record variants stored in the database.
    Actor
        boxed
        default: Self::Application(Box::new(Application::new())),
    {
        Application(Application),
        Factory(Factory),
        Person(Person),
        Repository(Repository),
        Team(Team),
        PatchTracker(PatchTracker),
        TicketTracker(TicketTracker),
    }
}

impl Actor {
    /// Gets the table associated with the [Actor].
    #[inline]
    pub const fn table(&self) -> TableType {
        match self {
            Self::Application(_) => TableType::Application,
            Self::Factory(_) => TableType::Factory,
            Self::Person(_) => TableType::Person,
            Self::Repository(_) => TableType::Repository,
            Self::Team(_) => TableType::Team,
            Self::PatchTracker(_) => TableType::PatchTracker,
            Self::TicketTracker(_) => TableType::TicketTracker,
        }
    }

    /// Creates a UUID based on unqiue identifiers of the [Actor].
    pub fn create_uuid(&self) -> Result<Uuid> {
        self.id().map(|id| self.table().uuid_from_id(id))
    }

    /// Gets the ID used to fetch the actor record.
    pub fn id(&self) -> Result<&Iri> {
        match self {
            Self::Application(actor) => actor.id().ok_or(Error::actor("application: missing ID")),
            Self::Factory(actor) => actor.id().ok_or(Error::actor("factory: missing ID")),
            Self::Person(actor) => actor.id().ok_or(Error::actor("person: missing ID")),
            Self::Repository(actor) => actor.id().ok_or(Error::actor("repository: missing ID")),
            Self::Team(actor) => actor.id().ok_or(Error::actor("team: missing ID")),
            Self::PatchTracker(actor) => {
                actor.id().ok_or(Error::actor("patch_tracker: missing ID"))
            }
            Self::TicketTracker(actor) => {
                actor.id().ok_or(Error::actor("ticket_tracker: missing ID"))
            }
        }
    }

    /// Attempts to convert an [Item] into an [Actor].
    pub fn from_item(table: TableType, item: &Item) -> Result<Self> {
        let id = if let Ok(iri) = item.as_iri() {
            iri.clone()
        } else if let Ok(link) = item.as_link() {
            link.href().clone()
        } else {
            return item.try_into();
        };

        match table {
            TableType::Factory => Ok(Self::factory(Factory::new().with_id(id))),
            TableType::Repository => Ok(Self::repository(Repository::new().with_id(id))),
            TableType::Team => Ok(Self::team(Team::new().with_id(id))),
            TableType::PatchTracker => Ok(Self::patchtracker(PatchTracker::new().with_id(id))),
            TableType::TicketTracker => Ok(Self::tickettracker(TicketTracker::new().with_id(id))),
            TableType::Application => Ok(Self::application(Application::new().with_id(id))),
            TableType::Person => Ok(Self::person(Person::new().with_id(id))),
            _ => Err(Error::actor(format!("unsupported database type: {table}"))),
        }
    }

    /// Attempts to convert an [Items] element into an [Actor].
    pub fn from_items(table: TableType, items: &Items) -> Result<Self> {
        match items {
            Items::Single(item) => Self::from_item(table, item),
            Items::List(list) => {
                let actor_type = ActorType::try_from(table)?;

                if let Some(item) = list.iter().find(|i| {
                    if let Ok(obj) = i.as_object()
                        && obj.kind().contains(actor_type)
                    {
                        true
                    } else {
                        i.is_link() || i.is_iri()
                    }
                }) {
                    Self::from_item(table, item)
                } else {
                    Err(Error::actor(format!(
                        "no valid item found for actor type: {actor_type}"
                    )))
                }
            }
        }
    }
}

impl From<&Actor> for Item {
    fn from(val: &Actor) -> Self {
        match val {
            Actor::Factory(actor) => actor.as_ref().clone().into(),
            Actor::Repository(actor) => actor.as_ref().clone().into(),
            Actor::Team(actor) => actor.as_ref().clone().into(),
            Actor::PatchTracker(actor) => actor.as_ref().clone().into(),
            Actor::TicketTracker(actor) => actor.as_ref().clone().into(),
            Actor::Application(actor) => actor.as_ref().clone().into(),
            Actor::Person(actor) => actor.as_ref().clone().into(),
        }
    }
}

impl From<Actor> for Item {
    fn from(val: Actor) -> Self {
        match val {
            Actor::Factory(actor) => (*actor).into(),
            Actor::Repository(actor) => (*actor).into(),
            Actor::Team(actor) => (*actor).into(),
            Actor::PatchTracker(actor) => (*actor).into(),
            Actor::TicketTracker(actor) => (*actor).into(),
            Actor::Application(actor) => (*actor).into(),
            Actor::Person(actor) => (*actor).into(),
        }
    }
}

impl TryFrom<Item> for Actor {
    type Error = Error;

    fn try_from(val: Item) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Item> for Actor {
    type Error = Error;

    fn try_from(val: &Item) -> Result<Self> {
        match val {
            Item::Object(obj) if obj.kind().contains(ActorType::Factory) => Factory::try_from(val)
                .map(Self::factory)
                .map_err(Error::from),
            Item::Object(obj) if obj.kind().contains(ActorType::Repository) => {
                Repository::try_from(val)
                    .map(Self::repository)
                    .map_err(Error::from)
            }
            Item::Object(obj) if obj.kind().contains(ActorType::Team) => {
                Team::try_from(val).map(Self::team).map_err(Error::from)
            }
            Item::Object(obj) if obj.kind().contains(ActorType::PatchTracker) => {
                PatchTracker::try_from(val)
                    .map(Self::patchtracker)
                    .map_err(Error::from)
            }
            Item::Object(obj) if obj.kind().contains(ActorType::TicketTracker) => {
                TicketTracker::try_from(val)
                    .map(Self::tickettracker)
                    .map_err(Error::from)
            }
            Item::Object(obj) if obj.kind().contains(VocabActorType::Application) => {
                Application::try_from(val)
                    .map(Self::application)
                    .map_err(Error::from)
            }
            Item::Object(obj) if obj.kind().contains(VocabActorType::Person) => {
                Person::try_from(val).map(Self::person).map_err(Error::from)
            }
            _ => Err(Error::actor(format!("invalid actor: {val}"))),
        }
    }
}

impl TryFrom<Items> for Actor {
    type Error = Error;

    fn try_from(val: Items) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Items> for Actor {
    type Error = Error;

    fn try_from(val: &Items) -> Result<Self> {
        match val {
            Items::Single(item) => item.try_into(),
            Items::List(list) => list
                .iter()
                .find(|l| {
                    l.as_object()
                        .map(|obj| {
                            let kind = obj.kind();

                            kind.contains(ActorType::Factory)
                                || kind.contains(ActorType::Repository)
                                || kind.contains(ActorType::Team)
                                || kind.contains(ActorType::PatchTracker)
                                || kind.contains(ActorType::TicketTracker)
                                || kind.contains(VocabActorType::Application)
                                || kind.contains(VocabActorType::Person)
                        })
                        .unwrap_or_default()
                })
                .ok_or(Error::actor("no valid actor found"))
                .and_then(|item| item.try_into()),
        }
    }
}
