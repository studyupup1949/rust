use activitystreams_vocabulary::{Follow as VocabFollow, field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::crypto::SymmetricKey;
use crate::db::{
    Application, Db, Factory, Iri, PatchTracker, Person, Repository, TableEntry, TableType,
    TicketTracker, Uuid,
};
use crate::{Actor as VocabActor, Error, Result, impl_sql_activity, util};

/// Represents a [Follow](https://www.w3.org/TR/activitystreams/#follow) database record.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Follow {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    actor: TableEntry,
    object: TableEntry,
}

impl Follow {
    /// Follows a new [Follow].
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

    /// Attempts to convert a [Follow] record into a [Follow](VocabFollow) JSON-LD object.
    pub async fn try_into_vocab(&self, db: &Db) -> Result<VocabFollow> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;
        let db_key = db.key()?;

        let follow = self.try_into_vocab_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| follow)
            .map_err(|err| Error::db(format!("follow: {err}")))
    }

    /// Attempts to convert a [Follow] record into a [Follow](VocabFollow) JSON-LD object using a transaction.
    pub async fn try_into_vocab_tx(
        &self,
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<VocabFollow> {
        let actor = match self.actor.table() {
            TableType::Person => {
                let person = Person::get_tx(dbtx, &self.actor.id()).await?;
                person
                    .try_into_vocab_tx(dbtx, db_key)
                    .await
                    .map(VocabActor::person)
            }
            table => Err(Error::db(format!(
                "follow: unsupported actor type: {table}"
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
                "follow: unsupported object type: {table}"
            ))),
        }?;

        Ok(VocabFollow::new()
            .with_id(self.id.clone())
            .with_actor(actor)
            .with_object(object))
    }
}

field_access! {
    Follow {
        /// Represents the primary key for the [Follow] record.
        uuid: Uuid,
        /// References the actor record for the publisher of the [Follow] activity.
        actor: TableEntry,
        /// References the resource being [Follow]d.
        object: TableEntry,
    }
}

field_access! {
    Follow {
        /// Represents the IRI used to fetch the [Follow].
        id: as_ref { Iri },
    }
}

impl_default!(Follow);
impl_display!(Follow, json);

impl_sql_activity! {
    Follow {
        id: { "id" Iri },
        actor: { "actor" TableEntry },
        object: { "object" TableEntry },
    }
}
