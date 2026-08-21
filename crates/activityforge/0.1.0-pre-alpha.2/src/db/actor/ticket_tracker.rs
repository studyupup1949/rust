use activitystreams_vocabulary::{
    Iri as VocabIri, Key as VocabPublicKey, KeyItem, KeyItems, Multikey, MultikeyItem,
    MultikeyItems, Name as VocabName, field_access, impl_default, impl_display,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::crypto::{PemPublicKey, SymmetricKey};
use crate::db::{
    Db, Grant, Inbox, Iri, Key, Name, OptionalString, Outbox, TableEntry, Transaction, Uuid,
    UuidList,
};
use crate::{
    Error, Result, Role, TicketTracker as VocabTicketTracker, impl_sql_actor, impl_sql_list_field,
    util,
};

mod builder;

pub use builder::*;

/// Represents a [TicketTracker Actor](https://forgefed.org/spec/#tickettracker) database record.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct TicketTracker {
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
    #[serde(skip_serializing_if = "Option::is_none")]
    followers_id: Option<Iri>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    followers: Vec<Uuid>,
}

impl TicketTracker {
    /// Creates a new [TicketTracker].
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
            followers_id: None,
            followers: Vec::new(),
        }
    }

    /// Creates a new [TicketTrackerBuilder] to create a [TicketTracker].
    pub fn builder<I: Into<Iri>, N: Into<Name>>(id: I, name: N) -> Result<TicketTrackerBuilder> {
        TicketTrackerBuilder::new(id, name)
    }

    /// Performs checks on record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.id.is_empty() {
            Err(Error::sql("ticket_tracker: empty ID"))
        } else if self.name.as_str().is_empty() {
            Err(Error::sql("ticket_tracker: empty name"))
        } else if self.inbox.is_nil() {
            Err(Error::sql("ticket_tracker: nil inbox"))
        } else if self.outbox.is_nil() {
            Err(Error::sql("ticket_tracker: nil outbox"))
        } else {
            Ok(())
        }
    }

    /// Converts a [TicketTracker](VocabTicketTracker) JSON-LD record into a database [TicketTracker] record.
    ///
    /// Useful for creating records fetched from remote instances.
    ///
    /// Creates a random UUID for local storage.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab(db: &Db, val: &VocabTicketTracker) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let ticket_tracker = Self::try_from_vocab_tx(&mut dbtx, &db_key, val).await?;

        dbtx.commit()
            .await
            .map(|_| ticket_tracker)
            .map_err(|err| Error::db(format!("ticket_tracker: {err}")))
    }

    /// Converts a [TicketTracker](VocabTicketTracker) JSON-LD record into a database [TicketTracker] record using a transaction.
    ///
    /// Useful for creating records fetched from remote instances.
    ///
    /// Creates a random UUID for local storage.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        val: &VocabTicketTracker,
    ) -> Result<Self> {
        let uuid = val
            .id()
            .ok_or(Error::db("ticket_tracker: missing ID"))
            .map(|id| Self::TABLE.uuid_from_id(id))?;

        Self::try_from_vocab_with_uuid_tx(dbtx, db_key, val, uuid).await
    }

    /// Converts a [TicketTracker](VocabTicketTracker) JSON-LD record into a database [TicketTracker] record.
    ///
    /// Useful for creating local records, e.g. parsed from a `Create` activity.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    #[allow(deprecated)]
    pub async fn try_from_vocab_with_uuid(
        db: &Db,
        val: &VocabTicketTracker,
        uuid: Uuid,
    ) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let ticket_tracker =
            Self::try_from_vocab_with_uuid_tx(&mut dbtx, &db_key, val, uuid).await?;

        dbtx.commit()
            .await
            .map(|_| ticket_tracker)
            .map_err(|err| Error::db(format!("ticket_tracker: {err}")))
    }

    /// Converts a [TicketTracker](VocabTicketTracker) JSON-LD record into a database [TicketTracker] record.
    ///
    /// Useful for creating local records, e.g. parsed from a `Create` activity.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    #[allow(deprecated)]
    pub async fn try_from_vocab_with_uuid_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        val: &VocabTicketTracker,
        uuid: Uuid,
    ) -> Result<Self> {
        let actor = TableEntry::create(Self::TABLE, uuid);

        let id: Iri = val
            .id()
            .map(|id| id.into())
            .ok_or(Error::db("ticket_tracker: missing id"))?;

        let name: Name = val
            .name()
            .map(|name| name.into())
            .ok_or(Error::db("ticket_tracker: missing name"))?;

        let inbox_id = val
            .inbox()
            .map(|i| i.into())
            .ok_or(Error::db("ticket_tracker: missing inbox"))
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
            .ok_or(Error::db("ticket_tracker: missing outbox"))
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

        let followers_id: Option<Iri> = val.followers().map(|v| v.into());

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
            key.find_or_create_tx(dbtx, db_key).await?;
        }

        let key_ids = keys.iter().map(|k| k.uuid()).collect::<Vec<_>>();

        log::debug!("ticket_tracker: commiting storage for ID: {id}");

        let mut ticket_tracker = Self {
            uuid,
            id,
            name,
            summary: val.summary().map(|v| v.to_string()),
            content: val.content().map(|v| v.to_string()),
            inbox: inbox.uuid(),
            outbox: outbox.uuid(),
            followers_id,
            followers: Vec::new(),
            key_ids,
        };

        ticket_tracker
            .find_or_create_tx(dbtx)
            .await
            .map(|_| ticket_tracker)
    }

    /// Attempts to convert a [TicketTracker] database record into a JSON-LD [TicketTracker](VocabTicketTracker).
    pub async fn try_into_vocab(&self, db: &Db) -> Result<VocabTicketTracker> {
        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let tracker = self.try_into_vocab_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| tracker)
            .map_err(|err| Error::db(format!("ticket_tracker: {err}")))
    }

    /// Attempts to convert a [TicketTracker] database record into a JSON-LD [TicketTracker](VocabTicketTracker) using a transaction.
    #[allow(deprecated)]
    pub async fn try_into_vocab_tx(
        &self,
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<VocabTicketTracker> {
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

        let mut tracker = VocabTicketTracker::new()
            .with_id(id)
            .with_name(name)
            .with_inbox(inbox)
            .with_outbox(outbox);

        if let Some(followers_id) = self.followers_id() {
            tracker.set_followers(VocabIri::from(followers_id));
        }

        if assertion_method.len() > 1 {
            tracker.set_assertion_method(assertion_method);
        } else if !assertion_method.is_empty() {
            assertion_method
                .into_iter()
                .next()
                .ok_or(Error::db("ticket_tracker: missing multikey info"))
                .map(|v| tracker.set_assertion_method(v))?;
        }

        if public_key.len() > 1 {
            tracker.set_public_key(public_key);
        } else if !public_key.is_empty() {
            public_key
                .into_iter()
                .next()
                .ok_or(Error::db("ticket_tracker: missing PEM public key info"))
                .map(|v| tracker.set_public_key(v))?;
        }

        Ok(tracker)
    }
}

