use std::marker::PhantomData;

use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};

use crate::db::{Db, Iri, TableEntry, Uuid};
use crate::{Error, Result, impl_sql_list_field, impl_sql_record, util};

/// Represents the mailbox variants.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MailboxType {
    Inbox,
    Outbox,
}

impl MailboxType {
    pub const INBOX: &str = "inbox";
    pub const OUTBOX: &str = "outbox";

    /// Creates a new [MailboxType]
    #[inline]
    pub const fn new() -> Self {
        Self::Inbox
    }

    /// Gets the [MailboxType] string representation.
    #[inline]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Inbox => Self::INBOX,
            Self::Outbox => Self::OUTBOX,
        }
    }
}

impl_default!(MailboxType);
impl_display!(MailboxType, str);

/// Marker trait for mailbox types.
///
/// Helpful for deduplication of database code.
pub trait MailboxDir {
    fn mailbox() -> MailboxType;
}

/// Marker type for an ActivityPub inbox.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InboxType;

/// Marker type for an ActivityPub outbox.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutboxType;

impl MailboxDir for InboxType {
    fn mailbox() -> MailboxType {
        MailboxType::Inbox
    }
}

impl MailboxDir for OutboxType {
    fn mailbox() -> MailboxType {
        MailboxType::Outbox
    }
}

/// Represents an ActivityPub mailbox
///
/// - [inbox](https://www.w3.org/TR/activitypub/#inbox)
/// - [outbox](https://www.w3.org/TR/activitypub/#inbox)
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Mailbox<Dir: MailboxDir> {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    actor: TableEntry,
    activities: Vec<TableEntry>,
    _phantom: PhantomData<Dir>,
}

