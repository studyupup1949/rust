use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};

use crate::db::{Db, TableEntry, Uuid};
use crate::{Error, Result, Role, impl_sql_record, util};

/// Represents a [Collaborator Relationship](https://forgefed.org/spec#collaborators)
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Collaborator {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    subject: TableEntry,
    object: TableEntry,
    instrument: Role,
}

impl Collaborator {
    /// Creates a new [Collaborator].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            subject: TableEntry::new(),
            object: TableEntry::new(),
            instrument: Role::new(),
        }
    }

    /// Attempts to get a [Collaborator] record by [Uuid].
    pub async fn get(db: &Db, uuid: &Uuid) -> Result<Self> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let factory = Self::get_tx(&mut dbtx, uuid).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| factory)
    }

    /// Attempts to get a [Collaborator] record by [Uuid] using a DB transaction.
    pub async fn get_tx(dbtx: &mut Transaction<'_, Postgres>, uuid: &Uuid) -> Result<Self> {
        util::check_uuid("collaborator", uuid)?;

        sqlx::query("SELECT * FROM collaborator WHERE uuid = $1")
            .bind(uuid)
            .fetch_one(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|row| {
                Ok(Self {
                    uuid: *uuid,
                    subject: row.try_get::<TableEntry, &str>("subject")?,
                    object: row.try_get::<TableEntry, &str>("object")?,
                    instrument: row.try_get::<Role, &str>("instrument")?,
                })
            })
    }

    /// Attempts to insert a [Collaborator] record into the database.
    pub async fn insert(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let uuid = self.insert_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| uuid).map_err(Error::from)
    }

    /// Attempts to insert a [Collaborator] record into the database using a [Transaction]..
    pub async fn insert_tx(&mut self, dbtx: &mut Transaction<'_, Postgres>) -> Result<Uuid> {
        if self.subject.is_empty() {
            return Err(Error::sql("collaborator: empty subject"));
        }

        if self.object.is_empty() {
            return Err(Error::sql("collaborator: empty object"));
        }

        let row = if self.uuid.is_nil() {
            sqlx::query(
                "INSERT INTO collaborator
                (subject, object, instrument)
                values ($1, $2, $3)
                RETURNING uuid",
            )
            .bind(self.subject())
            .bind(self.object())
            .bind(self.instrument)
            .fetch_one(&mut **dbtx)
            .await?
        } else {
            sqlx::query(
                "INSERT INTO collaborator
                (uuid, subject, object, instrument)
                values ($1, $2, $3, $4)
                RETURNING uuid",
            )
            .bind(self.uuid)
            .bind(self.subject())
            .bind(self.object())
            .bind(self.instrument)
            .fetch_one(&mut **dbtx)
            .await?
        };

        let uuid = row.try_get::<Uuid, &str>("uuid")?;

        if self.uuid.is_nil() {
            self.uuid = uuid;
        }

        Ok(uuid)
    }

    /// Attempts to update a [Collaborator] record in the database.
    pub async fn update(&self, db: &Db) -> Result<()> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        self.update_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| ()).map_err(Error::from)
    }

    /// Attempts to update a [Collaborator] record in the database using a [Transaction]..
    pub async fn update_tx(&self, dbtx: &mut Transaction<'_, Postgres>) -> Result<()> {
        util::check_uuid("collaborator", &self.uuid)?;

        if self.subject.is_empty() {
            return Err(Error::sql("collaborator: empty subject"));
        }

        if self.object.is_empty() {
            return Err(Error::sql("collaborator: empty object"));
        }

        sqlx::query(
            "UPDATE collaborator
            SET
            (subject, object, instrument)
            =
            ($2, $3, $4)
            WHERE uuid = $1",
        )
        .bind(self.uuid)
        .bind(self.subject())
        .bind(self.object())
        .bind(self.instrument)
        .execute(&mut **dbtx)
        .await
        .map(|_| ())
        .map_err(|err| Error::sql(format!("collaborator: error updating record: {err}")))
    }
}

field_access! {
    Collaborator {
        /// Main [Uuid] primary key for the record.
        uuid: Uuid,
        /// Represents the subject actor that has a collaborator.
        subject: TableEntry,
        /// Represents the collaborator actor.
        object: TableEntry,
        /// Represents the access-control [Role] the collaborator has on the `subject`'s resources.
        instrument: Role,
    }
}

impl_default!(Collaborator);
impl_display!(Collaborator, json);
impl_sql_record!(Collaborator);
