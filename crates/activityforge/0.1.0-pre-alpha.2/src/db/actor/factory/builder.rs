use serde::{Deserialize, Serialize};

use crate::db::{
    ActorType, Collaborator, Db, Factory, Follower, Grant, Inbox, Iri, Key, Name, Outbox,
    TableEntry, TableType, Team, Uuid,
};
use crate::{Error, Result, Role, util};

/// Represents a builder struct for a [Factory].
///
/// Allows for users to have a high-level interface for creating all of the
/// sub-record entries that make up a [Factory] entry.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct FactoryBuilder {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    inbox: Inbox,
    outbox: Outbox,
    available_actor_types: Vec<ActorType>,
    keys: Vec<Key>,
    collaborators: Vec<Collaborator>,
    followers: Vec<Follower>,
    teams: Vec<Team>,
}

impl FactoryBuilder {
    /// Creates a new [FactoryBuilder].
    ///
    /// Creates a default inbox and outbox based on the supplied `id`.
    pub fn new<I: Into<Iri>, N: Into<Name>>(id: I, name: N) -> Result<Self> {
        let uuid = util::rand_uuid();

        let id = Factory::TABLE.id_from_uuid(&id.into(), uuid)?;
        let name = name.into();

        let inbox_id = Iri::try_from(format!("{id}/inbox"))?;
        let outbox_id = Iri::try_from(format!("{id}/outbox"))?;

        let inbox = Inbox::new()
            .with_uuid(uuid)
            .with_id(inbox_id)
            .with_actor(TableEntry::create(TableType::Factory, uuid));

        let outbox = Outbox::new()
            .with_uuid(uuid)
            .with_id(outbox_id)
            .with_actor(TableEntry::create(TableType::Factory, uuid));

        Ok(Self {
            uuid,
            id,
            name,
            inbox,
            outbox,
            available_actor_types: Vec::new(),
            keys: Vec::new(),
            collaborators: Vec::new(),
            followers: Vec::new(),
            teams: Vec::new(),
        })
    }

    /// Gets the table type.
    #[inline]
    pub const fn table() -> TableType {
        Factory::TABLE
    }

    /// Gets the table entry.
    #[inline]
    pub const fn table_entry(&self) -> TableEntry {
        TableEntry::create(Self::table(), self.uuid)
    }

    /// Sets the [Uuid] for the [Factory].
    ///
    /// Optional, a random [Uuid] is assigned in [FactoryBuilder::new].
    pub fn uuid(self, uuid: Uuid) -> Result<Self> {
        Self::table()
            .id_from_uuid(&self.id, uuid)
            .map(|id| {
                Self {
                    uuid,
                    id,
                    name: self.name,
                    available_actor_types: self.available_actor_types,
                    inbox: Inbox::new(),
                    outbox: Outbox::new(),
                    keys: Vec::new(),
                    collaborators: Vec::new(),
                    followers: Vec::new(),
                    teams: Vec::new(),
                }
                .inbox(self.inbox)
                .outbox(self.outbox)
                .collaborators(self.collaborators)
                .followers(self.followers)
                .teams(self.teams)
            })
            .and_then(|s| s.keys(self.keys))
    }

    /// Sets the ID IRI for the [Factory].
    ///
    /// Optional, set a different ID than provided in [FactoryBuilder::new].
    pub fn id<I: Into<Iri>>(self, id: I) -> Result<Self> {
        Self::table()
            .id_from_uuid(&id.into(), self.uuid)
            .map(|id| Self { id, ..self })
    }

    /// Sets the name for the [Factory].
    ///
    /// Optional, set a different name than provided in [FactoryBuilder::new].
    pub fn name<N: Into<Name>>(self, name: N) -> Self {
        Self {
            name: name.into(),
            ..self
        }
    }

    /// Sets the [Inbox] for the [Factory].
    ///
    /// Only needed if the default [Inbox] created by [FactoryBuilder::new] is incorrect.
    pub fn inbox(self, inbox: Inbox) -> Self {
        let actor = self.table_entry();

        let inbox = if inbox.actor() != actor {
            inbox.with_actor(actor)
        } else {
            inbox
        };

        Self { inbox, ..self }
    }

    /// Sets the [Outbox] for the [Factory].
    ///
    /// Only needed if the default [Outbox] created by [FactoryBuilder::new] is incorrect.
    pub fn outbox(self, outbox: Outbox) -> Self {
        let actor = self.table_entry();

        let outbox = if outbox.actor() != actor {
            outbox.with_actor(actor)
        } else {
            outbox
        };

        Self { outbox, ..self }
    }

    /// Sets the list of available actor types that the [Factory] can create.
    pub fn available_actor_types<T, I>(self, types: I) -> Self
    where
        T: Into<ActorType>,
        I: IntoIterator<Item = T>,
    {
        Self {
            available_actor_types: types.into_iter().map(|i| i.into()).collect(),
            ..self
        }
    }

