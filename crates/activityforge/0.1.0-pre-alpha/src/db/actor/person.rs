use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};

use crate::db::{Db, Iri, Name, Uuid};
use crate::{Error, Result, impl_sql_record, util};

/// Represents a [Person Actor](https://forgefed.org/spec/#person) database record.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
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
    followers: Vec<Uuid>,
}

impl Person {
    /// Creates a new [Person].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            name: Name::new(),
            inbox: Uuid::nil(),
            outbox: Uuid::nil(),
            summary: None,
            content: None,
            key_ids: Vec::new(),
            followers: Vec::new(),
        }
    }

    /// Attempts to get a [Person] record by [Uuid].
    pub async fn get(db: &Db, uuid: &Uuid) -> Result<Self> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let factory = Self::get_tx(&mut dbtx, uuid).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| factory)
    }

    /// Attempts to get a [Person] record by [Uuid] using a DB transaction.
    pub async fn get_tx(dbtx: &mut Transaction<'_, Postgres>, uuid: &Uuid) -> Result<Self> {
        sqlx::query("SELECT * FROM person WHERE uuid = $1")
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
                    summary: row.try_get::<Option<String>, &str>("summary")?,
                    content: row.try_get::<Option<String>, &str>("content")?,
                    key_ids: row.try_get::<Vec<Uuid>, &str>("key_ids")?,
                    followers: row.try_get::<Vec<Uuid>, &str>("followers")?,
                })
            })
    }

    /// Attempts to insert a [Person] record into the database.
    pub async fn insert(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let uuid = self.insert_tx(&mut dbtx).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| uuid)
    }

    /// Attempts to insert a [Person] record into the database using a [Transaction].
    pub async fn insert_tx(&mut self, dbtx: &mut Transaction<'_, Postgres>) -> Result<Uuid> {
        if self.id.is_empty() {
            return Err(Error::sql("person: empty ID"));
        }

        if self.name.as_str().is_empty() {
            return Err(Error::sql("person: empty name"));
        }

        if self.inbox.is_nil() {
            return Err(Error::sql("person: nil inbox"));
        }

        if self.outbox.is_nil() {
            return Err(Error::sql("person: nil outbox"));
        }

        let row = if self.uuid.is_nil() {
            sqlx::query(
                "INSERT INTO person
                (id, name, inbox, outbox, summary, content, key_ids, followers)
                values ($1, $2, $3, $4, $5, $6, $7, $8)
                RETURNING uuid",
            )
            .bind(self.id())
            .bind(self.name())
            .bind(self.inbox)
            .bind(self.outbox)
            .bind(self.summary())
            .bind(self.content())
            .bind(self.key_ids())
            .bind(self.followers())
            .fetch_one(&mut **dbtx)
            .await?
        } else {
            sqlx::query(
                "INSERT INTO person
                (uuid, id, name, inbox, outbox, summary, content, key_ids, followers)
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                RETURNING uuid",
            )
            .bind(self.uuid)
            .bind(self.id())
            .bind(self.name())
            .bind(self.inbox)
            .bind(self.outbox)
            .bind(self.summary())
            .bind(self.content())
            .bind(self.key_ids())
            .bind(self.followers())
            .fetch_one(&mut **dbtx)
            .await?
        };

        let uuid = row.try_get::<Uuid, &str>("uuid")?;

        if self.uuid.is_nil() {
            self.uuid = uuid;
        }

        Ok(uuid)
    }

    /// Attempts to update a [Person] record in the database.
    pub async fn update(&self, db: &Db) -> Result<()> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        self.update_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| ()).map_err(Error::from)
    }

    /// Attempts to update a [Person] record in the database using a [Transaction].
    pub async fn update_tx(&self, dbtx: &mut Transaction<'_, Postgres>) -> Result<()> {
        util::check_uuid("person", &self.uuid)?;

        if self.id.is_empty() {
            return Err(Error::sql("person: empty ID"));
        }

        if self.name.as_str().is_empty() {
            return Err(Error::sql("person: empty name"));
        }

        if self.inbox.is_nil() {
            return Err(Error::sql("person: nil inbox"));
        }

        if self.outbox.is_nil() {
            return Err(Error::sql("person: nil outbox"));
        }

        sqlx::query(
            "UPDATE person
            SET
            (id, name, inbox, outbox, summary, content, key_ids, followers)
            =
            ($2, $3, $4, $5, $6, $7, $8, $9)
            WHERE uuid = $1",
        )
        .bind(self.uuid)
        .bind(self.id())
        .bind(self.name())
        .bind(self.inbox)
        .bind(self.outbox)
        .bind(self.summary())
        .bind(self.content())
        .bind(self.key_ids())
        .bind(self.followers())
        .execute(&mut **dbtx)
        .await
        .map(|_| ())
        .map_err(|err| Error::sql(format!("person: error updating record: {err}")))
    }
}

impl_sql_record!(Person);

field_access! {
    Person {
        /// Represents the [Uuid] primary key of the table entry.
        uuid: Uuid,
        /// Represents the [Uuid] of the [Inbox](crate::db::Inbox) record.
        inbox: Uuid,
        /// Represents the [Uuid] of the [Outbox](crate::db::Outbox) record.
        outbox: Uuid,
    }
}

field_access! {
    Person {
        /// Represents the IRI used to fetch the [Person] record.
        id: as_ref { Iri },
        /// Represents the human-readable [Person] name.
        name: as_ref { Name },
    }
}

field_access! {
    Person {
        /// Represents the [Person]'s content description.
        content: option_deref { &str, String },
        /// Represents the [Person]'s summary.
        summary: option_deref { &str, String },
    }
}

field_access! {
    Person {
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
        /// List of references to the [Factory]'s [Follower](crate::db::Follower) records.
        followers: as_ref { &[Uuid], Vec<Uuid> },
    }
}

impl_default!(Person);
impl_display!(Person, json);
