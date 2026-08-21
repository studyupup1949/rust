use activitystreams_vocabulary::{Iri as VocabIri, field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::crypto::SymmetricKey;
use crate::db::{
    Application, Create, DateTime, Db, Factory, Inbox, Iri, Like, OptionalDateTime,
    OptionalTableEntry, Outbox, PatchTracker, Person, Repository, RoleList, TableEntry, TableType,
    Team, TicketTracker, Uuid,
};
use crate::{
    Actor as VocabActor, Error, Grant as VocabGrant, Result, Role, impl_sql_activity,
    impl_sql_list_field, util,
};

/// Represents a [Grant](https://forgefed.org/spec/#grant) database record.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Grant {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    actor: TableEntry,
    objects: Vec<Role>,
    context: TableEntry,
    #[sqlx(rename = "target_entry")]
    target: Option<TableEntry>,
    fulfills: Option<TableEntry>,
    start_time: Option<DateTime>,
    end_time: Option<DateTime>,
}

impl Grant {
    /// Creates a new [Grant].
    pub const fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            actor: TableEntry::new(),
            objects: Vec::new(),
            context: TableEntry::new(),
            target: None,
            fulfills: None,
            start_time: None,
            end_time: None,
        }
    }

    /// Performs checks on record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.id.is_empty() {
            Err(Error::sql("grant: empty ID"))
        } else if self.actor.is_empty() {
            Err(Error::sql("grant: empty actor"))
        } else if self.context.is_empty() {
            Err(Error::sql("grant: empty context"))
        } else {
            Ok(())
        }
    }

    /// Gets whether the [Grant] permits the requested access.
    ///
    /// In other words, whether the `actor` has `object` ([Role]) access to the `context` resource.
    ///
    /// - `actor`: the actor requesting access
    /// - `object`: the access [Role] the `actor` is requesting
    /// - `context`: the resource to which `actor` is requesting access
    pub fn permits(&self, actor: TableEntry, object: Role, context: TableEntry) -> bool {
        self.actor == actor && self.objects.contains(&object) && self.context == context
    }

    /// Attempts to find the [Grant] by ID.
    pub async fn find_by_id(db: &Db, id: &Iri) -> Result<Option<Self>> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let grant = Self::find_by_id_tx(&mut dbtx, id).await?;

        dbtx.commit()
            .await
            .map(|_| grant)
            .map_err(|err| Error::db(format!("grant: {err}")))
    }

    /// Attempts to find the [Grant] by ID using a transaction.
    pub async fn find_by_id_tx(
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        id: &Iri,
    ) -> Result<Option<Self>> {
        let table = Self::TABLE;

        sqlx::query(format!("SELECT * FROM {table} where id = $1").as_str())
            .bind(id)
            .fetch_optional(&mut **dbtx)
            .await
            .map_err(|err| Error::db(format!("grant: {err}")))
            .and_then(|row| {
                if let Some(row) = row {
                    Self::from_row(&row)
                        .map_err(|err| Error::db(format!("grant: {err}")))
                        .map(Some)
                } else {
                    Ok(None)
                }
            })
    }

    /// Attempts to find the [Grant] by actor table entry.
    pub async fn find_by_actor(db: &Db, actor: &TableEntry) -> Result<Vec<Self>> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let grant = Self::find_by_actor_tx(&mut dbtx, actor).await?;

        dbtx.commit().await.map(|_| grant).map_err(Error::from)
    }

    /// Attempts to find the [Grant] by actor table entry using a transaction.
    pub async fn find_by_actor_tx(
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        actor: &TableEntry,
    ) -> Result<Vec<Self>> {
        let table = Self::TABLE;

        sqlx::query(format!("SELECT * FROM {table} where actor = $1").as_str())
            .bind(actor)
            .fetch_all(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|rows| {
                rows.into_iter()
                    .map(|row| Self::from_row(&row).map_err(Error::from))
                    .collect::<Result<Vec<_>>>()
            })
    }

    /// Attempts to convert a [Grant] record into a [Grant](VocabGrant) JSON-LD object.
    pub async fn try_into_vocab(&self, db: &Db) -> Result<VocabGrant> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let grant = self.try_into_vocab_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| grant)
            .map_err(|err| Error::db(format!("grant: {err}")))
    }

    /// Attempts to convert a [Grant] record into a [Grant](VocabGrant) JSON-LD object using a transaction.
    pub async fn try_into_vocab_tx(
        &self,
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<VocabGrant> {
        let actor = Self::actor_entry_tx(dbtx, db_key, &self.actor).await?;
        let context = Self::object_entry_id_tx(dbtx, &self.context).await?;

        let mut grant = VocabGrant::new()
            .with_id(self.id.clone())
            .with_actor(actor)
            .with_object(self.objects.clone())
            .with_context(context);

        if let Some(target_entry) = self.target().as_ref() {
            Self::object_entry_id_tx(dbtx, target_entry)
                .await
                .map(|id| grant.set_target(id))?;
        }

        if let Some(fulfills_entry) = self.fulfills().as_ref() {
            Self::activity_entry_id_tx(dbtx, fulfills_entry)
                .await
                .map(|id| grant.set_fulfills(id))?;
        }

        if let Some(start_time) = self.start_time().copied() {
            grant.set_start_time(start_time);
        }

        if let Some(end_time) = self.end_time().copied() {
            grant.set_end_time(end_time);
        }

        Ok(grant)
    }

    /// Attempts to fetch an actor record by table entry.
    pub async fn actor_entry(db: &Db, entry: &TableEntry) -> Result<VocabActor> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;
        let db_key = db.key()?;

        let actor = Self::actor_entry_tx(&mut dbtx, &db_key, entry).await?;

        dbtx.commit()
            .await
            .map(|_| actor)
            .map_err(|err| Error::db(format!("grant: {err}")))
    }

    /// Attempts to fetch an actor record by table entry using a transaction.
    pub async fn actor_entry_tx(
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
        entry: &TableEntry,
    ) -> Result<VocabActor> {
        match entry.table() {
            TableType::Application => Application::get_tx(dbtx, &entry.id())
                .await?
                .try_into_vocab_tx(dbtx, db_key)
                .await
                .map(VocabActor::application),
            TableType::Factory => Factory::get_tx(dbtx, &entry.id())
                .await?
                .try_into_vocab_tx(dbtx, db_key)
                .await
                .map(VocabActor::factory),
            TableType::Person => Person::get_tx(dbtx, &entry.id())
                .await?
                .try_into_vocab_tx(dbtx, db_key)
                .await
                .map(VocabActor::person),
            TableType::Repository => Repository::get_tx(dbtx, &entry.id())
                .await?
                .try_into_vocab_tx(dbtx, db_key)
                .await
                .map(VocabActor::repository),
            TableType::Team => Team::get_tx(dbtx, &entry.id())
                .await?
                .try_into_vocab_tx(dbtx, db_key)
                .await
                .map(VocabActor::team),
            TableType::PatchTracker => PatchTracker::get_tx(dbtx, &entry.id())
                .await?
                .try_into_vocab_tx(dbtx, db_key)
                .await
                .map(VocabActor::patchtracker),
            TableType::TicketTracker => TicketTracker::get_tx(dbtx, &entry.id())
                .await?
                .try_into_vocab_tx(dbtx, db_key)
                .await
                .map(VocabActor::tickettracker),
            table => Err(Error::db(format!("grant: unsupported entry type: {table}"))),
        }
    }

    /// Gets an object ID based on a table entry.
    pub async fn object_entry_id(db: &Db, entry: &TableEntry) -> Result<VocabIri> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let id = Self::object_entry_id_tx(&mut dbtx, entry).await?;

        dbtx.commit()
            .await
            .map(|_| id)
            .map_err(|err| Error::db(format!("grant: {err}")))
    }

    /// Gets an object ID based on a table entry using a transaction.
    pub async fn object_entry_id_tx(
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        entry: &TableEntry,
    ) -> Result<VocabIri> {
        match entry.table() {
            TableType::Application => Application::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::Factory => Factory::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::Person => Person::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::Repository => Repository::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::Team => Team::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::PatchTracker => PatchTracker::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::TicketTracker => TicketTracker::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::Inbox => Inbox::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::Outbox => Outbox::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            table => Err(Error::db(format!("grant: unsupported entry type: {table}"))),
        }
    }

    /// Gets an activity ID based on a table entry.
    pub async fn activity_entry_id(db: &Db, entry: &TableEntry) -> Result<VocabIri> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let id = Self::activity_entry_id_tx(&mut dbtx, entry).await?;

        dbtx.commit()
            .await
            .map(|_| id)
            .map_err(|err| Error::db(format!("grant: {err}")))
    }

    /// Gets an activity ID based on a table entry using a transaction.
    pub async fn activity_entry_id_tx(
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        entry: &TableEntry,
    ) -> Result<VocabIri> {
        match entry.table() {
            TableType::Create => Create::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::Grant => Grant::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::Like => Like::get_tx(dbtx, &entry.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            table => Err(Error::db(format!(
                "grant: unsupported activity entry type: {table}"
            ))),
        }
    }
}

field_access! {
    Grant {
        /// Represents the primary key for the [Grant] record.
        uuid: Uuid,
        /// References the actor record the [Grant] gives access to the `context` resource.
        actor: TableEntry,
        /// References the resource to which the [Grant] gives the `actor` access.
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

field_access! {
    Grant {
        /// Represents the effective start time when then [Grant].
        start_time: option_ref { DateTime },
        /// Represents the effective end time when then [Grant].
        end_time: option_ref { DateTime },
    }
}

impl_default!(Grant);
impl_display!(Grant, json);

impl_sql_list_field! {
    Grant {
        /// Represents the [Role]s used to fine-tune access to the `context` resource.
        object, objects: { "objects" Role },
    }
}

impl_sql_activity! {
    Grant {
        id: { "id" Iri },
        actor: { "actor" TableEntry },
        objects: { "objects" RoleList },
        context: { "context" TableEntry },
        target: { "target_entry" OptionalTableEntry },
        fulfills: { "fulfills" OptionalTableEntry },
        start_time: { "start_time" OptionalDateTime },
        end_time: { "end_time" OptionalDateTime },
    }
}
