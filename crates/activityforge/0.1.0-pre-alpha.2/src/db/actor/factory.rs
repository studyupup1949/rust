use activitystreams_vocabulary::{
    Iri as VocabIri, Key as VocabPublicKey, KeyItem, KeyItems, LinkItems, Multikey, MultikeyItem,
    MultikeyItems, Name as VocabName, field_access, impl_default, impl_display,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::crypto::{PemPublicKey, SymmetricKey};
use crate::db::{
    ActorType, ActorTypeList, Db, Grant, Inbox, Iri, Key, Name, OptionalIri, Outbox, TableEntry,
    Uuid, UuidList,
};
use crate::{
    ActorType as VocabActorType, Error, Factory as VocabFactory, Result, Role, impl_sql_actor,
    impl_sql_list_field, util,
};

mod builder;

pub use builder::*;

/// Represents a [Factory Actor](https://forgefed.org/spec/#factory) database record.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Factory {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    available_actor_types: Vec<ActorType>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    collaborators_id: Option<Iri>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    collaborators: Vec<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    followers_id: Option<Iri>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    followers: Vec<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    teams_id: Option<Iri>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    teams: Vec<Uuid>,
}

impl Factory {
    /// Creates a new [Factory].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            name: Name::new(),
            available_actor_types: Vec::new(),
            inbox: Uuid::nil(),
            outbox: Uuid::nil(),
            key_ids: Vec::new(),
            collaborators_id: None,
            collaborators: Vec::new(),
            followers_id: None,
            followers: Vec::new(),
            teams_id: None,
            teams: Vec::new(),
        }
    }

    /// Creates a new [FactoryBuilder] to create a [Factory].
    pub fn builder(id: Iri, name: Name) -> Result<FactoryBuilder> {
        FactoryBuilder::new(id, name)
    }

    /// Performs checks on record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.id.is_empty() {
            Err(Error::sql("factory: ID is empty"))
        } else if self.name.is_empty() {
            Err(Error::sql("factory: name is empty"))
        } else if self.inbox.is_nil() {
            Err(Error::sql("factory: inbox is nil"))
        } else if self.outbox.is_nil() {
            Err(Error::sql("factory: outbox is nil"))
        } else {
            Ok(())
        }
    }

    /// Converts a [Factory](VocabFactory) JSON-LD record into a database [Factory] record.
    ///
    /// Useful for creating records fetched from remote instances.
    ///
    /// Creates a random UUID for local storage.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab(db: &Db, val: &VocabFactory) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let factory = Self::try_from_vocab_tx(&mut dbtx, &db_key, val).await?;

        dbtx.commit()
            .await
            .map(|_| factory)
            .map_err(|err| Error::db(format!("factory: {err}")))
    }

    /// Converts a [Factory](VocabFactory) JSON-LD record into a database [Factory] record using a transaction.
    ///
    /// Useful for creating records fetched from remote instances.
    ///
    /// Creates a random UUID for local storage.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab_tx(
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
        val: &VocabFactory,
    ) -> Result<Self> {
        let uuid = val
            .id()
            .ok_or(Error::db("factory: missing ID"))
            .map(|id| Self::TABLE.uuid_from_id(id))?;

        Self::try_from_vocab_with_uuid_tx(dbtx, db_key, val, uuid).await
    }

    /// Converts a [Factory](VocabFactory) JSON-LD record into a database [Factory] record.
    ///
    /// Useful for creating local records, e.g. parsed from a `Create` activity.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab_with_uuid(db: &Db, val: &VocabFactory, uuid: Uuid) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let factory = Self::try_from_vocab_with_uuid_tx(&mut dbtx, &db_key, val, uuid).await?;

        dbtx.commit()
            .await
            .map(|_| factory)
            .map_err(|err| Error::db(format!("factory: {err}")))
    }

    /// Converts a [Factory](VocabFactory) JSON-LD record into a database [Factory] record using a transaction.
    ///
    /// Useful for creating local records, e.g. parsed from a `Create` activity.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    #[allow(deprecated)]
    pub async fn try_from_vocab_with_uuid_tx(
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
        val: &VocabFactory,
        uuid: Uuid,
    ) -> Result<Self> {
        let actor = TableEntry::create(Self::TABLE, uuid);

        let id: Iri = val
            .id()
            .map(|id| id.into())
            .ok_or(Error::db("factory: missing id"))?;

        let name: Name = val
            .name()
            .map(|name| name.into())
            .ok_or(Error::db("factory: missing name"))?;

        let inbox_id = val
            .inbox()
            .map(|i| i.into())
            .ok_or(Error::db("factory: missing inbox"))
            .or_else(|_| Iri::try_from(format!("{id}/inbox")))?;

        let mut inbox = Inbox::new().with_id(inbox_id).with_actor(actor);

        inbox.find_or_create_tx(dbtx).await?;

        let mailbox_roles = [Role::Visit, Role::Write];
        let inbox_grant_id = Grant::TABLE.id_from_uuid(&id, util::rand_uuid())?;

        let mut inbox_grant = Grant::new()
            .with_id(inbox_grant_id)
            .with_actor(actor)
            .with_context(inbox.table_entry())
            .with_objects(mailbox_roles)?;

        inbox_grant.find_or_create_tx(dbtx).await?;

        let outbox_id = val
            .outbox()
            .map(|i| i.into())
            .ok_or(Error::db("factory: missing outbox"))
            .or_else(|_| Iri::try_from(format!("{id}/outbox")))?;

        let mut outbox = Outbox::new().with_id(outbox_id).with_actor(actor);

        outbox.find_or_create_tx(dbtx).await?;

        let outbox_grant_id = Grant::TABLE.id_from_uuid(&id, util::rand_uuid())?;

        let mut outbox_grant = Grant::new()
            .with_id(outbox_grant_id)
            .with_actor(actor)
            .with_context(outbox.table_entry())
            .with_objects(mailbox_roles)?;

        outbox_grant.find_or_create_tx(dbtx).await?;

        let actor_types = val
            .available_actor_types()
            .and_then(|i| i.clone().into_list().ok())
            .unwrap_or_default();

        let followers_id: Option<Iri> = val.followers().map(|v| v.into());

        let collaborators_id: Option<Iri> = val.collaborators().and_then(|v| match v {
            LinkItems::Single(i) => Some(i.into()),
            LinkItems::List(_) => None,
        });

        let teams_id: Option<Iri> = val.teams().and_then(|v| match v {
            LinkItems::Single(i) => Some(i.into()),
            LinkItems::List(_) => None,
        });

        let mut keys: Vec<Key> = Vec::new();

        if let Some(pem) = val.public_key() {
            match pem {
                KeyItems::Single(KeyItem::Key(key)) => PemPublicKey::try_from(key)
                    .and_then(|k| k.with_owner(&id).try_into())
                    .map(|k| keys.push(k))?,
                KeyItems::List(list) => {
                    for key in list.iter() {
                        if let KeyItem::Key(k) = key {
                            PemPublicKey::try_from(k)
                                .and_then(|k| k.with_owner(&id).try_into())
                                .map(|k| keys.push(k))?;
                        }
                    }
                }
                _ => (),
            }
        }

        if let Some(multikey) = val.assertion_method() {
            match multikey {
                MultikeyItems::Single(MultikeyItem::Multikey(key)) => key
                    .clone()
                    .with_controller(&id)
                    .try_into()
                    .map(|k| keys.push(k))?,
                MultikeyItems::List(list) => {
                    for key in list.iter() {
                        if let MultikeyItem::Multikey(k) = key {
                            k.clone()
                                .with_controller(&id)
                                .try_into()
                                .map(|k| keys.push(k))?;
                        }
                    }
                }
                _ => (),
            }
        }

        keys.sort();
        keys.dedup();

        for key in keys.iter_mut() {
            key.set_actor(actor);
            key.set_actor_id(&id);
            key.insert_or_update_tx(dbtx, db_key).await?;
        }

        let key_ids = keys.iter().map(|k| k.uuid()).collect::<Vec<_>>();

        let mut factory = Self {
            uuid,
            id,
            name,
            inbox: inbox.uuid(),
            outbox: outbox.uuid(),
            available_actor_types: actor_types.into_iter().map(|i| i.into()).collect(),
            followers_id,
            followers: Vec::new(),
            collaborators_id,
            collaborators: Vec::new(),
            teams_id,
            teams: Vec::new(),
            key_ids,
        };

        factory.find_or_create_tx(dbtx).await.map(|_| factory)
    }

    /// Attempts to convert a [Factory] database record into a JSON-LD [Factory](VocabFactory).
    pub async fn try_into_vocab(&self, db: &Db) -> Result<VocabFactory> {
        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let factory = self.try_into_vocab_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| factory)
            .map_err(|err| Error::db(format!("factory: {err}")))
    }

    /// Attempts to convert a [Factory] database record into a JSON-LD [Factory](VocabFactory) using a transaction.
    #[allow(deprecated)]
    pub async fn try_into_vocab_tx(
        &self,
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<VocabFactory> {
        let id = VocabIri::from(self.id());
        let name = VocabName::from(self.name());

        let inbox = Inbox::get_tx(dbtx, &self.inbox())
            .await
            .map(|i| VocabIri::from(i.id()))?;

        let outbox = Outbox::get_tx(dbtx, &self.outbox())
            .await
            .map(|i| VocabIri::from(i.id()))?;

        let keys = self.keys_tx(dbtx, db_key).await?;

        let mut assertion_method = Vec::with_capacity(keys.len());
        let mut public_key = Vec::with_capacity(keys.len());

        for key in keys.iter() {
            if let Ok(multikey) = Multikey::try_from(key) {
                assertion_method.push(multikey);
            }

            if let Ok(pemkey) = VocabPublicKey::try_from(key) {
                public_key.push(pemkey);
            }
        }

        let available_actor_types = self
            .available_actor_types()
            .iter()
            .filter_map(|i| VocabActorType::try_from(i).ok())
            .collect::<Vec<_>>();

        let mut factory = VocabFactory::new()
            .with_id(id)
            .with_name(name)
            .with_inbox(inbox)
            .with_outbox(outbox)
            .with_available_actor_types(available_actor_types);

        if let Some(followers_id) = self.followers_id() {
            factory.set_followers(VocabIri::from(followers_id));
        }

        if let Some(collaborators_id) = self.collaborators_id() {
            factory.set_collaborators(VocabIri::from(collaborators_id));
        }

        if let Some(teams_id) = self.teams_id() {
            factory.set_teams(VocabIri::from(teams_id));
        }

        if assertion_method.len() > 1 {
            factory.set_assertion_method(assertion_method);
        } else if !assertion_method.is_empty() {
            assertion_method
                .into_iter()
                .next()
                .ok_or(Error::db("factory: missing multikey info"))
                .map(|v| factory.set_assertion_method(v))?;
        }

        if public_key.len() > 1 {
            factory.set_public_key(public_key);
        } else if !public_key.is_empty() {
            public_key
                .into_iter()
                .next()
                .ok_or(Error::db("factory: missing PEM public key info"))
                .map(|v| factory.set_public_key(v))?;
        }

        Ok(factory)
    }
}