impl<Dir: MailboxDir> Mailbox<Dir> {
    /// Creates a new [Mailbox].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            actor: TableEntry::new(),
            activities: Vec::new(),
            _phantom: PhantomData,
        }
    }

    /// Gets the [Uuid].
    ///
    /// Represents the [Uuid] primary key of the table entry.
    pub const fn uuid(&self) -> Uuid {
        self.uuid
    }

    /// Sets the [Uuid].
    ///
    /// Represents the [Uuid] primary key of the table entry.
    pub fn set_uuid<I: Into<Uuid>>(&mut self, uuid: I) {
        self.uuid = uuid.into();
    }

    /// Builder function that sets the [Uuid].
    ///
    /// Represents the [Uuid] primary key of the table entry.
    pub fn with_uuid<I: Into<Uuid>>(self, uuid: I) -> Self {
        Self {
            uuid: uuid.into(),
            ..self
        }
    }

    /// Gets the `id`.
    ///
    /// Represents the IRI used to fetch the [Mailbox] record.
    pub const fn id(&self) -> &Iri {
        &self.id
    }

    /// Sets the `id`.
    ///
    /// Represents the IRI used to fetch the [Mailbox] record.
    pub fn set_id<I: Into<Iri>>(&mut self, id: I) {
        self.id = id.into();
    }

    /// Builder function that sets the `id`.
    ///
    /// Represents the IRI used to fetch the [Mailbox] record.
    pub fn with_id<I: Into<Iri>>(self, id: I) -> Self {
        Self {
            id: id.into(),
            ..self
        }
    }

    /// Gets the `actor`.
    ///
    /// References the actor receiving activities in this [Mailbox].
    pub const fn actor(&self) -> TableEntry {
        self.actor
    }

    /// Sets the `actor`.
    ///
    /// References the actor receiving activities in this [Mailbox].
    pub fn set_actor<I: Into<TableEntry>>(&mut self, actor: I) {
        self.actor = actor.into();
    }

    /// Builder function that sets the `actor`.
    ///
    /// References the actor receiving activities in this [Mailbox].
    pub fn with_actor<I: Into<TableEntry>>(self, actor: I) -> Self {
        Self {
            actor: actor.into(),
            ..self
        }
    }

    /// Attempts to get a [Mailbox] record by [Uuid].
    pub async fn get(db: &Db, uuid: &Uuid) -> Result<Self> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let factory = Self::get_tx(&mut dbtx, uuid).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| factory)
    }

    /// Attempts to get a [Mailbox] record by [Uuid] using a DB transaction.
    pub async fn get_tx(dbtx: &mut Transaction<'_, Postgres>, uuid: &Uuid) -> Result<Self> {
        let mailbox = Dir::mailbox();
        sqlx::query(format!("SELECT * FROM {mailbox} WHERE uuid = $1").as_str())
            .bind(uuid)
            .fetch_one(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|row| {
                Ok(Self {
                    uuid: *uuid,
                    id: row.try_get::<Iri, &str>("id")?,
                    actor: row.try_get::<TableEntry, &str>("actor")?,
                    activities: row.try_get::<Vec<TableEntry>, &str>("activities")?,
                    _phantom: PhantomData,
                })
            })
    }

    /// Attempts to insert a [Mailbox] record into the database.
    pub async fn insert(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let uuid = self.insert_tx(&mut dbtx).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| uuid)
    }

    /// Attempts to insert a [Mailbox] record into the database using a [Transaction].
    pub async fn insert_tx(&mut self, dbtx: &mut Transaction<'_, Postgres>) -> Result<Uuid> {
        let mailbox = Dir::mailbox();

        if self.actor.is_empty() {
            return Err(Error::sql(format!("{mailbox}: empty actor")));
        }

        if self.id.is_empty() {
            return Err(Error::sql(format!("{mailbox}: empty ID")));
        }

        let row = if self.uuid.is_nil() {
            sqlx::query(
                format!(
                    "INSERT INTO {mailbox}
                    (id, actor, activities)
                    VALUES
                    ($1, $2, $3)
                    RETURNING uuid"
                )
                .as_str(),
            )
            .bind(self.id())
            .bind(self.actor())
            .bind(self.activities.as_slice())
            .fetch_one(&mut **dbtx)
            .await?
        } else {
            sqlx::query(
                format!(
                    "INSERT INTO {mailbox}
                    (uuid, id, actor, activities)
                    VALUES
                    ($1, $2, $3, $4)
                    RETURNING uuid"
                )
                .as_str(),
            )
            .bind(self.uuid)
            .bind(self.id())
            .bind(self.actor())
            .bind(self.activities.as_slice())
            .fetch_one(&mut **dbtx)
            .await?
        };

        let uuid = row.try_get::<Uuid, &str>("uuid")?;

        if self.uuid.is_nil() {
            self.uuid = uuid;
        }

        Ok(uuid)
    }

    /// Attempts to insert a reference to an activity into the [Mailbox].
    ///
    /// Fully sets the activities, replacing the current list.
    ///
    /// Can be used to clear the activities by setting `activities` to en empty list.
    pub async fn update_activities<T, I>(&mut self, db: &Db, activities: I) -> Result<()>
    where
        T: Into<TableEntry>,
        I: IntoIterator<Item = T>,
    {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        self.update_activities_tx(&mut dbtx, activities).await?;

        dbtx.commit().await.map(|_| ()).map_err(Error::from)
    }

    /// Attempts to update the list of [Mailbox] activities using a [Transaction].
    ///
    /// Fully sets the activities, replacing the current list.
    ///
    /// Can be used to clear the activities by setting `activities` to en empty list.
    pub async fn update_activities_tx<T, I>(
        &mut self,
        dbtx: &mut Transaction<'_, Postgres>,
        activities: I,
    ) -> Result<()>
    where
        T: Into<TableEntry>,
        I: IntoIterator<Item = T>,
    {
        let mailbox = Dir::mailbox();
        util::check_uuid(mailbox.as_str(), &self.uuid)?;

        let activities = activities.into_iter().map(|i| i.into()).collect::<Vec<_>>();

        for activity in activities.iter() {
            activity.table().check_activity()?;
        }

        self.activities = util::dedup_list(mailbox.as_str(), activities)?;

        sqlx::query(format!("UPDATE {mailbox} SET activities = $2 WHERE uuid = $1").as_str())
            .bind(self.uuid)
            .bind(self.activities.as_slice())
            .execute(&mut **dbtx)
            .await
            .map(|_| ())
            .map_err(|err| Error::sql(format!("{mailbox}: error updating entry: {err}")))
    }
}

impl<Dir: MailboxDir> Default for Mailbox<Dir> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Dir: MailboxDir> core::fmt::Display for Mailbox<Dir> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        serde_json::to_string(self)
            .map_err(|_| core::fmt::Error)
            .and_then(|s| write!(f, "{s}"))
    }
}

/// Convenience alias for an ActivityPub [Inbox](https://www.w3.org/TR/activitypub/#inbox).
pub type Inbox = Mailbox<InboxType>;

/// Convenience alias for an ActivityPub [Outbox](https://www.w3.org/TR/activitypub/#outbox).
pub type Outbox = Mailbox<OutboxType>;

impl_sql_record!(Inbox);
impl_sql_record!(Outbox);

impl_sql_list_field! {
    Inbox {
        /// Represents the list of mailbox activities.
        activity, activities: "activitiies" TableEntry,
    }
}

impl_sql_list_field! {
    Outbox {
        /// Represents the list of mailbox activities.
        activity, activities: "activitiies" TableEntry,
    }
}
