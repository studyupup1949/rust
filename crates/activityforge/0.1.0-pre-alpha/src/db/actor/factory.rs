use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};

use crate::db::{ActorType, Db, Iri, Name, Uuid};
use crate::{Error, Result, util};

/// Represents a [Factory Actor](https://forgefed.org/spec/#factory) database record.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Factory {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    available_actor_types: Vec<ActorType>,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    inbox: Uuid,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    outbox: Uuid,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    key_ids: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    collaborators: Vec<Uuid>,
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
    teams: Vec<Uuid>,
}

impl Factory {
    /// Creates a new [Factory].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            name: Name::new(),
            available_actor_types: Vec::new(),
            inbox: Uuid::nil(),
            outbox: Uuid::nil(),
            key_ids: Vec::new(),
            collaborators: Vec::new(),
            followers: Vec::new(),
            teams: Vec::new(),
        }
    }

    /// Attempts to get a [Factory] record by [Uuid].
    pub async fn get(db: &Db, uuid: &Uuid) -> Result<Self> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let factory = Self::get_tx(&mut dbtx, uuid).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| factory)
    }

    /// Attempts to get a [Factory] record by [Uuid] using a DB transaction.
    pub async fn get_tx(dbtx: &mut Transaction<'_, Postgres>, uuid: &Uuid) -> Result<Self> {
        sqlx::query("SELECT * FROM factory WHERE uuid = $1")
            .bind(uuid)
            .fetch_one(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|row| {
                Ok(Self {
                    uuid: *uuid,
                    id: row.try_get::<Iri, &str>("id")?,
                    name: row.try_get::<Name, &str>("name")?,
                    available_actor_types: row
                        .try_get::<Vec<ActorType>, &str>("available_actor_types")?,
                    inbox: row.try_get::<Uuid, &str>("inbox")?,
                    outbox: row.try_get::<Uuid, &str>("outbox")?,
                    key_ids: row.try_get::<Vec<Uuid>, &str>("key_ids")?,
                    collaborators: row.try_get::<Vec<Uuid>, &str>("collaborators")?,
                    followers: row.try_get::<Vec<Uuid>, &str>("followers")?,
                    teams: row.try_get::<Vec<Uuid>, &str>("teams")?,
                })
            })
    }

    /// Attempts to insert a [Factory] record into the database.
    pub async fn insert(&self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let uuid = self.insert_tx(&mut dbtx).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| uuid)
    }

    /// Attempts to insert a [Factory] record into the database using a [Transaction]..
    pub async fn insert_tx(&self, dbtx: &mut Transaction<'_, Postgres>) -> Result<Uuid> {
        if self.inbox.is_nil() {
            return Err(Error::sql("factory: nil inbox"));
        }

        if self.outbox.is_nil() {
            return Err(Error::sql("factory: nil outbox"));
        }

        let row = if self.uuid.is_nil() {
            sqlx::query(
                "INSERT INTO factory
                (id, name, available_actor_types, inbox, outbox, key_ids, collaborators, followers, teams)
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING uuid",
            )
            .bind(self.id())
            .bind(self.name())
            .bind(self.available_actor_types.as_slice())
            .bind(self.inbox)
            .bind(self.outbox)
            .bind(self.key_ids.as_slice())
            .bind(self.collaborators.as_slice())
            .bind(self.followers.as_slice())
            .bind(self.teams.as_slice())
            .fetch_one(&mut **dbtx)
            .await?
        } else {
            sqlx::query(
                "INSERT INTO factory
                (uuid, id, name, available_actor_types, inbox, outbox, key_ids, collaborators, followers, teams)
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
                RETURNING uuid",
            )
            .bind(self.uuid)
            .bind(self.id())
            .bind(self.name())
            .bind(self.available_actor_types.as_slice())
            .bind(self.inbox)
            .bind(self.outbox)
            .bind(self.key_ids.as_slice())
            .bind(self.collaborators.as_slice())
            .bind(self.followers.as_slice())
            .bind(self.teams.as_slice())
            .fetch_one(&mut **dbtx)
            .await?
        };

        row.try_get::<Uuid, &str>("uuid").map_err(Error::from)
    }
}

field_access! {
    Factory {
        /// Main [Uuid] primary key for the record.
        uuid: Uuid,
        /// Represents the [Uuid] of the [Inbox](crate::db::Inbox) record.
        inbox: Uuid,
        /// Represents the [Uuid] of the [Outbox](crate::db::Outbox) record.
        outbox: Uuid,
    }
}

field_access! {
    Factory {
        /// ActivityPub [Iri] for external actors to reference the [Factory].
        id: as_ref { Iri },
        /// Represents the human-readable name of the [Factory].
        name: as_ref { Name },
    }
}

field_access! {
    Factory {
        /// Represents the list of available [ActorType]s that the [Factory] can create.
        available_actor_types: as_ref { &[ActorType], Vec<ActorType> },
        /// Represents a list of references to the [Factory]'s [Key](crate::db::Key) records.
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
        key_ids: as_ref { &[Uuid], Vec<Uuid> },
        /// List of references to the [Factory]'s [Collaborator](crate::db::Collaborator) records.
        collaborators: as_ref { &[Uuid], Vec<Uuid> },
        /// List of references to the [Factory]'s [Follower](crate::db::Follower) records.
        followers: as_ref { &[Uuid], Vec<Uuid> },
        /// List of references to the [Factory]'s [Team](crate::db::Team) records.
        teams: as_ref { &[Uuid], Vec<Uuid> },
    }
}

impl_default!(Factory);
impl_display!(Factory, json);