field_access! {
    Factory {
        /// Main [Uuid] primary key for the record.
        uuid: Uuid,
        /// Represents the [Uuid] of the [Inbox](crate::db::Inbox) record.
        inbox: Uuid,
        /// Represents the [Uuid] of the [Outbox](crate::db::Outbox) record.
        outbox: Uuid,
    }
}

field_access! {
    Factory {
        /// ActivityPub [Iri] for external actors to reference the [Factory].
        id: as_ref { Iri },
        /// Represents the human-readable name of the [Factory].
        name: as_ref { Name },
    }
}

field_access! {
    Factory {
        /// Optional IRI for downloading the list of collaborators.
        collaborators_id: option_ref { Iri },
        /// Optional IRI for downloading the list of followers.
        followers_id: option_ref { Iri },
        /// Optional IRI for downloading the list of teams.
        teams_id: option_ref { Iri },
    }
}

impl_default!(Factory);
impl_display!(Factory, json);

impl_sql_actor! {
    Factory {
        id: { "id" Iri },
        name: { "name" Name },
        inbox: { "inbox" Uuid },
        outbox: { "outbox" Uuid },
        available_actor_types: { "available_actor_types" ActorTypeList },
        key_ids: { "key_ids" UuidList },
        collaborators_id: { "collaborators_id" OptionalIri },
        collaborators: { "collaborators" UuidList },
        followers_id: { "followers_id" OptionalIri },
        followers: { "followers" UuidList },
        teams_id: { "teams_id" OptionalIri },
        teams: { "teams" UuidList },
    }
}

impl_sql_list_field! {
    Factory {
        /// List of available [ActorType](crate::db::ActorType)s the [Factory] can create.
        available_actor_type, available_actor_types: { "available_actor_types" ActorType },
        /// Represents a list of references to the [Key](crate::db::Key) records.
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
        key_id, key_ids: { "key_ids" Uuid },
        /// List of references to the [Follower](crate::db::Follower) records.
        follower, followers: { "followers" Uuid },
        /// List of references to the [Collaborator](crate::db::Collaborator) records.
        collaborator, collaborators: { "collaborators" Uuid },
        /// List of references to the [Factory]'s [Team](crate::db::Team) records.
        team, teams: { "teams" Uuid },
    }
}