field_access! {
    TicketTracker {
        /// Represents the [Uuid] primary key of the table entry.
        uuid: Uuid,
        /// Represents the [Uuid] of the [Inbox](crate::db::Inbox) record.
        inbox: Uuid,
        /// Represents the [Uuid] of the [Outbox](crate::db::Outbox) record.
        outbox: Uuid,
    }
}

field_access! {
    TicketTracker {
        /// Represents the IRI used to fetch the [TicketTracker] record.
        id: as_ref { Iri },
        /// Represents the human-readable [TicketTracker] name.
        name: as_ref { Name },
    }
}

field_access! {
    TicketTracker {
        /// Represents the [TicketTracker]'s content description.
        content: option_deref { &str, String },
        /// Represents the [TicketTracker]'s summary.
        summary: option_deref { &str, String },
    }
}

field_access! {
    TicketTracker {
        /// Represents the IRI used to fetch the [TicketTracker] follower records.
        followers_id: option_ref { Iri },
    }
}

impl_sql_actor! {
    TicketTracker {
        id: { "id" Iri },
        name: { "name" Name },
        summary: { "summary" OptionalString },
        content: { "content" OptionalString },
        inbox: { "inbox" Uuid },
        outbox: { "outbox" Uuid },
        key_ids: { "key_ids" UuidList },
        followers: { "followers" UuidList },
    }
}

impl_sql_list_field! {
    TicketTracker {
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
    }
}

impl_default!(TicketTracker);
impl_display!(TicketTracker, json);
