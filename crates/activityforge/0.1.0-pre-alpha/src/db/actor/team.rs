use activitystreams_vocabulary::{DateTime, field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};

use crate::db::{Db, Iri, Name, Uuid};
use crate::{Error, Result, impl_sql_record, util};

/// Represents a [Team Actor](https://forgefed.org/spec/#team) database record.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "util::ser_uuid_opt",
        deserialize_with = "util::de_uuid_opt"
    )]
    context: Option<Uuid>,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    inbox: Uuid,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    outbox: Uuid,
    published: DateTime,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    members: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    subteams: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    oversees: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    overseen_by: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    role_filter: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    key_ids: Vec<Uuid>,
}

impl Team {
    /// Creates a new [Team].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            name: Name::new(),
            summary: None,
            content: None,
            context: None,
            inbox: Uuid::nil(),
            outbox: Uuid::nil(),
            published: DateTime::default(),
            members: Vec::new(),
            subteams: Vec::new(),
            oversees: Vec::new(),
            overseen_by: Vec::new(),
            role_filter: Vec::new(),
            key_ids: Vec::new(),
        }
    }

    /// Attempts to get a [Team] record by [Uuid].
    pub async fn get(db: &Db, uuid: &Uuid) -> Result<Self> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let factory = Self::get_tx(&mut dbtx, uuid).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| factory)
    }

    /// Attempts to get a [Team] record by [Uuid] using a DB transaction.
    pub async fn get_tx(dbtx: &mut Transaction<'_, Postgres>, uuid: &Uuid) -> Result<Self> {
        sqlx::query("SELECT * FROM team WHERE uuid = $1")
            .bind(uuid)
            .fetch_one(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|row| {
                Ok(Self {
                    uuid: *uuid,
                    id: row.try_get::<Iri, &str>("id")?,
                    name: row.try_get::<Name, &str>("name")?,
                    summary: row.try_get::<Option<String>, &str>("summary")?,
                    content: row.try_get::<Option<String>, &str>("content")?,
                    context: row.try_get::<Option<Uuid>, &str>("content")?,
                    inbox: row.try_get::<Uuid, &str>("inbox")?,
                    outbox: row.try_get::<Uuid, &str>("outbox")?,
                    published: row.try_get::<DateTime, &str>("published")?,
                    members: row.try_get::<Vec<Uuid>, &str>("members")?,
                    subteams: row.try_get::<Vec<Uuid>, &str>("subteams")?,
                    oversees: row.try_get::<Vec<Uuid>, &str>("oversees")?,
                    overseen_by: row.try_get::<Vec<Uuid>, &str>("overseen_by")?,
                    role_filter: row.try_get::<Vec<Uuid>, &str>("role_filter")?,
                    key_ids: row.try_get::<Vec<Uuid>, &str>("key_ids")?,
                })
            })
    }

    /// Attempts to insert a [Team] record into the database.
    pub async fn insert(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let uuid = self.insert_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| uuid).map_err(Error::from)
    }

    /// Attempts to insert a [Team] record into the database using a [Transaction].
    pub async fn insert_tx(&mut self, dbtx: &mut Transaction<'_, Postgres>) -> Result<Uuid> {
        self.db_checks()?;

        let row = if self.uuid.is_nil() {
            sqlx::query(
                "INSERT INTO team
                (id, name, summary, content, context, inbox, outbox, members, subteams, oversees, overseen_by, role_filter, key_ids)
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13)
                RETURNING uuid, published",
            )
            .bind(self.id.as_str())
            .bind(self.name.as_str())
            .bind(self.summary.as_deref())
            .bind(self.content.as_deref())
            .bind(self.context)
            .bind(self.inbox)
            .bind(self.outbox)
            .bind(self.members.as_slice())
            .bind(self.subteams.as_slice())
            .bind(self.oversees.as_slice())
            .bind(self.overseen_by.as_slice())
            .bind(self.role_filter.as_slice())
            .bind(self.key_ids.as_slice())
            .fetch_one(&mut **dbtx)
            .await?
        } else {
            sqlx::query(
                "INSERT INTO team
                (uuid, id, name, summary, content, context, inbox, outbox, members, subteams, oversees, overseen_by, role_filter, key_ids)
                values ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                RETURNING uuid, published",
            )
            .bind(self.uuid)
            .bind(self.id.as_str())
            .bind(self.name.as_str())
            .bind(self.summary.as_deref())
            .bind(self.content.as_deref())
            .bind(self.context)
            .bind(self.inbox)
            .bind(self.outbox)
            .bind(self.members.as_slice())
            .bind(self.subteams.as_slice())
            .bind(self.oversees.as_slice())
            .bind(self.overseen_by.as_slice())
            .bind(self.role_filter.as_slice())
            .bind(self.key_ids.as_slice())
            .fetch_one(&mut **dbtx)
            .await?
        };

        let uuid = row.try_get::<Uuid, &str>("uuid")?;
        self.published = row.try_get::<DateTime, &str>("published")?;

        if self.uuid.is_nil() {
            self.uuid = uuid;
        }

        Ok(uuid)
    }
    /// Attempts to update a [Team] record in the database.
    pub async fn update(&self, db: &Db) -> Result<()> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        self.update_tx(&mut dbtx).await?;

        dbtx.commit().await.map(|_| ()).map_err(Error::from)
    }

    /// Attempts to update a [Team] record in the database using a [Transaction].
    pub async fn update_tx(&self, dbtx: &mut Transaction<'_, Postgres>) -> Result<()> {
        util::check_uuid("team", &self.uuid).and_then(|_| self.db_checks())?;

        sqlx::query(
            "UPDATE team
            SET
            (id, name, summary, content, context, inbox, outbox, members, subteams, oversees, overseen_by, role_filter, key_ids)
            =
            ($2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            WHERE uuid = $1",
        )
        .bind(self.uuid)
        .bind(self.id.as_str())
        .bind(self.name.as_str())
        .bind(self.summary.as_deref())
        .bind(self.content.as_deref())
        .bind(self.context)
        .bind(self.inbox)
        .bind(self.outbox)
        .bind(self.members.as_slice())
        .bind(self.subteams.as_slice())
        .bind(self.oversees.as_slice())
        .bind(self.overseen_by.as_slice())
        .bind(self.role_filter.as_slice())
        .bind(self.key_ids.as_slice())
        .execute(&mut **dbtx)
        .await
        .map(|_| ())
        .map_err(|err| Error::sql(format!("team: error updating record: {err}")))
    }

    fn db_checks(&self) -> Result<()> {
        if self.id.is_empty() {
            return Err(Error::sql("team: empty ID"));
        }

        if self.name.as_str().is_empty() {
            return Err(Error::sql("team: empty name"));
        }

        if self.inbox.is_nil() {
            return Err(Error::sql("team: nil inbox"));
        }

        if self.outbox.is_nil() {
            return Err(Error::sql("team: nil outbox"));
        }

        if let Some(context) = self.context.as_ref()
            && !self.uuid.is_nil()
            && &self.uuid == context
        {
            return Err(Error::sql("team: context references this Team"));
        }

        if !self.uuid.is_nil() && self.subteams.contains(&self.uuid) {
            return Err(Error::sql(
                "team: subteams contains a reference to this Team",
            ));
        }

        if !self.uuid.is_nil() && self.oversees.contains(&self.uuid) {
            return Err(Error::sql(
                "team: oversees contains a reference to this Team",
            ));
        }

        if !self.uuid.is_nil() && self.overseen_by.contains(&self.uuid) {
            return Err(Error::sql(
                "team: overseen_by contains a reference to this Team",
            ));
        }

        Ok(())
    }
}

