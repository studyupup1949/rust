//! Database `Actor` records.

use activitystreams_vocabulary::{Item, Items};

use crate::crypto::SymmetricKey;
use crate::db::{ActorType, Db, Iri, TableEntry, TableType, Transaction, Uuid};
use crate::{Actor as VocabActor, Error, Result};

mod application;
mod factory;
mod patch_tracker;
mod person;
mod repository;
mod team;
mod ticket_tracker;

pub use application::{Application, ApplicationBuilder};
pub use factory::{Factory, FactoryBuilder};
pub use patch_tracker::{PatchTracker, PatchTrackerBuilder};
pub use person::{Person, PersonBuilder};
pub use repository::{Repository, RepositoryBuilder};
pub use team::{RoleFilter, Team, TeamBuilder};
pub use ticket_tracker::{TicketTracker, TicketTrackerBuilder};

/// Represents the `Actor` record variants stored in the database.
#[derive(Clone, Eq, PartialEq)]
pub enum Actor {
    Application(Box<Application>),
    Factory(Box<Factory>),
    Person(Box<Person>),
    Repository(Box<Repository>),
    Team(Box<Team>),
    PatchTracker(Box<PatchTracker>),
    TicketTracker(Box<TicketTracker>),
}

impl Actor {
    /// Creates a new [Actor] with an ID.
    pub fn new_with_id<A, I>(actor: A, id: I) -> Result<Self>
    where
        A: Into<ActorType>,
        I: Into<Iri>,
    {
        match actor.into() {
            ActorType::Application => Ok(Self::application(Application::new().with_id(id))),
            ActorType::Factory => Ok(Self::factory(Factory::new().with_id(id))),
            ActorType::Person => Ok(Self::person(Person::new().with_id(id))),
            ActorType::Repository => Ok(Self::repository(Repository::new().with_id(id))),
            ActorType::Team => Ok(Self::team(Team::new().with_id(id))),
            ActorType::PatchTracker => Ok(Self::patch_tracker(PatchTracker::new().with_id(id))),
            ActorType::TicketTracker => Ok(Self::ticket_tracker(TicketTracker::new().with_id(id))),
            actor => Err(Error::db(format!("actor: unsupported actor type: {actor}"))),
        }
    }

    /// Gets the actor record table.
    #[inline]
    pub const fn table(&self) -> TableType {
        match self {
            Self::Application(actor) => actor.table(),
            Self::Factory(actor) => actor.table(),
            Self::Person(actor) => actor.table(),
            Self::Repository(actor) => actor.table(),
            Self::Team(actor) => actor.table(),
            Self::PatchTracker(actor) => actor.table(),
            Self::TicketTracker(actor) => actor.table(),
        }
    }

    /// Gets the actor record UUID.
    pub fn uuid(&self) -> Uuid {
        match self {
            Self::Application(actor) => actor.uuid(),
            Self::Factory(actor) => actor.uuid(),
            Self::Person(actor) => actor.uuid(),
            Self::Repository(actor) => actor.uuid(),
            Self::Team(actor) => actor.uuid(),
            Self::PatchTracker(actor) => actor.uuid(),
            Self::TicketTracker(actor) => actor.uuid(),
        }
    }

    /// Creates a UUID based on unqiue identifiers of the [Actor].
    #[inline]
    pub fn create_uuid(&self) -> Uuid {
        self.table().uuid_from_id(self.id())
    }

    /// Gets the ID used to fetch the actor record.
    pub fn id(&self) -> &Iri {
        match self {
            Self::Application(actor) => actor.id(),
            Self::Factory(actor) => actor.id(),
            Self::Person(actor) => actor.id(),
            Self::Repository(actor) => actor.id(),
            Self::Team(actor) => actor.id(),
            Self::PatchTracker(actor) => actor.id(),
            Self::TicketTracker(actor) => actor.id(),
        }
    }

    /// Gets the table entry referencing the actor record.
    pub fn table_entry(&self) -> TableEntry {
        match self {
            Self::Application(actor) => actor.table_entry(),
            Self::Factory(actor) => actor.table_entry(),
            Self::Person(actor) => actor.table_entry(),
            Self::Repository(actor) => actor.table_entry(),
            Self::Team(actor) => actor.table_entry(),
            Self::PatchTracker(actor) => actor.table_entry(),
            Self::TicketTracker(actor) => actor.table_entry(),
        }
    }

    /// Gets the UUID referencing the actor's inbox.
    pub fn inbox(&self) -> Uuid {
        match self {
            Self::Application(actor) => actor.inbox(),
            Self::Factory(actor) => actor.inbox(),
            Self::Person(actor) => actor.inbox(),
            Self::Repository(actor) => actor.inbox(),
            Self::Team(actor) => actor.inbox(),
            Self::PatchTracker(actor) => actor.inbox(),
            Self::TicketTracker(actor) => actor.inbox(),
        }
    }

