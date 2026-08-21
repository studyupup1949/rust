use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};

use crate::db::{Db, Iri, Name, Uuid};
use crate::{Error, Result, impl_sql_list_field, impl_sql_record, util};

/// Represents a [Repository Actor](https://forgefed.org/spec/#repository) database record.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    inbox: Uuid,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    outbox: Uuid,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    clone_uris: Vec<Iri>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    push_uris: Vec<Iri>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    forks: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    likes: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    followers: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    key_ids: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "util::ser_uuid_opt",
        deserialize_with = "util::de_uuid_opt"
    )]
    patches_tracked_by: Option<Uuid>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "util::ser_uuid_opt",
        deserialize_with = "util::de_uuid_opt"
    )]
    tickets_tracked_by: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_archived: Option<bool>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "util::ser_uuid_opt",
        deserialize_with = "util::de_uuid_opt"
    )]
    moved_to: Option<Uuid>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "util::ser_uuid_opt",
        deserialize_with = "util::de_uuid_opt"
    )]
    mirrors: Option<Uuid>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "util::ser_uuid_opt",
        deserialize_with = "util::de_uuid_opt"
    )]
    team: Option<Uuid>,
}

impl Repository {
    /// Creates a new [Repository].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            name: Name::new(),
            inbox: Uuid::nil(),
            outbox: Uuid::nil(),
            clone_uris: Vec::new(),
            push_uris: Vec::new(),
            forks: Vec::new(),
            likes: Vec::new(),
            followers: Vec::new(),
            key_ids: Vec::new(),
            patches_tracked_by: None,
            tickets_tracked_by: None,
            is_archived: None,
            moved_to: None,
            mirrors: None,
            team: None,
        }
    }

    /// Attempts to get a [Repository] record by [Uuid].
    pub async fn get(db: &Db, uuid: &Uuid) -> Result<Self> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let factory = Self::get_tx(&mut dbtx, uuid).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| factory)
    }

    /// Attempts to get a [Repository] record by [Uuid] using a DB transaction.
    pub async fn get_tx(dbtx: &mut Transaction<'_, Postgres>, uuid: &Uuid) -> Result<Self> {
        sqlx::query("SELECT * FROM repository WHERE uuid = $1")
            .bind(uuid)
            .fetch_one(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|row| {
                Ok(Self {
                    uuid: *uuid,
                    id: row.try_get::<Iri, &str>("id")?,
                    name: row.try_get::<Name, &str>("name")?,
                    inbox: row.try_get::<Uuid, &str>("inbox")?,
                    outbox: row.try_get::<Uuid, &str>("outbox")?,
                    clone_uris: row.try_get::<Vec<Iri>, &str>("clone_uris")?,
                    push_uris: row.try_get::<Vec<Iri>, &str>("push_uris")?,
                    forks: row.try_get::<Vec<Uuid>, &str>("forks")?,
                    likes: row.try_get::<Vec<Uuid>, &str>("likes")?,
                    followers: row.try_get::<Vec<Uuid>, &str>("followers")?,
                    key_ids: row.try_get::<Vec<Uuid>, &str>("key_ids")?,
                    patches_tracked_by: row.try_get::<Option<Uuid>, &str>("patches_tracked_by")?,
                    tickets_tracked_by: row.try_get::<Option<Uuid>, &str>("tickets_tracked_by")?,
                    is_archived: row.try_get::<Option<bool>, &str>("is_archived")?,
                    moved_to: row.try_get::<Option<Uuid>, &str>("moved_to")?,
                    mirrors: row.try_get::<Option<Uuid>, &str>("mirrors")?,
                    team: row.try_get::<Option<Uuid>, &str>("team")?,
                })
            })
    }

    /// Attempts to insert a [Repository] record into the database.
    pub async fn insert(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let uuid = self.insert_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| uuid).map_err(Error::from)
    }

    /// Attempts to insert a [Repository] record into the database using a [Transaction].
    pub async fn insert_tx(&mut self, dbtx: &mut Transaction<'_, Postgres>) -> Result<Uuid> {
        if self.inbox.is_nil() {
            return Err(Error::sql("nil Repository inbox"));
        }

        if self.outbox.is_nil() {
            return Err(Error::sql("nil Repository outbox"));
        }

        let row = if self.uuid.is_nil() {
            sqlx::query(
                "INSERT INTO repository
                (id, name, inbox, outbox, clone_uris, push_uris, forks, likes, patches_tracked_by, tickets_tracked_by, followers, key_ids, is_archived, moved_to, mirrors, team)
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16)
                RETURNING uuid",
            )
            .bind(self.id())
            .bind(self.name())
            .bind(self.inbox)
            .bind(self.outbox)
            .bind(self.clone_uris.as_slice())
            .bind(self.push_uris.as_slice())
            .bind(self.forks.as_slice())
            .bind(self.likes.as_slice())
            .bind(self.patches_tracked_by)
            .bind(self.tickets_tracked_by)
            .bind(self.followers.as_slice())
            .bind(self.key_ids.as_slice())
            .bind(self.is_archived)
            .bind(self.moved_to)
            .bind(self.mirrors)
            .bind(self.team)
            .fetch_one(&mut **dbtx)
            .await?
        } else {
            sqlx::query(
                "INSERT INTO repository
                (uuid, id, name, inbox, outbox, clone_uris, push_uris, forks, likes, patches_tracked_by, tickets_tracked_by, followers, key_ids, is_archived, moved_to, mirrors, team)
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
                RETURNING uuid",
            )
            .bind(self.uuid)
            .bind(self.id())
            .bind(self.name())
            .bind(self.inbox)
            .bind(self.outbox)
            .bind(self.clone_uris.as_slice())
            .bind(self.push_uris.as_slice())
            .bind(self.forks.as_slice())
            .bind(self.likes.as_slice())
            .bind(self.patches_tracked_by)
            .bind(self.tickets_tracked_by)
            .bind(self.followers.as_slice())
            .bind(self.key_ids.as_slice())
            .bind(self.is_archived)
            .bind(self.moved_to)
            .bind(self.mirrors)
            .bind(self.team)
            .fetch_one(&mut **dbtx)
            .await?
        };

        let uuid = row.try_get::<Uuid, &str>("uuid")?;

        if self.uuid.is_nil() {
            self.uuid = uuid;
        }

        Ok(uuid)
    }

    /// Attempts to update a [Repository] record into the database.
    pub async fn update(&self, db: &Db) -> Result<()> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        self.update_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| ()).map_err(Error::from)
    }

    /// Attempts to update a [Repository] record into the database using a [Transaction].
    pub async fn update_tx(&self, dbtx: &mut Transaction<'_, Postgres>) -> Result<()> {
        util::check_uuid("repository", &self.uuid)?;

        if self.inbox.is_nil() {
            return Err(Error::sql("nil Repository inbox"));
        }

        if self.outbox.is_nil() {
            return Err(Error::sql("nil Repository outbox"));
        }

        sqlx::query(
            "UPDATE repository
            SET
            (id, name, inbox, outbox, clone_uris, push_uris, forks, likes, patches_tracked_by, tickets_tracked_by, followers, key_ids, is_archived, moved_to, mirrors, team)
            =
            ($2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17)
            WHERE uuid = $1",
        )
        .bind(self.uuid)
        .bind(self.id())
        .bind(self.name())
        .bind(self.inbox)
        .bind(self.outbox)
        .bind(self.clone_uris.as_slice())
        .bind(self.push_uris.as_slice())
        .bind(self.forks.as_slice())
        .bind(self.likes.as_slice())
        .bind(self.patches_tracked_by)
        .bind(self.tickets_tracked_by)
        .bind(self.followers.as_slice())
        .bind(self.key_ids.as_slice())
        .bind(self.is_archived)
        .bind(self.moved_to)
        .bind(self.mirrors)
        .bind(self.team)
        .execute(&mut **dbtx)
        .await
        .map(|_| ())
        .map_err(|err| Error::sql(format!("repository: error updating record: {err}")))
    }
}