impl_sql_record!(Team);

field_access! {
    Team {
        /// Represents the [Uuid] primary key of the table entry.
        uuid: Uuid,
        /// Represents the [Uuid] of the [Inbox](crate::db::Inbox) record.
        inbox: Uuid,
        /// Represents the [Uuid] of the [Outbox](crate::db::Outbox) record.
        outbox: Uuid,
    }
}

field_access! {
    Team {
        /// Represents the IRI used to fetch the [Team] record.
        id: as_ref { Iri },
        /// Represents the human-readable [Team] name.
        name: as_ref { Name },
        /// Represents the timestamp for the [Team]'s creation.
        published: as_ref { DateTime },
    }
}

field_access! {
    Team {
        /// Represents the [Team]'s content description.
        content: option_deref { &str, String },
        /// Represents the [Team]'s summary.
        summary: option_deref { &str, String },
    }
}

field_access! {
    Team {
        /// References the list of [Collaborator](crate::db::Collaborator) members.
        members: as_ref { &[Uuid], Vec<Uuid> },
        /// References the list of [subteams](Self).
        subteams: as_ref { &[Uuid], Vec<Uuid> },
        /// References the list of [Team]s this [Team] oversees.
        oversees: as_ref { &[Uuid], Vec<Uuid> },
        /// References the list of [Team]s overseen by this [Team].
        overseen_by: as_ref { &[Uuid], Vec<Uuid> },
        /// References the list of [RoleFilter] entries for [Team] members.
        ///
        /// Allows for fine-grained access control for individual [Team] members to shared resources.
        role_filter: as_ref { &[Uuid], Vec<Uuid> },
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
    }
}

impl_default!(Team);
impl_display!(Team, json);
