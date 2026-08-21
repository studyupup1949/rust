use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};

use crate::db::{Db, Iri, TableEntry, TableType, Uuid};
use crate::{Error, Result, impl_sql_record, util};

/// Represents a [Like](https://forgefed.org/spec/#like) database record.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Like {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    actor: TableEntry,
    object: TableEntry,
}

impl Like {
    /// Creates a new [Like].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            actor: TableEntry::new(),
            object: TableEntry::new(),
        }
    }

    /// Attempts to get a [Like] record by [Uuid].
    pub async fn get(db: &Db, uuid: &Uuid) -> Result<Self> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let factory = Self::get_tx(&mut dbtx, uuid).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| factory)
    }

    /// Attempts to get a [Like] record by [Uuid] using a DB transaction.
    pub async fn get_tx(dbtx: &mut Transaction<'_, Postgres>, uuid: &Uuid) -> Result<Self> {
        let table = TableType::Like;
        sqlx::query(format!("SELECT * FROM {table} WHERE uuid = $1").as_str())
            .bind(uuid)
            .fetch_one(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|row| {
                Ok(Self {
                    uuid: *uuid,
                    id: row.try_get::<Iri, &str>("id")?,
                    actor: row.try_get::<TableEntry, &str>("actor")?,
                    object: row.try_get::<TableEntry, &str>("object")?,
                })
            })
    }

    /// Attempts to insert a [Like] record into the database.
    pub async fn insert(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let uuid = self.insert_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| uuid).map_err(Error::from)
    }

    /// Attempts to insert a [Like] record into the database using a [Transaction].
    pub async fn insert_tx(&mut self, dbtx: &mut Transaction<'_, Postgres>) -> Result<Uuid> {
        let table = self.table();

        if self.id.is_empty() {
            return Err(Error::sql("like: empty ID"));
        }

        if self.actor.is_empty() {
            return Err(Error::sql("like: empty actor"));
        }

        if self.object.is_empty() {
            return Err(Error::sql("like: empty actor"));
        }

        let row = if self.uuid.is_nil() {
            sqlx::query(
                format!(
                    "INSERT INTO {table}
                    (id, actor, object)
                    VALUES
                    ($1, $2, $3)
                    RETURNING uuid"
                )
                .as_str(),
            )
            .bind(self.id())
            .bind(self.actor())
            .bind(self.object())
            .fetch_one(&mut **dbtx)
            .await?
        } else {
            sqlx::query(
                format!(
                    "INSERT INTO {table}
                    (uuid, id, actor, object)
                    VALUES
                    ($1, $2, $3, $4)
                    RETURNING uuid"
                )
                .as_str(),
            )
            .bind(self.uuid)
            .bind(self.id())
            .bind(self.actor())
            .bind(self.object())
            .fetch_one(&mut **dbtx)
            .await?
        };

        let uuid = row.try_get::<Uuid, &str>("uuid")?;

        if self.uuid.is_nil() {
            self.uuid = uuid;
        }

        Ok(uuid)
    }

    /// Attempts to update a [Like] record in the database.
    pub async fn update(&self, db: &Db) -> Result<()> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        self.update_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| ()).map_err(Error::from)
    }

    /// Attempts to update a [Like] record in the database using a [Transaction].
    pub async fn update_tx(&self, dbtx: &mut Transaction<'_, Postgres>) -> Result<()> {
        let table = self.table();

        util::check_uuid(table.as_str(), &self.uuid)?;

        if self.id.is_empty() {
            return Err(Error::sql("like: empty ID"));
        }

        if self.actor.is_empty() {
            return Err(Error::sql("like: empty actor"));
        }

        if self.object.is_empty() {
            return Err(Error::sql("like: empty object"));
        }

        sqlx::query(
            format!(
                "UPDATE {table}
                SET
                (id, actor, object)
                =
                ($2, $3, $4)
                WHERE uuid = $1"
            )
            .as_str(),
        )
        .bind(self.uuid)
        .bind(self.id.as_str())
        .bind(self.actor())
        .bind(self.object())
        .execute(&mut **dbtx)
        .await
        .map(|_| ())
        .map_err(|err| Error::sql(format!("like: error updating record: {err}")))
    }
}

field_access! {
    Like {
        /// Represents the primary key for the [Like] record.
        uuid: Uuid,
        /// References the actor record for the publisher of the [Like] activity.
        actor: TableEntry,
        /// References the resource being [Like]d.
        object: TableEntry,
    }
}

field_access! {
    Like {
        /// Represents the IRI used to fetch the [Like].
        id: as_ref { Iri },
    }
}

impl_default!(Like);
impl_display!(Like, json);
impl_sql_record!(Like);
