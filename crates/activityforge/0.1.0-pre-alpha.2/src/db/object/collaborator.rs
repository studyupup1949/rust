use activitystreams_vocabulary::{Item, Items, field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::{Db, Iri, Transaction, Uuid};
use crate::{
    ActorType, CollabRelationship, Collaborator as VocabCollaborator, Error, Result, Role,
    impl_sql_object, util,
};

/// Represents a [Collaborator Relationship](https://forgefed.org/spec#collaborators)
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Collaborator {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    subject: Iri,
    relationship: CollabRelationship,
    object: Iri,
    tag: Role,
}

impl Collaborator {
    /// Creates a new [Collaborator].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            subject: Iri::new(),
            relationship: CollabRelationship::new(),
            object: Iri::new(),
            tag: Role::new(),
        }
    }

    /// Performs checks on record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.subject.is_empty() {
            Err(Error::sql("collaborator: empty subject"))
        } else if self.object.is_empty() {
            Err(Error::sql("collaborator: empty object"))
        } else {
            Ok(())
        }
    }

    /// Attempts to convert a JSON-LD [Collaborator](VocabCollaborator) into a [Collaborator] record.
    pub async fn try_from_vocab(db: &Db, val: &VocabCollaborator) -> Result<Self> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let collab = Self::try_from_vocab_tx(&mut dbtx, val).await?;

        dbtx.commit()
            .await
            .map(|_| collab)
            .map_err(|err| Error::db(format!("collaborator: {err}")))
    }

    /// Attempts to convert a JSON-LD [Collaborator](VocabCollaborator) into a [Collaborator] record using a transaction.
    pub async fn try_from_vocab_tx(
        dbtx: &mut Transaction<'_>,
        val: &VocabCollaborator,
    ) -> Result<Self> {
        let subject_item = val
            .subject()
            .ok_or(Error::db("collaborator: missing subject"))?;

        let subject = match subject_item {
            Item::Object(object) => {
                let kind = object.kind();

                if kind.contains(ActorType::Factory) || kind.contains(ActorType::Team) {
                    object
                        .id()
                        .map(Iri::from)
                        .ok_or(Error::db("collaborator: missing subject ID"))
                } else {
                    Err(Error::db(format!("collaborator: invalid subject: {kind}")))
                }?
            }
            Item::Iri(iri) => iri.as_ref().into(),
            Item::Link(link) => link.href().into(),
        };

        let relationship = val
            .relationship()
            .ok_or(Error::db("collaborator: missing relationship"))?;

        let tag = match val.tag() {
            Some(Items::Single(Item::Iri(iri))) => Role::try_from(iri.as_ref()),
            Some(_) => Err(Error::db("collaborator: invalid tag")),
            None => Err(Error::db("collaborator: missing tag")),
        }?;

        let object_item = val
            .object()
            .ok_or(Error::db("collaborator: missing object"))?;

        let object = match object_item {
            Item::Object(object) => {
                let kind = object.kind();

                if kind.contains(ActorType::Person) {
                    object
                        .id()
                        .map(Iri::from)
                        .ok_or(Error::db("collaborator: missing object ID"))
                } else {
                    Err(Error::db(format!("collaborator: invalid object: {kind}")))
                }?
            }
            Item::Iri(iri) => iri.as_ref().into(),
            Item::Link(link) => link.href().into(),
        };

        let uuid = util::rand_uuid();
        let id = Self::TABLE.id_from_uuid(&subject, uuid)?;

        let mut collab = Self {
            uuid,
            id,
            subject,
            relationship,
            object,
            tag,
        };

        collab.find_or_create_tx(dbtx).await.map(|_| collab)
    }

    /// Attempts to convert a JSON-LD [Item] into a [Collaborator] record.
    pub async fn try_from_item(db: &Db, val: &Item) -> Result<Self> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let collab = Self::try_from_item_tx(&mut dbtx, val).await?;

        dbtx.commit()
            .await
            .map(|_| collab)
            .map_err(|err| Error::db(format!("collaborator: {err}")))
    }

    /// Attempts to convert a JSON-LD [Item] into a [Collaborator] record using a transaction.
    pub async fn try_from_item_tx(dbtx: &mut Transaction<'_>, val: &Item) -> Result<Self> {
        let object = val
            .as_object()
            .map_err(|err| Error::db(format!("collaborator: invalid item: {err}")))?;
        let collab = VocabCollaborator::try_from(object)
            .map_err(|err| Error::db(format!("collaborator: {err}")))?;
        Self::try_from_vocab_tx(dbtx, &collab).await
    }
}

field_access! {
    Collaborator {
        /// Main [Uuid] primary key for the record.
        uuid: Uuid,
        /// Represents the collaborator `object` relationship to the `subject`.
        relationship: CollabRelationship,
        /// Represents the access-control [Role] the collaborator has on the `subject`'s resources.
        tag: Role,
    }
}

field_access! {
    Collaborator {
        /// Represents the IRI used to fetch the [Collaborator].
        id: as_ref { Iri },
        /// Represents the subject IRI used to fetch the [Collaborator]'s subject.
        subject: as_ref { Iri },
        /// Represents the object IRI used to fetch the [Collaborator]'s object.
        object: as_ref { Iri },
    }
}

impl_default!(Collaborator);
impl_display!(Collaborator, json);

impl_sql_object! {
    Collaborator {
        id: { "id" Iri },
        subject: { "subject" Iri },
        relationship: { "relationship" CollabRelationship },
        object: { "object" Iri },
        tag: { "tag" Role },
    }
}