    /// Gets the table entry referencing the actor's inbox.
    pub fn inbox_entry(&self) -> TableEntry {
        TableEntry::create(TableType::Inbox, self.inbox())
    }

    /// Gets the UUID referencing the actor's outbox.
    pub fn outbox(&self) -> Uuid {
        match self {
            Self::Application(actor) => actor.outbox(),
            Self::Factory(actor) => actor.outbox(),
            Self::Person(actor) => actor.outbox(),
            Self::Repository(actor) => actor.outbox(),
            Self::Team(actor) => actor.outbox(),
            Self::PatchTracker(actor) => actor.outbox(),
            Self::TicketTracker(actor) => actor.outbox(),
        }
    }

    /// Gets the table entry referencing the actor's outbox.
    pub fn outbox_entry(&self) -> TableEntry {
        TableEntry::create(TableType::Outbox, self.outbox())
    }

    /// Converts an [Actor](VocabActor) JSON-LD object into an [Actor] record.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab(db: &Db, val: &VocabActor) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let actor = Self::try_from_vocab_tx(&mut dbtx, &db_key, val).await?;

        dbtx.commit()
            .await
            .map(|_| actor)
            .map_err(|err| Error::db(format!("actor: {err}")))
    }

    /// Converts a [Actor](VocabActor) JSON-LD object into an [Actor] record using a transaction.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        val: &VocabActor,
    ) -> Result<Self> {
        let uuid = val.create_uuid()?;

        log::debug!("actor: trying to convert actor for {}", val.id()?);
        log::debug!("actor: storing converted actor at {uuid}");

        match val {
            VocabActor::Application(actor) => {
                Application::try_from_vocab_with_uuid_tx(dbtx, db_key, actor, uuid)
                    .await
                    .map(Self::application)
            }
            VocabActor::Factory(actor) => {
                Factory::try_from_vocab_with_uuid_tx(dbtx, db_key, actor, uuid)
                    .await
                    .map(Self::factory)
            }
            VocabActor::Person(actor) => {
                Person::try_from_vocab_with_uuid_tx(dbtx, db_key, actor, uuid)
                    .await
                    .map(Self::person)
            }
            VocabActor::Repository(actor) => {
                Repository::try_from_vocab_with_uuid_tx(dbtx, db_key, actor, uuid)
                    .await
                    .map(Self::repository)
            }
            VocabActor::Team(actor) => Team::try_from_vocab_with_uuid_tx(dbtx, db_key, actor, uuid)
                .await
                .map(Self::team),
            VocabActor::PatchTracker(actor) => {
                PatchTracker::try_from_vocab_with_uuid_tx(dbtx, db_key, actor, uuid)
                    .await
                    .map(Self::patch_tracker)
            }
            VocabActor::TicketTracker(actor) => {
                TicketTracker::try_from_vocab_with_uuid_tx(dbtx, db_key, actor, uuid)
                    .await
                    .map(Self::ticket_tracker)
            }
        }
    }

    /// Converts an [Item] JSON-LD object into an [Actor] record.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_item(db: &Db, val: &Item) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let actor = Self::try_from_item_tx(&mut dbtx, &db_key, val).await?;

        dbtx.commit()
            .await
            .map(|_| actor)
            .map_err(|err| Error::db(format!("actor: {err}")))
    }

    /// Converts an [Item] JSON-LD object into an [Actor] record using a transaction.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_item_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        val: &Item,
    ) -> Result<Self> {
        let actor = VocabActor::try_from(val)?;
        Self::try_from_vocab_tx(dbtx, db_key, &actor).await
    }

    /// Converts an [Items] JSON-LD object into an [Actor] record.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_items(db: &Db, val: &Items) -> Result<Vec<Self>> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let actors = Self::try_from_items_tx(&mut dbtx, &db_key, val).await?;

        dbtx.commit()
            .await
            .map(|_| actors)
            .map_err(|err| Error::db(format!("actor: {err}")))
    }

    /// Converts an [Items] JSON-LD object into an [Actor] record using a transaction.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_items_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        val: &Items,
    ) -> Result<Vec<Self>> {
        match val {
            Items::Single(item) => Self::try_from_item_tx(dbtx, db_key, item)
                .await
                .map(|i| vec![i]),
            Items::List(list) => {
                let mut actors = Vec::new();

                for item in list.iter() {
                    let actor = Self::try_from_item_tx(dbtx, db_key, item).await?;
                    actors.push(actor);
                }

                Ok(actors)
            }
        }
    }

    /// Attempts to find an actor record by key ID.
    pub async fn find_by_key_id(db: &Db, key_id: &Iri) -> Result<Option<Self>> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let actor = Self::find_by_key_id_tx(&mut dbtx, key_id).await?;

        dbtx.commit().await.map(|_| actor).map_err(Error::from)
    }

    /// Attempts to find an actor record by key ID using a transaction.
    pub async fn find_by_key_id_tx(
        dbtx: &mut Transaction<'_>,
        key_id: &Iri,
    ) -> Result<Option<Self>> {
        log::debug!("actor: looking up actor for key ID: {key_id}");

        if let Some(actor) = Application::find_by_key_id_tx(dbtx, key_id).await? {
            Ok(Some(Self::application(actor)))
        } else if let Some(actor) = Factory::find_by_key_id_tx(dbtx, key_id).await? {
            Ok(Some(Self::factory(actor)))
        } else if let Some(actor) = Person::find_by_key_id_tx(dbtx, key_id).await? {
            Ok(Some(Self::person(actor)))
        } else if let Some(actor) = Repository::find_by_key_id_tx(dbtx, key_id).await? {
            Ok(Some(Self::repository(actor)))
        } else if let Some(actor) = Team::find_by_key_id_tx(dbtx, key_id).await? {
            Ok(Some(Self::team(actor)))
        } else if let Some(actor) = PatchTracker::find_by_key_id_tx(dbtx, key_id).await? {
            Ok(Some(Self::patch_tracker(actor)))
        } else if let Some(actor) = TicketTracker::find_by_key_id_tx(dbtx, key_id).await? {
            Ok(Some(Self::ticket_tracker(actor)))
        } else {
            Ok(None)
        }
    }

    /// Attempts to find an actor record by ID.
    pub async fn find_by_id(db: &Db, id: &Iri) -> Result<Option<Self>> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let actor = Self::find_by_id_tx(&mut dbtx, id).await?;

        dbtx.commit().await.map(|_| actor).map_err(Error::from)
    }

    /// Attempts to find an actor record by ID using a transaction.
    pub async fn find_by_id_tx(dbtx: &mut Transaction<'_>, id: &Iri) -> Result<Option<Self>> {
        log::debug!("actor: looking up actor for ID: {id}");

        if let Some(actor) = Application::find_by_id_tx(dbtx, id).await? {
            Ok(Some(Self::application(actor)))
        } else if let Some(actor) = Factory::find_by_id_tx(dbtx, id).await? {
            Ok(Some(Self::factory(actor)))
        } else if let Some(actor) = Person::find_by_id_tx(dbtx, id).await? {
            Ok(Some(Self::person(actor)))
        } else if let Some(actor) = Repository::find_by_id_tx(dbtx, id).await? {
            Ok(Some(Self::repository(actor)))
        } else if let Some(actor) = Team::find_by_id_tx(dbtx, id).await? {
            Ok(Some(Self::team(actor)))
        } else if let Some(actor) = PatchTracker::find_by_id_tx(dbtx, id).await? {
            Ok(Some(Self::patch_tracker(actor)))
        } else if let Some(actor) = TicketTracker::find_by_id_tx(dbtx, id).await? {
            Ok(Some(Self::ticket_tracker(actor)))
        } else {
            Ok(None)
        }
    }

    /// Find or create an [Actor] record.
    pub async fn find_or_create(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let uuid = self.find_or_create_tx(&mut dbtx).await?;

        dbtx.commit()
            .await
            .map(|_| uuid)
            .map_err(|err| Error::db(format!("actor: {err}")))
    }

    /// Find or create an [Actor] record using a transaction.
    pub async fn find_or_create_tx(&mut self, dbtx: &mut Transaction<'_>) -> Result<Uuid> {
        match self {
            Self::Application(actor) => actor.find_or_create_tx(dbtx).await,
            Self::Factory(actor) => actor.find_or_create_tx(dbtx).await,
            Self::Person(actor) => actor.find_or_create_tx(dbtx).await,
            Self::Repository(actor) => actor.find_or_create_tx(dbtx).await,
            Self::Team(actor) => actor.find_or_create_tx(dbtx).await,
            Self::PatchTracker(actor) => actor.find_or_create_tx(dbtx).await,
            Self::TicketTracker(actor) => actor.find_or_create_tx(dbtx).await,
        }
    }

    /// Gets the [ActorType].
    pub const fn actor_type(&self) -> ActorType {
        match self {
            Self::Application(_) => ActorType::Application,
            Self::Factory(_) => ActorType::Factory,
            Self::Person(_) => ActorType::Person,
            Self::Repository(_) => ActorType::Repository,
            Self::Team(_) => ActorType::Team,
            Self::PatchTracker(_) => ActorType::PatchTracker,
            Self::TicketTracker(_) => ActorType::TicketTracker,
        }
    }

    /// Gets whether the [Actor] is a valid `object` of a [Like](activitystreams_vocabulary::Like) activity.
    pub const fn is_valid_like_object(&self) -> bool {
        // TODO: add `Project` type when implemented
        // all other types are not valid `Like` activity objects
        matches!(self, Self::Repository(_))
    }

    /// Creates a a new [Application](Self::Application) variant.
    #[inline]
    pub fn application<I: Into<Application>>(application: I) -> Self {
        Self::Application(Box::new(application.into()))
    }

    /// Attempts to get a reference to the [Application](Self::Application) variant.
    pub fn as_application(&self) -> Result<&Application> {
        match self {
            Self::Application(actor) => Ok(actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Attempts to convert to the [Application](Self::Application) variant.
    pub fn into_application(self) -> Result<Application> {
        match self {
            Self::Application(actor) => Ok(*actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Creates a a new [Factory](Self::Factory) variant.
    #[inline]
    pub fn factory<I: Into<Factory>>(factory: I) -> Self {
        Self::Factory(Box::new(factory.into()))
    }

    /// Attempts to get a reference to the [Factory](Self::Factory) variant.
    pub fn as_factory(&self) -> Result<&Factory> {
        match self {
            Self::Factory(actor) => Ok(actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Attempts to convert to the [Factory](Self::Factory) variant.
    pub fn into_factory(self) -> Result<Factory> {
        match self {
            Self::Factory(actor) => Ok(*actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Creates a a new [Person](Self::Person) variant.
    #[inline]
    pub fn person<I: Into<Person>>(person: I) -> Self {
        Self::Person(Box::new(person.into()))
    }

    /// Attempts to get a reference to the [Person](Self::Person) variant.
    pub fn as_person(&self) -> Result<&Person> {
        match self {
            Self::Person(actor) => Ok(actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Attempts to convert to the [Person](Self::Person) variant.
    pub fn into_person(self) -> Result<Person> {
        match self {
            Self::Person(actor) => Ok(*actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Creates a a new [Repository](Self::Repository) variant.
    #[inline]
    pub fn repository<I: Into<Repository>>(repository: I) -> Self {
        Self::Repository(Box::new(repository.into()))
    }

    /// Attempts to get a reference to the [Repository](Self::Repository) variant.
    pub fn as_repository(&self) -> Result<&Repository> {
        match self {
            Self::Repository(actor) => Ok(actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Attempts to convert to the [Repository](Self::Repository) variant.
    pub fn into_repository(self) -> Result<Repository> {
        match self {
            Self::Repository(actor) => Ok(*actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Creates a a new [Team](Self::Team) variant.
    #[inline]
    pub fn team<I: Into<Team>>(team: I) -> Self {
        Self::Team(Box::new(team.into()))
    }

    /// Attempts to get a reference to the [Team](Self::Team) variant.
    pub fn as_team(&self) -> Result<&Team> {
        match self {
            Self::Team(actor) => Ok(actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Attempts to convert to the [Team](Self::Team) variant.
    pub fn into_team(self) -> Result<Team> {
        match self {
            Self::Team(actor) => Ok(*actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Creates a a new [PatchTracker](Self::PatchTracker) variant.
    #[inline]
    pub fn patch_tracker<I: Into<PatchTracker>>(patch_tracker: I) -> Self {
        Self::PatchTracker(Box::new(patch_tracker.into()))
    }

    /// Attempts to get a reference to the [PatchTracker](Self::PatchTracker) variant.
    pub fn as_patch_tracker(&self) -> Result<&PatchTracker> {
        match self {
            Self::PatchTracker(actor) => Ok(actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Attempts to convert to the [PatchTracker](Self::PatchTracker) variant.
    pub fn into_patch_tracker(self) -> Result<PatchTracker> {
        match self {
            Self::PatchTracker(actor) => Ok(*actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Creates a a new [TicketTracker](Self::TicketTracker) variant.
    #[inline]
    pub fn ticket_tracker<I: Into<TicketTracker>>(ticket_tracker: I) -> Self {
        Self::TicketTracker(Box::new(ticket_tracker.into()))
    }

    /// Attempts to get a reference to the [TicketTracker](Self::TicketTracker) variant.
    pub fn as_ticket_tracker(&self) -> Result<&TicketTracker> {
        match self {
            Self::TicketTracker(actor) => Ok(actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }

    /// Attempts to convert to the [TicketTracker](Self::TicketTracker) variant.
    pub fn into_ticket_tracker(self) -> Result<TicketTracker> {
        match self {
            Self::TicketTracker(actor) => Ok(*actor),
            _ => Err(Error::db("invalid actor variant")),
        }
    }
}
