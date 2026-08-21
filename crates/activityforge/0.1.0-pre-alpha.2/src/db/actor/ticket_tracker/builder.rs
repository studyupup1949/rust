use serde::{Deserialize, Serialize};

use crate::db::{
    Db, Follower, Inbox, Iri, Key, Name, Outbox, TableEntry, TableType, TicketTracker, Uuid,
};
use crate::{Error, Result, util};

/// Represents a builder struct for a [TicketTracker].
///
/// Allows for users to have a high-level interface for creating all of the
/// sub-record entries that make up a [TicketTracker] entry.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct TicketTrackerBuilder {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    inbox: Inbox,
    outbox: Outbox,
    summary: Option<String>,
    content: Option<String>,
    keys: Vec<Key>,
    followers: Vec<Follower>,
}

impl TicketTrackerBuilder {
    /// Creates a new [TicketTrackerBuilder].
    ///
    /// Creates a default inbox and outbox based on the supplied `id`.
    pub fn new<I: Into<Iri>, N: Into<Name>>(id: I, name: N) -> Result<Self> {
        let uuid = util::rand_uuid();

        let id = TicketTracker::TABLE.id_from_uuid(&id.into(), uuid)?;
        let name = name.into();

        let inbox_id = Iri::try_from(format!("{id}/inbox"))?;
        let outbox_id = Iri::try_from(format!("{id}/outbox"))?;

        let actor = TableEntry::create(Self::table(), uuid);

        let inbox = Inbox::new()
            .with_uuid(util::rand_uuid())
            .with_id(inbox_id)
            .with_actor(actor);

        let outbox = Outbox::new()
            .with_uuid(util::rand_uuid())
            .with_id(outbox_id)
            .with_actor(actor);

        Ok(Self {
            uuid,
            id,
            name,
            inbox,
            outbox,
            summary: None,
            content: None,
            keys: Vec::new(),
            followers: Vec::new(),
        })
    }

    /// Gets the table type.
    #[inline]
    pub const fn table() -> TableType {
        TicketTracker::TABLE
    }

    /// Gets the table entry.
    #[inline]
    pub const fn table_entry(&self) -> TableEntry {
        TableEntry::create(Self::table(), self.uuid)
    }

    /// Sets the [Uuid] for the [TicketTracker].
    ///
    /// Optional, a random [Uuid] is assigned in [TicketTrackerBuilder::new].
    pub fn uuid(self, uuid: Uuid) -> Result<Self> {
        Self::table()
            .id_from_uuid(&self.id, uuid)
            .map(|id| {
                Self {
                    uuid,
                    id,
                    name: self.name,
                    summary: self.summary,
                    content: self.content,
                    inbox: Inbox::new(),
                    outbox: Outbox::new(),
                    keys: Vec::new(),
                    followers: Vec::new(),
                }
                .inbox(self.inbox)
                .outbox(self.outbox)
                .followers(self.followers)
            })
            .and_then(|s| s.keys(self.keys))
    }

    /// Sets the ID IRI for the [TicketTracker].
    ///
    /// Optional, set a different ID than provided in [TicketTrackerBuilder::new].
    pub fn id<I: Into<Iri>>(self, id: I) -> Result<Self> {
        Self::table()
            .id_from_uuid(&id.into(), self.uuid)
            .map(|id| Self { id, ..self })
    }

    /// Sets the name for the [TicketTracker].
    ///
    /// Optional, set a different name than provided in [TicketTrackerBuilder::new].
    pub fn name<N: Into<Name>>(self, name: N) -> Self {
        Self {
            name: name.into(),
            ..self
        }
    }

    /// Sets the content for the [TicketTracker].
    pub fn content<I: Into<String>>(self, content: I) -> Self {
        Self {
            content: Some(content.into()),
            ..self
        }
    }

    /// Sets the summary for the [TicketTracker].
    pub fn summary<I: Into<String>>(self, summary: I) -> Self {
        Self {
            summary: Some(summary.into()),
            ..self
        }
    }

    /// Sets the [Inbox] for the [TicketTracker].
    ///
    /// Only needed if the default [Inbox] created by [TicketTrackerBuilder::new] is incorrect.
    pub fn inbox(self, inbox: Inbox) -> Self {
        let actor = self.table_entry();

        let inbox = if inbox.actor() != actor {
            inbox.with_actor(actor)
        } else {
            inbox
        };

        Self { inbox, ..self }
    }

    /// Sets the [Outbox] for the [TicketTracker].
    ///
    /// Only needed if the default [Outbox] created by [TicketTrackerBuilder::new] is incorrect.
    pub fn outbox(self, outbox: Outbox) -> Self {
        let actor = self.table_entry();

        let outbox = if outbox.actor() != actor {
            outbox.with_actor(actor)
        } else {
            outbox
        };

        Self { outbox, ..self }
    }

    /// Sets the list of [TicketTracker] [Key]s.
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

    /// Sets the list of [TicketTracker] [Follower]s.
    pub fn followers<T, I>(self, val: I) -> Self
    where
        T: Into<Follower>,
        I: IntoIterator<Item = T>,
    {
        let actor = self.table_entry();

        let mut followers: Vec<Follower> = val
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

        followers.sort();
        followers.dedup();

        Self { followers, ..self }
    }

    /// Builds the [TicketTracker] record, and inserts all sub-records into the database.
    pub async fn build(mut self, db: &Db) -> Result<TicketTracker> {
        let uuid = self.uuid;

        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let inbox_uuid = self.inbox.find_or_create_tx(&mut dbtx).await?;
        let outbox_uuid = self.outbox.find_or_create_tx(&mut dbtx).await?;

        for key in self.keys.iter_mut() {
            key.find_or_create_tx(&mut dbtx, &db_key).await?;
        }

        for follower in self.followers.iter_mut() {
            follower.find_or_create_tx(&mut dbtx).await?;
        }

        let key_ids = self.keys.into_iter().map(|k| k.uuid()).collect::<Vec<_>>();

        let followers = self
            .followers
            .into_iter()
            .map(|f| f.uuid())
            .collect::<Vec<_>>();

        let mut ticket_tracker = TicketTracker::new()
            .with_uuid(uuid)
            .with_id(self.id)
            .with_name(self.name)
            .with_inbox(inbox_uuid)
            .with_outbox(outbox_uuid)
            .with_key_ids(key_ids)
            .and_then(|p| p.with_followers(followers))?;

        if let Some(content) = self.content {
            ticket_tracker.set_content(content);
        }

        if let Some(summary) = self.summary {
            ticket_tracker.set_summary(summary);
        }

        ticket_tracker.find_or_create_tx(&mut dbtx).await?;

        dbtx.commit()
            .await
            .map(|_| ticket_tracker)
            .map_err(Error::from)
    }
}
