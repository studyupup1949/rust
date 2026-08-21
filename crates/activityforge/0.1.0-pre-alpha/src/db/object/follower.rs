use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};

use crate::db::{Db, TableEntry, Uuid};
use crate::{Error, Result, impl_sql_list_field, impl_sql_record, util};

/// Represents a an entry from a [Followers Collection](https://www.w3.org/TR/activitypub/#followers).
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Follower {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    actor: TableEntry,
    following: Vec<TableEntry>,
}

impl Follower {
    /// Creates a new [Follower].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            actor: TableEntry::new(),
            following: Vec::new(),
        }
    }

    /// Attempts to get a [Follower] record by [Uuid].
    pub async fn get(db: &Db, uuid: &Uuid) -> Result<Self> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let factory = Self::get_tx(&mut dbtx, uuid).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| factory)
    }

    /// Attempts to get a [Follower] record by [Uuid] using a DB transaction.
    pub async fn get_tx(dbtx: &mut Transaction<'_, Postgres>, uuid: &Uuid) -> Result<Self> {
        util::check_uuid("follower", uuid)?;

        sqlx::query("SELECT * FROM follower WHERE uuid = $1")
            .bind(uuid)
            .fetch_one(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|row| {
                Ok(Self {
                    uuid: *uuid,
                    actor: row.try_get::<TableEntry, &str>("actor")?,
                    following: row.try_get::<Vec<TableEntry>, &str>("following")?,
                })
            })
    }

    /// Attempts to insert a [Follower] record into the database.
    pub async fn insert(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let uuid = self.insert_tx(&mut dbtx).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| uuid)
    }

    /// Attempts to insert a [Follower] record into the database using a [Transaction].
    pub async fn insert_tx(&mut self, dbtx: &mut Transaction<'_, Postgres>) -> Result<Uuid> {
        if self.actor.is_empty() {
            return Err(Error::sql("follower: empty actor"));
        }

        if self.following.is_empty() {
            return Err(Error::sql("follower: empty following list"));
        }

        let row = if self.uuid.is_nil() {
            sqlx::query(
                "INSERT INTO follower
                (actor, following)
                values ($1, $2)
                RETURNING uuid",
            )
            .bind(self.actor())
            .bind(self.following())
            .fetch_one(&mut **dbtx)
            .await?
        } else {
            sqlx::query(
                "INSERT INTO follower
                (uuid, actor, following)
                values ($1, $2, $3)
                RETURNING uuid",
            )
            .bind(self.uuid)
            .bind(self.actor())
            .bind(self.following())
            .fetch_one(&mut **dbtx)
            .await?
        };

        let uuid = row.try_get::<Uuid, &str>("uuid")?;

        if self.uuid.is_nil() {
            self.uuid = uuid;
        }

        Ok(uuid)
    }

    /// Attempts to update a [Follower] record in the database.
    pub async fn update(&self, db: &Db) -> Result<()> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        self.update_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| ()).map_err(Error::from)
    }

    /// Attempts to update a [Follower] record in the database using a [Transaction].
    pub async fn update_tx(&self, dbtx: &mut Transaction<'_, Postgres>) -> Result<()> {
        util::check_uuid("follower", &self.uuid)?;

        if self.actor.is_empty() {
            return Err(Error::sql("follower: empty actor"));
        }

        if self.following.is_empty() {
            return Err(Error::sql("follower: empty following list"));
        }

        sqlx::query(
            "UPDATE follower
            SET
            (actor, following)
            =
            ($2, $3)
            WHERE uuid = $1",
        )
        .bind(self.uuid)
        .bind(self.actor())
        .bind(self.following())
        .execute(&mut **dbtx)
        .await
        .map(|_| ())
        .map_err(|err| Error::sql(format!("follower: error updating record: {err}")))
    }
}

field_access! {
    Follower {
        /// Represents the [Uuid] primary key of the table entry.
        uuid: Uuid,
        /// References the actor following the other actors.
        actor: TableEntry,
    }
}

impl_default!(Follower);
impl_display!(Follower, json);
impl_sql_record!(Follower);

impl_sql_list_field! {
    Follower {
        /// Represents the list of actors the [Follower] is following.
        follow, following: "following" TableEntry,
    }
}
