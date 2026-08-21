use activitystreams_vocabulary::DateTime;
use chrono::{Timelike, Utc};
use serde::{Deserialize, Serialize};

use crate::db::{
    Collaborator, Db, Inbox, Iri, Key, Name, Outbox, RoleFilter, TableEntry, TableType, Team, Uuid,
};
use crate::{Error, Result, util};

/// Represents a builder struct for a [Team].
///
/// Allows for users to have a high-level interface for creating all of the
/// sub-record entries that make up a [Team] entry.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TeamBuilder {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    inbox: Inbox,
    outbox: Outbox,
    published: DateTime,
    summary: Option<String>,
    content: Option<String>,
    context: Option<Team>,
    members: Vec<Collaborator>,
    subteams: Vec<Team>,
    oversees: Vec<Team>,
    overseen_bys: Vec<Team>,
    role_filters: Vec<RoleFilter>,
    keys: Vec<Key>,
}

impl TeamBuilder {
    /// Creates a new [TeamBuilder].
    ///
    /// Creates a default inbox and outbox based on the supplied `id`.
    pub fn new<I: Into<Iri>, N: Into<Name>>(id: I, name: N) -> Result<Self> {
        let uuid = util::rand_uuid();

        let id = Team::TABLE.id_from_uuid(&id.into(), uuid)?;
        let name = name.into();

        let inbox_id = Iri::try_from(format!("{id}/inbox"))?;
        let outbox_id = Iri::try_from(format!("{id}/outbox"))?;

        let actor = TableEntry::create(Self::table(), uuid);

        let inbox = Inbox::new()
            .with_uuid(uuid)
            .with_id(inbox_id)
            .with_actor(actor);

        let outbox = Outbox::new()
            .with_uuid(util::rand_uuid())
            .with_id(outbox_id)
            .with_actor(actor);

        let published = Utc::now()
            .with_nanosecond(0)
            .map(|p| p.into())
            .ok_or(Error::db("team: invalid published date"))?;

        Ok(Self {
            uuid,
            id,
            name,
            inbox,
            outbox,
            published,
            summary: None,
            content: None,
            context: None,
            members: Vec::new(),
            subteams: Vec::new(),
            oversees: Vec::new(),
            overseen_bys: Vec::new(),
            role_filters: Vec::new(),
            keys: Vec::new(),
        })
    }

    /// Gets the table type.
    #[inline]
    pub const fn table() -> TableType {
        Team::TABLE
    }

    /// Gets the table entry.
    #[inline]
    pub const fn table_entry(&self) -> TableEntry {
        TableEntry::create(Self::table(), self.uuid)
    }

    /// Sets the [Uuid] for the [Team].
    ///
    /// Optional, a random [Uuid] is assigned in [TeamBuilder::new].
    pub fn uuid(self, uuid: Uuid) -> Result<Self> {
        Self::table()
            .id_from_uuid(&self.id, uuid)
            .map(|id| {
                Self {
                    uuid,
                    id,
                    name: self.name,
                    published: self.published,
                    summary: self.summary,
                    content: self.content,
                    context: self.context,
                    inbox: Inbox::new(),
                    outbox: Outbox::new(),
                    members: Vec::new(),
                    subteams: Vec::new(),
                    oversees: Vec::new(),
                    overseen_bys: Vec::new(),
                    role_filters: Vec::new(),
                    keys: Vec::new(),
                }
                .inbox(self.inbox)
                .outbox(self.outbox)
                .members(self.members)
                .subteams(self.subteams)
                .oversees(self.oversees)
                .overseen_bys(self.overseen_bys)
                .role_filters(self.role_filters)
            })
            .and_then(|t| t.keys(self.keys))
    }

    /// Sets the ID IRI for the [Team].
    ///
    /// Optional, set a different ID than provided in [TeamBuilder::new].
    pub fn id<I: Into<Iri>>(self, id: I) -> Result<Self> {
        Self::table()
            .id_from_uuid(&id.into(), self.uuid)
            .map(|id| Self { id, ..self })
    }

    /// Sets the name for the [Team].
    ///
    /// Optional, set a different name than provided in [TeamBuilder::new].
    pub fn name<N: Into<Name>>(self, name: N) -> Self {
        Self {
            name: name.into(),
            ..self
        }
    }

    /// Sets the content for the [Team].
    pub fn content<I: Into<String>>(self, content: I) -> Self {
        Self {
            content: Some(content.into()),
            ..self
        }
    }

    /// Sets the summary for the [Team].
    pub fn summary<I: Into<String>>(self, summary: I) -> Self {
        Self {
            summary: Some(summary.into()),
            ..self
        }
    }

    /// Sets the context for the [Team].
    ///
    /// Optional: this represents the parent [Team].
    pub fn context<I: Into<Team>>(self, val: I) -> Self {
        Self {
            context: Some(val.into()),
            ..self
        }
    }

    /// Sets the published date for the [Team].
    pub fn published<I: Into<DateTime>>(self, val: I) -> Self {
        Self {
            published: val.into(),
            ..self
        }
    }

    /// Sets the [Inbox] for the [Team].
    ///
    /// Only needed if the default [Inbox] created by [TeamBuilder::new] is incorrect.
    pub fn inbox(self, inbox: Inbox) -> Self {
        let actor = self.table_entry();

        let inbox = if inbox.actor() != actor {
            inbox.with_actor(actor)
        } else {
            inbox
        };

        Self { inbox, ..self }
    }

