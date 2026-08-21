use activitystreams_vocabulary::{
    Accept as VocabAccept, Iri as VocabIri, field_access, impl_default, impl_display,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::crypto::SymmetricKey;
use crate::db::{Create, Db, Follow, Grant, Iri, Like, Person, TableEntry, TableType, Uuid};
use crate::{Actor as VocabActor, Error, Result, impl_sql_activity, util};

/// Represents a [Accept](https://www.w3.org/TR/activitystreams/#accept) database record.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Accept {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    actor: TableEntry,
    object: TableEntry,
}

impl Accept {
    /// Accepts a new [Accept].
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

    /// Attempts to convert a [Accept] record into a [Accept](VocabAccept) JSON-LD object.
    pub async fn try_into_vocab(&self, db: &Db) -> Result<VocabAccept> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;
        let db_key = db.key()?;

        let accept = self.try_into_vocab_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| accept)
            .map_err(|err| Error::db(format!("accept: {err}")))
    }

    /// Attempts to convert a [Accept] record into a [Accept](VocabAccept) JSON-LD object using a transaction.
    pub async fn try_into_vocab_tx(
        &self,
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<VocabAccept> {
        let actor = match self.actor.table() {
            TableType::Person => {
                let person = Person::get_tx(dbtx, &self.actor.id()).await?;
                person
                    .try_into_vocab_tx(dbtx, db_key)
                    .await
                    .map(VocabActor::person)
            }
            table => Err(Error::db(format!(
                "accept: unsupported actor type: {table}"
            ))),
        }?;

        let object = match self.object.table() {
            TableType::Create => Create::get_tx(dbtx, &self.object.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::Follow => Follow::get_tx(dbtx, &self.object.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::Grant => Grant::get_tx(dbtx, &self.object.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            TableType::Like => Like::get_tx(dbtx, &self.object.id())
                .await
                .map(|e| VocabIri::from(e.id())),
            table => Err(Error::db(format!(
                "accept: unsupported object type: {table}"
            ))),
        }?;

        Ok(VocabAccept::new()
            .with_id(self.id.clone())
            .with_actor(actor)
            .with_object(object))
    }
}

field_access! {
    Accept {
        /// Represents the primary key for the [Accept] record.
        uuid: Uuid,
        /// References the actor record for the publisher of the [Accept] activity.
        actor: TableEntry,
        /// References the resource being [Accept]d.
        object: TableEntry,
    }
}

field_access! {
    Accept {
        /// Represents the IRI used to fetch the [Accept].
        id: as_ref { Iri },
    }
}

impl_default!(Accept);
impl_display!(Accept, json);

impl_sql_activity! {
    Accept {
        id: { "id" Iri },
        actor: { "actor" TableEntry },
        object: { "object" TableEntry },
    }
}
