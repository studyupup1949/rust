use activitystreams_vocabulary::{Iri as VocabIri, field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::{Db, Iri, Transaction, Uuid};
use crate::{Error, Like as VocabLike, Result, impl_sql_activity, util};

/// Represents a [Like](https://forgefed.org/spec/#like) database record.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Like {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    actor: Iri,
    object: Iri,
}

impl Like {
    /// Creates a new [Like].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            actor: Iri::new(),
            object: Iri::new(),
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

    /// Attempts to convert a [Like](VocabLike) JSON-LD object into a [Like] record.
    ///
    /// **NOTE**: `object_uri` is the expected `Object` URI for the [Like].
    pub async fn try_from_vocab(db: &Db, object_uri: &Iri, val: &VocabLike) -> Result<Self> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let like = Self::try_from_vocab_tx(&mut dbtx, object_uri, val).await?;

        dbtx.commit()
            .await
            .map(|_| like)
            .map_err(|err| Error::db(format!("like: {err}")))
    }

    /// Attempts to convert a [Like](VocabLike) JSON-LD object into a [Like] record using a transaction.
    ///
    /// **NOTE**: `object_uri` is the expected `Object` URI for the [Like].
    pub async fn try_from_vocab_tx(
        dbtx: &mut Transaction<'_>,
        object_uri: &Iri,
        val: &VocabLike,
    ) -> Result<Self> {
        let object = val
            .object()
            .ok_or(Error::db("like: missing object"))
            .and_then(|v| v.ids().map_err(Error::from))
            .and_then(|v| {
                v.into_iter()
                    .next()
                    .ok_or(Error::db("like: missing object ID"))
            })
            .map(Iri::from)?;

        if object.as_str() != object_uri.as_str() {
            Err(Error::db(
                "like: invalid object ID: {object}, expected: {object_uri}",
            ))
        } else {
            let actor = val
                .actor()
                .ok_or(Error::db("like: missing actor"))
                .and_then(|v| v.ids().map_err(Error::from))
                .and_then(|v| {
                    v.into_iter()
                        .next()
                        .ok_or(Error::db("like: missing actor ID"))
                })
                .map(Iri::from)?;

            let uuid = Self::TABLE.uuid_from_ids([&actor, &object]);
            let id = Self::TABLE.id_from_uuid(object_uri, uuid)?;

            let mut like = Self {
                uuid,
                id,
                actor,
                object,
            };

            like.find_or_create_tx(dbtx).await.map(|_| like)
        }
    }

    /// Attempts to convert a [Like] record into a [Like](VocabLike) JSON-LD object.
    pub fn try_into_vocab(&self) -> Result<VocabLike> {
        let actor = VocabIri::from(self.actor());
        let object = VocabIri::from(self.object());

        Ok(VocabLike::new().with_actor(actor).with_object(object))
    }
}

field_access! {
    Like {
        /// Represents the primary key for the [Like] record.
        uuid: Uuid,
    }
}

field_access! {
    Like {
        /// Represents the IRI used to fetch the [Like].
        id: as_ref { Iri },
        /// References the actor record for the publisher of the [Like] activity.
        actor: as_ref { Iri },
        /// References the resource being [Like]d.
        object: as_ref { Iri },
    }
}

impl_default!(Like);
impl_display!(Like, json);

impl_sql_activity! {
    Like {
        id: { "id" Iri },
        actor: { "actor" Iri },
        object: { "object" Iri },
    }
}