field_access! {
    Repository {
        /// Main [Uuid] for the [Repository] record.
        uuid: Uuid,
        /// Represents the [Uuid] of the [Inbox](crate::db::Inbox) record.
        inbox: Uuid,
        /// Represents the [Uuid] of the [Outbox](crate::db::Outbox) record.
        outbox: Uuid,
    }
}

field_access! {
    Repository {
        /// ActivityPub [Iri] for external actors to reference the [Repository].
        id: as_ref { Iri },
        /// Represents the human-readable name of the [Repository].
        name: as_ref { Name },
    }
}

field_access! {
    Repository {
        /// References the actor record for the [Repository] [PatchTracker](crate::db::PatchTracker).
        ///
        /// If nil, the [Repository] tracks its own patches.
        patches_tracked_by: option { Uuid },
        /// References the actor record for the [Repository] [TicketTracker](crate::db::TicketTracker).
        ///
        /// If nil, the [Repository] tracks its own patches.
        tickets_tracked_by: option { Uuid },
        /// Represents whether the [Repository] is archived.
        is_archived: option { bool },
        /// Represents the new [Repository] record for a [Repository] that is archived.
        moved_to: option { Uuid },
        /// References the [Repository] record mirrored by this [Repostory].
        ///
        /// The referenced [Repository] can be a minimal record with just the `id` field filled out.
        mirrors: option { Uuid },
        /// References to the [Team](crate::Team) record.
        team: option { Uuid },
    }
}

impl_sql_list_field! {
    Repository {
        /// Represents a list URIs to clone the [Repository].
        clone_uri: "clone_uris" Iri,
        /// Represents a list URIs to push to the [Repository].
        push_uri: "push_uris" Iri,
        /// References to other [Repository] records that are forked from this repository.
        ///
        /// List of [Uuid]s that should reference the [Repository] records for forks of this [Repository].
        ///
        /// Can be minimal records with just the `id` field filled out.
        fork: "forks" Uuid,
        /// Represents a list of references to the [Repository]'s [Follower](crate::db::Follower) records.
        follower: "followers" Uuid,
        /// Represents a list of references to the [Repository]'s [Key](crate::db::Key) records.
        ///
        /// For local records:
        ///
        /// - there should be at least one private [Key](crate::db::Key) used by the server to sign requests
        /// - there can be any number of public [Key](crate::db::Key) records
        ///   - the private key is stored offline (requires the client to sign requests)
        ///
        /// For remote records:
        ///
        /// - [Key](crate::db::Key) records should be a public keys
        key_id: "key_ids" Uuid,
        /// Represents a list of references to the [Repository]'s [Like](crate::db::Like) records.
        like: "likes" Uuid,
    }
}

impl_default!(Repository);
impl_display!(Repository, json);
impl_sql_record!(Repository);