    /// Sets the list of [Factory] [Key]s.
    pub fn keys<T, I>(self, val: I) -> Result<Self>
    where
        T: Into<Key>,
        I: IntoIterator<Item = T>,
    {
        let actor = self.table_entry();
        let table = TableType::Key;

        val.into_iter()
            .map(|i| {
                let key = i.into();

                table.id_from_uuid(key.id(), key.uuid()).map(|id| {
                    if key.actor() == actor {
                        key.with_id(id).with_actor_id(&self.id)
                    } else {
                        key.with_id(id).with_actor_id(&self.id).with_actor(actor)
                    }
                })
            })
            .collect::<Result<Vec<Key>>>()
            .map(|keys| Self { keys, ..self })
    }

    /// Sets the list of [Factory] [Collaborator]s.
    pub fn collaborators<T, I>(self, val: I) -> Self
    where
        T: Into<Collaborator>,
        I: IntoIterator<Item = T>,
    {
        let collaborators = val
            .into_iter()
            .map(|i| {
                let key = i.into();

                if key.subject() == &self.id {
                    key
                } else {
                    key.with_subject(&self.id)
                }
            })
            .collect();

        Self {
            collaborators,
            ..self
        }
    }

    /// Sets the list of [Factory] [Follower]s.
    pub fn followers<T, I>(self, val: I) -> Self
    where
        T: Into<Follower>,
        I: IntoIterator<Item = T>,
    {
        let actor = self.table_entry();

        let followers = val
            .into_iter()
            .map(|i| {
                let mut follower = i.into();

                if follower.following().contains(&actor) {
                    follower
                } else {
                    follower.add_following_entry(actor).ok();
                    follower
                }
            })
            .collect();

        Self { followers, ..self }
    }

    /// Sets the list of [Factory] [Team]s.
    pub fn teams<T, I>(self, types: I) -> Self
    where
        T: Into<Team>,
        I: IntoIterator<Item = T>,
    {
        Self {
            teams: types.into_iter().map(|i| i.into()).collect(),
            ..self
        }
    }

    /// Builds the [Factory] record, and inserts all sub-records into the database.
    pub async fn build(mut self, db: &Db) -> Result<Factory> {
        let uuid = self.uuid;

        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let inbox_uuid = self.inbox.find_or_create_tx(&mut dbtx).await?;

        let mailbox_roles = [Role::Visit, Role::Write];

        let inbox_grant_id = Grant::TABLE.id_from_uuid(&self.id, db.rand_uuid())?;

        let mut inbox_grant = Grant::new()
            .with_id(inbox_grant_id)
            .with_actor(self.table_entry())
            .with_context(self.inbox.table_entry())
            .with_objects(mailbox_roles)?;

        inbox_grant.find_or_create_tx(&mut dbtx).await?;

        let outbox_uuid = self.outbox.find_or_create_tx(&mut dbtx).await?;
        let outbox_grant_id = Grant::TABLE.id_from_uuid(&self.id, db.rand_uuid())?;

        let mut outbox_grant = Grant::new()
            .with_id(outbox_grant_id)
            .with_actor(self.table_entry())
            .with_context(self.outbox.table_entry())
            .with_objects(mailbox_roles)?;

        outbox_grant.find_or_create_tx(&mut dbtx).await?;

        for key in self.keys.iter_mut() {
            key.find_or_create_tx(&mut dbtx, &db_key).await?;
        }

        for collaborator in self.collaborators.iter_mut() {
            collaborator.find_or_create_tx(&mut dbtx).await?;
        }

        for follower in self.followers.iter_mut() {
            follower.find_or_create_tx(&mut dbtx).await?;
        }

        for team in self.teams.iter_mut() {
            team.find_or_create_tx(&mut dbtx).await?;
        }

        let key_ids = self.keys.into_iter().map(|k| k.uuid()).collect::<Vec<_>>();

        let collaborators = self
            .collaborators
            .into_iter()
            .map(|c| c.uuid())
            .collect::<Vec<_>>();

        let followers = self
            .followers
            .into_iter()
            .map(|f| f.uuid())
            .collect::<Vec<_>>();

        let teams = self.teams.into_iter().map(|t| t.uuid()).collect::<Vec<_>>();

        let mut factory = Factory::new()
            .with_uuid(uuid)
            .with_id(self.id)
            .with_name(self.name)
            .with_inbox(inbox_uuid)
            .with_outbox(outbox_uuid)
            .with_key_ids(key_ids)
            .and_then(|f| f.with_available_actor_types(self.available_actor_types))
            .and_then(|f| f.with_collaborators(collaborators))
            .and_then(|f| f.with_followers(followers))
            .and_then(|f| f.with_teams(teams))?;

        factory.find_or_create_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| factory).map_err(Error::from)
    }
}
