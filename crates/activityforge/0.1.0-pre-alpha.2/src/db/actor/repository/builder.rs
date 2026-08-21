use serde::{Deserialize, Serialize};

use crate::db::{
    Db, Follower, Inbox, Iri, Key, Like, Name, Outbox, PatchTracker, Repository, TableEntry,
    TableType, Team, TicketTracker, Uuid,
};
use crate::{Error, Result, util};

/// Represents a builder struct for a [Repository].
///
/// Allows for users to have a high-level interface for creating all of the
/// sub-record entries that make up a [Repository] entry.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub struct RepositoryBuilder {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    inbox: Inbox,
    outbox: Outbox,
    clone_uris: Vec<Iri>,
    push_uris: Vec<Iri>,
    forks: Vec<Repository>,
    likes: Vec<Like>,
    followers: Vec<Follower>,
    keys: Vec<Key>,
    patches_tracked_by: Option<PatchTracker>,
    tickets_tracked_by: Option<TicketTracker>,
    is_archived: Option<bool>,
    moved_to: Option<Repository>,
    mirrors: Option<Repository>,
    team: Option<Team>,
}

impl RepositoryBuilder {
    /// Creates a new [RepositoryBuilder].
    ///
    /// Creates a default inbox and outbox based on the supplied `id`.
    pub fn new<I: Into<Iri>, N: Into<Name>>(id: I, name: N) -> Result<Self> {
        let uuid = util::rand_uuid();

        let id = Repository::TABLE.id_from_uuid(&id.into(), uuid)?;
        let name = name.into();

        let inbox_id = Iri::try_from(format!("{id}/inbox")).unwrap_or_default();
        let outbox_id = Iri::try_from(format!("{id}/outbox")).unwrap_or_default();

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
            clone_uris: Vec::new(),
            push_uris: Vec::new(),
            forks: Vec::new(),
            likes: Vec::new(),
            followers: Vec::new(),
            keys: Vec::new(),
            patches_tracked_by: None,
            tickets_tracked_by: None,
            is_archived: None,
            moved_to: None,
            mirrors: None,
            team: None,
        })
    }

    /// Gets the table type.
    #[inline]
    pub const fn table() -> TableType {
        Repository::TABLE
    }

    /// Gets the table entry.
    #[inline]
    pub const fn table_entry(&self) -> TableEntry {
        TableEntry::create(Self::table(), self.uuid)
    }

    /// Sets the [Uuid] for the [Repository].
    ///
    /// Optional, a random [Uuid] is assigned in [RepositoryBuilder::new].
    pub fn uuid(self, uuid: Uuid) -> Result<Self> {
        Self::table()
            .id_from_uuid(&self.id, uuid)
            .map(|id| {
                Self {
                    uuid,
                    id,
                    name: self.name,
                    clone_uris: self.clone_uris,
                    push_uris: self.push_uris,
                    is_archived: self.is_archived,
                    moved_to: self.moved_to,
                    mirrors: self.mirrors,
                    patches_tracked_by: self.patches_tracked_by,
                    tickets_tracked_by: self.tickets_tracked_by,
                    team: self.team,
                    inbox: Inbox::new(),
                    outbox: Outbox::new(),
                    forks: Vec::new(),
                    likes: Vec::new(),
                    keys: Vec::new(),
                    followers: Vec::new(),
                }
                .inbox(self.inbox)
                .outbox(self.outbox)
                .likes(self.likes)
                .forks(self.forks)
                .followers(self.followers)
            })
            .and_then(|s| s.keys(self.keys))
    }

    /// Sets the ID IRI for the [Repository].
    ///
    /// Optional, set a different ID than provided in [RepositoryBuilder::new].
    pub fn id<I: Into<Iri>>(self, id: I) -> Result<Self> {
        Repository::TABLE
            .id_from_uuid(&id.into(), self.uuid)
            .map(|id| Self { id, ..self })
    }

    /// Sets the name for the [Repository].
    ///
    /// Optional, set a different name than provided in [RepositoryBuilder::new].
    pub fn name<N: Into<Name>>(self, name: N) -> Self {
        Self {
            name: name.into(),
            ..self
        }
    }

    /// Sets the [Inbox] for the [Repository].
    ///
    /// Only needed if the default [Inbox] created by [RepositoryBuilder::new] is incorrect.
    pub fn inbox(self, inbox: Inbox) -> Self {
        let actor = self.table_entry();

        let inbox = if inbox.actor() != actor {
            inbox.with_actor(actor)
        } else {
            inbox
        };

        Self { inbox, ..self }
    }

    /// Sets the [Outbox] for the [Repository].
    ///
    /// Only needed if the default [Outbox] created by [RepositoryBuilder::new] is incorrect.
    pub fn outbox(self, outbox: Outbox) -> Self {
        let actor = self.table_entry();

        let outbox = if outbox.actor() != actor {
            outbox.with_actor(actor)
        } else {
            outbox
        };

        Self { outbox, ..self }
    }

    /// Sets the list of [Repository] clone URIs.
    pub fn clone_uris<T, I>(self, val: I) -> Self
    where
        T: Into<Iri>,
        I: IntoIterator<Item = T>,
    {
        let mut clone_uris: Vec<Iri> = val.into_iter().map(|i| i.into()).collect();
        clone_uris.sort();
        clone_uris.dedup();

        Self { clone_uris, ..self }
    }

    /// Sets the list of [Repository] push URIs.
    pub fn push_uris<T, I>(self, val: I) -> Self
    where
        T: Into<Iri>,
        I: IntoIterator<Item = T>,
    {
        let mut push_uris: Vec<Iri> = val.into_iter().map(|i| i.into()).collect();
        push_uris.sort();
        push_uris.dedup();

        Self { push_uris, ..self }
    }

    /// Sets the list of [Repository] forks.
    pub fn forks<T, I>(self, val: I) -> Self
    where
        T: Into<Repository>,
        I: IntoIterator<Item = T>,
    {
        let mut forks: Vec<Repository> = val.into_iter().map(|i| i.into()).collect();
        forks.sort();
        forks.dedup();

        Self { forks, ..self }
    }

    /// Sets the list of [Repository] likes.
    pub fn likes<T, I>(self, val: I) -> Self
    where
        T: Into<Like>,
        I: IntoIterator<Item = T>,
    {
        let mut likes: Vec<Like> = val.into_iter().map(|i| i.into()).collect();
        likes.sort();
        likes.dedup();

        Self { likes, ..self }
    }

    /// Sets the list of [Repository] [Follower]s.
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

    /// Sets the list of [Repository] [Key]s.
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

    /// Sets the [Repository] patch tracker.
    ///
    /// Optional: if unset, the [Repository] is assumed to be its own [PatchTracker].
    pub fn patches_tracked_by<T>(self, val: T) -> Self
    where
        T: Into<PatchTracker>,
    {
        Self {
            patches_tracked_by: Some(val.into()),
            ..self
        }
    }

    /// Sets the [Repository] ticket tracker.
    ///
    /// Optional: if unset, the [Repository] is assumed to be its own [TicketTracker].
    pub fn tickets_tracked_by<T>(self, val: T) -> Self
    where
        T: Into<TicketTracker>,
    {
        Self {
            tickets_tracked_by: Some(val.into()),
            ..self
        }
    }

    /// Sets where [Repository] moved to, if `is_archived` is true.
    ///
    /// Optional: only set if the [Repository] is archived, sets `is_archived` to true.
    pub fn moved_to<T>(self, val: T) -> Self
    where
        T: Into<Repository>,
    {
        Self {
            moved_to: Some(val.into()),
            is_archived: Some(true),
            ..self
        }
    }

    /// Sets which [Repository] this [Repository] mirrors.
    ///
    /// Optional: only set if this [Repository] is a mirror of another.
    pub fn mirrors<T>(self, val: T) -> Self
    where
        T: Into<Repository>,
    {
        Self {
            mirrors: Some(val.into()),
            ..self
        }
    }

    /// Sets the [Repository] team.
    ///
    /// Optional: only set if this [Repository] has a [Team].
    pub fn team<T>(self, val: T) -> Self
    where
        T: Into<Team>,
    {
        Self {
            team: Some(val.into()),
            ..self
        }
    }

    /// Builds the [Repository] record, and inserts all sub-records into the database.
    pub async fn build(mut self, db: &Db) -> Result<Repository> {
        let uuid = self.uuid;

        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let inbox_uuid = self.inbox.find_or_create_tx(&mut dbtx).await?;
        let outbox_uuid = self.outbox.find_or_create_tx(&mut dbtx).await?;

        for fork in self.forks.iter_mut() {
            fork.find_or_create_tx(&mut dbtx).await?;
        }

        for like in self.likes.iter_mut() {
            like.find_or_create_tx(&mut dbtx).await?;
        }

        for follower in self.followers.iter_mut() {
            follower.find_or_create_tx(&mut dbtx).await?;
        }

        for key in self.keys.iter_mut() {
            key.find_or_create_tx(&mut dbtx, &db_key).await?;
        }

        let patches_tracked_by = if let Some(mut p) = self.patches_tracked_by {
            p.find_or_create_tx(&mut dbtx)
                .await
                .map(|_| Some(p.id().clone()))?
        } else {
            None
        };

        let tickets_tracked_by = if let Some(mut p) = self.tickets_tracked_by {
            p.find_or_create_tx(&mut dbtx)
                .await
                .map(|_| Some(p.id().clone()))?
        } else {
            None
        };

        let moved_to = if let Some(mut m) = self.moved_to {
            m.find_or_create_tx(&mut dbtx)
                .await
                .map(|_| Some(m.id().clone()))?
        } else {
            None
        };

        let mirrors = if let Some(mut m) = self.mirrors {
            m.find_or_create_tx(&mut dbtx)
                .await
                .map(|_| Some(m.id().clone()))?
        } else {
            None
        };

        let team = if let Some(mut t) = self.team {
            t.find_or_create_tx(&mut dbtx)
                .await
                .map(|_| Some(t.id().clone()))?
        } else {
            None
        };

        let forks = self
            .forks
            .into_iter()
            .map(|f| f.id().clone())
            .collect::<Vec<_>>();
        let likes = self.likes.into_iter().map(|l| l.uuid()).collect::<Vec<_>>();
        let followers = self
            .followers
            .into_iter()
            .map(|f| f.id().clone())
            .collect::<Vec<_>>();
        let key_ids = self.keys.into_iter().map(|k| k.uuid()).collect::<Vec<_>>();

        let mut repository = Repository::new()
            .with_uuid(uuid)
            .with_id(self.id)
            .with_name(self.name)
            .with_inbox(inbox_uuid)
            .with_outbox(outbox_uuid)
            .with_clone_uris(self.clone_uris)
            .and_then(|p| p.with_push_uris(self.push_uris))
            .and_then(|p| p.with_forks(forks))
            .and_then(|p| p.with_likes(likes))
            .and_then(|p| p.with_followers(followers))
            .and_then(|p| p.with_key_ids(key_ids))?;

        if let Some(p) = patches_tracked_by {
            repository.set_send_patches_to(p);
        }

        if let Some(t) = tickets_tracked_by {
            repository.set_tickets_tracked_by(t);
        }

        if let Some(m) = self.is_archived {
            repository.set_is_archived(m);
        }

        if let Some(m) = moved_to {
            repository.set_moved_to(m);
        }

        if let Some(m) = mirrors {
            repository.set_mirrors(m);
        }

        if let Some(t) = team {
            repository.set_team(t);
        }

        repository.find_or_create_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| repository).map_err(Error::from)
    }
}