    /// Sets the [Outbox] for the [Team].
    ///
    /// Only needed if the default [Outbox] created by [TeamBuilder::new] is incorrect.
    pub fn outbox(self, outbox: Outbox) -> Self {
        let actor = self.table_entry();

        let outbox = if outbox.actor() != actor {
            outbox.with_actor(actor)
        } else {
            outbox
        };

        Self { outbox, ..self }
    }

    /// Sets the list of [Team] [Key]s.
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

    /// Sets the list of [Team] [members](Collaborator).
    pub fn members<T, I>(self, val: I) -> Self
    where
        T: Into<Collaborator>,
        I: IntoIterator<Item = T>,
    {
        let mut members: Vec<Collaborator> = val
            .into_iter()
            .map(|i| {
                let member = i.into();

                if member.subject() == &self.id {
                    member
                } else {
                    member.with_subject(&self.id)
                }
            })
            .collect();

        members.sort();
        members.dedup();

        Self { members, ..self }
    }

    /// Sets the list of [Team] [subteams](Team).
    pub fn subteams<T, I>(self, val: I) -> Self
    where
        T: Into<Team>,
        I: IntoIterator<Item = T>,
    {
        let mut subteams: Vec<Team> = val
            .into_iter()
            .map(|i| {
                let subteam = i.into();

                if subteam.context() == Some(self.uuid) {
                    subteam
                } else {
                    subteam.with_context(self.uuid)
                }
            })
            .collect();

        subteams.sort();
        subteams.dedup();

        Self { subteams, ..self }
    }

    /// Sets the list of [Team] [oversees](Team) teams.
    ///
    /// Optional: this is the list of [Team]s overseen by this [Team].
    pub fn oversees<T, I>(self, val: I) -> Self
    where
        T: Into<Team>,
        I: IntoIterator<Item = T>,
    {
        let mut oversees: Vec<Team> = val
            .into_iter()
            .map(|i| {
                let mut oversee = i.into();

                if oversee.overseen_bys().contains(&self.uuid) {
                    oversee
                } else {
                    oversee.overseen_bys.push(self.uuid);
                    oversee
                }
            })
            .collect();

        oversees.sort();
        oversees.dedup();

        Self { oversees, ..self }
    }

    /// Sets the list of [Team] [overseen_by](Team) teams.
    ///
    /// Optional: this is the list of [Team]s overseeing this [Team].
    pub fn overseen_bys<T, I>(self, val: I) -> Self
    where
        T: Into<Team>,
        I: IntoIterator<Item = T>,
    {
        let mut overseen_bys: Vec<Team> = val
            .into_iter()
            .map(|i| {
                let mut oversee = i.into();

                if oversee.oversees().contains(&self.uuid) {
                    oversee
                } else {
                    oversee.oversees.push(self.uuid);
                    oversee
                }
            })
            .collect();

        overseen_bys.sort();
        overseen_bys.dedup();

        Self {
            overseen_bys,
            ..self
        }
    }

    /// Sets the list of [Team] [role_filters](Team).
    pub fn role_filters<T, I>(self, val: I) -> Self
    where
        T: Into<RoleFilter>,
        I: IntoIterator<Item = T>,
    {
        let mut role_filters: Vec<RoleFilter> = val.into_iter().map(|i| i.into()).collect();

        role_filters.sort();
        role_filters.dedup();

        Self {
            role_filters,
            ..self
        }
    }

    /// Builds the [Team] record, and inserts all sub-records into the database.
    pub async fn build(mut self, db: &Db) -> Result<Team> {
        let uuid = self.uuid;

        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let inbox_uuid = self.inbox.find_or_create_tx(&mut dbtx).await?;
        let outbox_uuid = self.outbox.find_or_create_tx(&mut dbtx).await?;

        for key in self.keys.iter_mut() {
            key.find_or_create_tx(&mut dbtx, &db_key).await?;
        }

        for member in self.members.iter_mut() {
            member.find_or_create_tx(&mut dbtx).await?;
        }

        for subteam in self.subteams.iter_mut() {
            subteam.find_or_create_tx(&mut dbtx).await?;
        }

        for oversee in self.oversees.iter_mut() {
            oversee.find_or_create_tx(&mut dbtx).await?;
        }

        for oversee in self.overseen_bys.iter_mut() {
            oversee.find_or_create_tx(&mut dbtx).await?;
        }

        let key_ids = self.keys.into_iter().map(|k| k.uuid()).collect::<Vec<_>>();

        let members = self
            .members
            .into_iter()
            .map(|m| m.uuid())
            .collect::<Vec<_>>();

        let subteams = self
            .subteams
            .into_iter()
            .map(|s| s.uuid())
            .collect::<Vec<_>>();

        let oversees = self
            .oversees
            .into_iter()
            .map(|o| o.uuid())
            .collect::<Vec<_>>();

        let overseen_bys = self
            .overseen_bys
            .into_iter()
            .map(|o| o.uuid())
            .collect::<Vec<_>>();

        let mut team = Team::new()
            .with_uuid(uuid)
            .with_id(self.id)
            .with_name(self.name)
            .with_inbox(inbox_uuid)
            .with_outbox(outbox_uuid)
            .with_published(self.published)
            .with_key_ids(key_ids)
            .and_then(|p| p.with_members(members))
            .and_then(|p| p.with_subteams(subteams))
            .and_then(|p| p.with_oversees(oversees))
            .and_then(|p| p.with_overseen_bys(overseen_bys))
            .and_then(|p| p.with_role_filters(self.role_filters))?;

        if let Some(content) = self.content {
            team.set_content(content);
        }

        if let Some(summary) = self.summary {
            team.set_summary(summary);
        }

        team.find_or_create_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| team).map_err(Error::from)
    }
}
