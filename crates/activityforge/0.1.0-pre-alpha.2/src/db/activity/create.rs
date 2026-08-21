use activitystreams_vocabulary::{Create as VocabCreate, field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::crypto::SymmetricKey;
use crate::db::{
    Application, Db, Factory, Iri, PatchTracker, Person, Repository, TableEntry, TableType,
    TicketTracker, Uuid,
};
use crate::{Actor as VocabActor, Error, Result, impl_sql_activity, util};

/// Represents a [Create](https://www.w3.org/TR/activitystreams/#create) database record.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Create {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    actor: TableEntry,
    object: TableEntry,
}

impl Create {
    /// Creates a new [Create].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            actor: TableEntry::new(),
            object: TableEntry::new(),
        }
    }

    /// Performs checks on record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.id.is_empty() {
            Err(Error::sql("like: empty ID"))
        } else if self.actor.is_empty() {
            Err(Error::sql("like: empty actor"))
        } else if self.object.is_empty() {
            Err(Error::sql("like: empty object"))
        } else {
            Ok(())
        }
    }

    /// Attempts to convert a [Create] record into a [Create](VocabCreate) JSON-LD object.
    pub async fn try_into_vocab(&self, db: &Db) -> Result<VocabCreate> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;
        let db_key = db.key()?;

        let create = self.try_into_vocab_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| create)
            .map_err(|err| Error::db(format!("create: {err}")))
    }

    /// Attempts to convert a [Create] record into a [Create](VocabCreate) JSON-LD object using a transaction.
    pub async fn try_into_vocab_tx(
        &self,
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<VocabCreate> {
        let actor = match self.actor.table() {
            TableType::Person => {
                let person = Person::get_tx(dbtx, &self.actor.id()).await?;
                person
                    .try_into_vocab_tx(dbtx, db_key)
                    .await
                    .map(VocabActor::person)
            }
            TableType::Factory => {
                let factory = Factory::get_tx(dbtx, &self.object.id()).await?;
                factory
                    .try_into_vocab_tx(dbtx, db_key)
                    .await
                    .map(VocabActor::factory)
            }
            table => Err(Error::db(format!(
                "create: unsupported actor type: {table}"
            ))),
        }?;

        let object = match self.object.table() {
            TableType::Application => {
                let app = Application::get_tx(dbtx, &self.object.id()).await?;
                app.try_into_vocab_tx(dbtx, db_key)
                    .await
                    .map(VocabActor::application)
            }
            TableType::Factory => {
                let factory = Factory::get_tx(dbtx, &self.object.id()).await?;
                factory
                    .try_into_vocab_tx(dbtx, db_key)
                    .await
                    .map(VocabActor::factory)
            }
            TableType::Person => {
                let person = Person::get_tx(dbtx, &self.object.id()).await?;
                person
                    .try_into_vocab_tx(dbtx, db_key)
                    .await
                    .map(VocabActor::person)
            }
            TableType::Repository => {
                let repo = Repository::get_tx(dbtx, &self.object.id()).await?;
                repo.try_into_vocab_tx(dbtx, db_key)
                    .await
                    .map(VocabActor::repository)
            }
            TableType::PatchTracker => {
                let tracker = PatchTracker::get_tx(dbtx, &self.object.id()).await?;
                tracker
                    .try_into_vocab_tx(dbtx, db_key)
                    .await
                    .map(VocabActor::patchtracker)
            }
            TableType::TicketTracker => {
                let tracker = TicketTracker::get_tx(dbtx, &self.object.id()).await?;
                tracker
                    .try_into_vocab_tx(dbtx, db_key)
                    .await
                    .map(VocabActor::tickettracker)
            }
            table => Err(Error::db(format!(
                "create: unsupported object type: {table}"
            ))),
        }?;

        Ok(VocabCreate::new()
            .with_id(self.id.clone())
            .with_actor(actor)
            .with_object(object))
    }
}

field_access! {
    Create {
        /// Represents the primary key for the [Create] record.
        uuid: Uuid,
        /// References the actor record for the publisher of the [Create] activity.
        actor: TableEntry,
        /// References the resource being [Create]d.
        object: TableEntry,
    }
}

field_access! {
    Create {
        /// Represents the IRI used to fetch the [Create].
        id: as_ref { Iri },
    }
}

impl_default!(Create);
impl_display!(Create, json);

impl_sql_activity! {
    Create {
        id: { "id" Iri },
        actor: { "actor" TableEntry },
        object: { "object" TableEntry },
    }
}
