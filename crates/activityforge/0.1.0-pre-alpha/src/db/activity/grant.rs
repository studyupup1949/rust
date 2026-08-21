use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};

use crate::db::{Db, Iri, TableEntry, Uuid};
use crate::{Error, Result, Role, impl_sql_record, util};

/// Represents a [Grant](https://forgefed.org/spec/#grant) database record.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Grant {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    actor: TableEntry,
    object: Role,
    context: TableEntry,
    target: Option<TableEntry>,
    fulfills: Option<TableEntry>,
}

impl Grant {
    /// Creates a new [Grant].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            actor: TableEntry::new(),
            object: Role::new(),
            context: TableEntry::new(),
            target: None,
            fulfills: None,
        }
    }

    /// Attempts to get a [Grant] record by [Uuid].
    pub async fn get(db: &Db, uuid: &Uuid) -> Result<Self> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let factory = Self::get_tx(&mut dbtx, uuid).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| factory)
    }

    /// Attempts to get a [Grant] record by [Uuid] using a DB transaction.
    pub async fn get_tx(dbtx: &mut Transaction<'_, Postgres>, uuid: &Uuid) -> Result<Self> {
        sqlx::query("SELECT * FROM role_grant WHERE uuid = $1")
            .bind(uuid)
            .fetch_one(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|row| {
                Ok(Self {
                    uuid: *uuid,
                    id: row.try_get::<Iri, &str>("id")?,
                    actor: row.try_get::<TableEntry, &str>("actor")?,
                    object: row.try_get::<Role, &str>("object")?,
                    context: row.try_get::<TableEntry, &str>("context")?,
                    target: row.try_get::<Option<TableEntry>, &str>("target")?,
                    fulfills: row.try_get::<Option<TableEntry>, &str>("fulfills")?,
                })
            })
    }

    /// Attempts to insert a [Grant] record into the database.
    pub async fn insert(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let uuid = self.insert_tx(&mut dbtx).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| uuid)
    }

    /// Attempts to insert a [Grant] record into the database using a [Transaction].
    pub async fn insert_tx(&mut self, dbtx: &mut Transaction<'_, Postgres>) -> Result<Uuid> {
        if self.id.is_empty() {
            return Err(Error::sql("grant: empty ID"));
        }

        if self.actor.is_empty() {
            return Err(Error::sql("grant: empty actor"));
        }

        if self.context.is_empty() {
            return Err(Error::sql("grant: empty actor"));
        }

        let row = if self.uuid.is_nil() {
            sqlx::query(
                "INSERT INTO role_grant
                (id, actor, object, context, target, fulfills)
                VALUES
                ($1, $2, $3, $4, $5, $6)
                RETURNING uuid",
            )
            .bind(self.id.as_str())
            .bind(self.actor())
            .bind(self.object)
            .bind(self.context())
            .bind(self.target())
            .bind(self.fulfills())
            .fetch_one(&mut **dbtx)
            .await?
        } else {
            sqlx::query(
                "INSERT INTO role_grant
                (uuid, id, actor, object, context, target, fulfills)
                VALUES
                ($1, $2, $3, $4, $5, $6, $7)
                RETURNING uuid",
            )
            .bind(self.uuid)
            .bind(self.id.as_str())
            .bind(self.actor())
            .bind(self.object)
            .bind(self.context())
            .bind(self.target())
            .bind(self.fulfills())
            .fetch_one(&mut **dbtx)
            .await?
        };

        let uuid = row.try_get::<Uuid, &str>("uuid")?;

        if self.uuid.is_nil() {
            self.uuid = uuid;
        }

        Ok(uuid)
    }

    /// Attempts to update a [Grant] record in the database.
    pub async fn update(&self, db: &Db) -> Result<()> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        self.update_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| ()).map_err(Error::from)
    }

    /// Attempts to update a [Grant] record in the database using a [Transaction].
    pub async fn update_tx(&self, dbtx: &mut Transaction<'_, Postgres>) -> Result<()> {
        util::check_uuid("grant", &self.uuid)?;

        if self.id.is_empty() {
            return Err(Error::sql("grant: empty ID"));
        }

        if self.actor.is_empty() {
            return Err(Error::sql("grant: empty actor"));
        }

        if self.context.is_empty() {
            return Err(Error::sql("grant: empty actor"));
        }

        sqlx::query(
            "UPDATE role_grant
            SET
            (id, actor, object, context, target, fulfills)
            =
            ($2, $3, $4, $5, $6)
            WHERE uuid = $1",
        )
        .bind(self.uuid)
        .bind(self.id.as_str())
        .bind(self.actor())
        .bind(self.object)
        .bind(self.context())
        .bind(self.target())
        .bind(self.fulfills())
        .execute(&mut **dbtx)
        .await
        .map(|_| ())
        .map_err(|err| Error::sql(format!("grant: error updating record: {err}")))
    }
}

field_access! {
    Grant {
        /// Represents the primary key for the [Grant] record.
        uuid: Uuid,
        /// References the actor record of the resource that is filtering.
        actor: TableEntry,
        /// Represents the [Role] used to fine-tune access to the filtered resource.
        object: Role,
        /// References the resource being given access by the [Grant].
        context: TableEntry,
    }
}

field_access! {
    Grant {
        /// Represents the IRI used to fetch the [Grant].
        id: as_ref { Iri },
    }
}

field_access! {
    Grant {
        /// References the actor record that inherits the [Role].
        target: option { TableEntry },
        /// References the activity that triggered the [Grant].
        fulfills: option { TableEntry },
    }
}

impl_default!(Grant);
impl_display!(Grant, json);
impl_sql_record!(Grant);
