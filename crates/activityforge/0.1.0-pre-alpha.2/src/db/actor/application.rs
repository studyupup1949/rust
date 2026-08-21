use activitystreams_vocabulary::{
    Application as VocabApplication, Iri as VocabIri, Key as VocabPublicKey, KeyItem, KeyItems,
    Multikey, MultikeyItem, MultikeyItems, Name as VocabName, field_access, impl_default,
    impl_display,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::app::oauth::{Scope, ScopeList};
use crate::crypto::{Password, PemPublicKey, SymmetricKey};
use crate::db::{
    Db, Grant, Inbox, Iri, Key, Name, OptionalIri, OptionalPassword, OptionalString, Outbox,
    TableEntry, Uuid, UuidList,
};
use crate::{Error, Result, Role, impl_sql_actor, impl_sql_list_field, util};

mod builder;

pub use builder::*;

/// Represents a [Application Actor](https://forgefed.org/spec/#application) database record.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
#[sqlx(type_name = "application")]
pub struct Application {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    #[serde(skip)]
    password: Option<Password>,
    #[serde(skip_serializing_if = "ScopeList::is_empty")]
    scopes: ScopeList,
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

impl Application {
    /// Creates a new [Application].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            name: Name::new(),
            password: None,
            scopes: ScopeList::new(),
            inbox: Uuid::nil(),
            outbox: Uuid::nil(),
            summary: None,
            content: None,
            key_ids: Vec::new(),
            followers_id: None,
            followers: Vec::new(),
        }
    }

    /// Creates a new [ApplicationBuilder] to create a [Application].
    pub fn builder<I: Into<Iri>, N: Into<Name>>(id: I, name: N) -> Result<ApplicationBuilder> {
        ApplicationBuilder::new(id, name)
    }

    /// Performs checks on record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.id.is_empty() {
            Err(Error::sql("application: empty ID"))
        } else if self.name.as_str().is_empty() {
            Err(Error::sql("application: empty name"))
        } else if self.inbox.is_nil() {
            Err(Error::sql("application: nil inbox"))
        } else if self.outbox.is_nil() {
            Err(Error::sql("application: nil outbox"))
        } else {
            Ok(())
        }
    }

    /// Converts a [Application](VocabApplication) JSON-LD record into a database [Application] record.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab(db: &Db, val: &VocabApplication) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let app = Self::try_from_vocab_tx(&mut dbtx, &db_key, val).await?;

        dbtx.commit()
            .await
            .map(|_| app)
            .map_err(|err| Error::db(format!("application: {err}")))
    }

    /// Converts a [Application](VocabApplication) JSON-LD record into a database [Application] record.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab_tx(
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
        val: &VocabApplication,
    ) -> Result<Self> {
        let uuid = val
            .id()
            .ok_or(Error::db("application: missing ID"))
            .map(|id| Self::TABLE.uuid_from_id(id))?;

        Self::try_from_vocab_with_uuid_tx(dbtx, db_key, val, uuid).await
    }

    /// Converts a [Application](VocabApplication) JSON-LD record into a database [Application] record.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab_with_uuid(
        db: &Db,
        val: &VocabApplication,
        uuid: Uuid,
    ) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let app = Self::try_from_vocab_with_uuid_tx(&mut dbtx, &db_key, val, uuid).await?;

        dbtx.commit()
            .await
            .map(|_| app)
            .map_err(|err| Error::db(format!("application: {err}")))
    }

    /// Converts a [Application](VocabApplication) JSON-LD record into a database [Application] record using a transaction.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    #[allow(deprecated)]
    pub async fn try_from_vocab_with_uuid_tx(
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
        val: &VocabApplication,
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

        let followers_id: Option<Iri> = val.followers().map(|v| v.into());

        let mut keys: Vec<Key> = Vec::new();

        if let Some(pem) = val.public_key() {
            match pem {
                KeyItems::Single(KeyItem::Key(key)) => PemPublicKey::try_from(key)
                    .and_then(|k| k.try_into())
                    .map(|k| keys.push(k))?,
                KeyItems::List(list) => {
                    for key in list.iter() {
                        if let KeyItem::Key(k) = key {
                            PemPublicKey::try_from(k)
                                .and_then(|k| k.try_into())
                                .map(|k| keys.push(k))?;
                        }
                    }
                }
                _ => (),
            }
        }

        if let Some(multikey) = val.assertion_method() {
            match multikey {
                MultikeyItems::Single(MultikeyItem::Multikey(key)) => {
                    key.try_into().map(|k| keys.push(k))?
                }
                MultikeyItems::List(list) => {
                    for key in list.iter() {
                        if let MultikeyItem::Multikey(k) = key {
                            k.try_into().map(|k| keys.push(k))?;
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

        let mut application = Self {
            uuid,
            id,
            name,
            password: None,
            scopes: ScopeList::new(),
            summary: val.summary().map(|v| v.to_string()),
            content: val.content().map(|v| v.to_string()),
            inbox: inbox.uuid(),
            outbox: outbox.uuid(),
            followers_id,
            followers: Vec::new(),
            key_ids,
        };

        application
            .find_or_create_tx(dbtx)
            .await
            .map(|_| application)
    }

    /// Tries to convert an [Application] record into an [Application](VocabApplication) JSON-LD object.
    pub async fn try_into_vocab(&self, db: &Db) -> Result<VocabApplication> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let app = self.try_into_vocab_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| app)
            .map_err(|err| Error::db(format!("application: {err}")))
    }

    /// Tries to convert an [Application] record into an [Application](VocabApplication) JSON-LD object using a transaction.
    #[allow(deprecated)]
    pub async fn try_into_vocab_tx(
        &self,
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<VocabApplication> {
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

        let mut application = VocabApplication::new()
            .with_id(id)
            .with_name(name)
            .with_inbox(inbox)
            .with_outbox(outbox);

        if let Some(summary) = self.summary() {
            application.set_summary(summary);
        }

        if let Some(content) = self.content() {
            application.set_content(content);
        }

        if let Some(followers) = self.followers_id() {
            application.set_followers(VocabIri::from(followers));
        }

        if assertion_method.len() > 1 {
            application.set_assertion_method(assertion_method);
        } else if !assertion_method.is_empty() {
            assertion_method
                .into_iter()
                .next()
                .ok_or(Error::db("application: missing multikey info"))
                .map(|v| application.set_assertion_method(v))?;
        }

        if public_key.len() > 1 {
            application.set_public_key(public_key);
        } else if !public_key.is_empty() {
            public_key
                .into_iter()
                .next()
                .ok_or(Error::db("application: missing PEM public key info"))
                .map(|v| application.set_public_key(v))?;
        }

        Ok(application)
    }
}

field_access! {
    Application {
        /// Represents the [Uuid] primary key of the table entry.
        uuid: Uuid,
        /// Represents the [Uuid] of the [Inbox](crate::db::Inbox) record.
        inbox: Uuid,
        /// Represents the [Uuid] of the [Outbox](crate::db::Outbox) record.
        outbox: Uuid,
    }
}

field_access! {
    Application {
        /// Represents the IRI used to fetch the [Application] record.
        id: as_ref { Iri },
        /// Represents the human-readable [Application] name.
        name: as_ref { Name },
    }
}

field_access! {
    Application {
        /// Represents the [Application]'s content description.
        content: option_deref { &str, String },
        /// Represents the [Application]'s summary.
        summary: option_deref { &str, String },
    }
}
field_access! {
    Application {
        /// Represents an IRI referencing the followers list.
        followers_id: option_ref { Iri },
        /// Represents the [Person] password hash (local-accounts only).
        password: option_ref { Password },
    }
}

impl_sql_actor! {
    Application {
        id: { "id" Iri },
        name: { "name" Name },
        password: { "password" OptionalPassword },
        scopes: { "scopes" ScopeList },
        summary: { "summary" OptionalString },
        content: { "content" OptionalString },
        inbox: { "inbox" Uuid },
        outbox: { "outbox" Uuid },
        key_ids: { "key_ids" UuidList },
        followers_id: { "followers_id" OptionalIri },
        followers: { "followers" UuidList },
    }
}

impl_sql_list_field! {
    Application {
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
        /// List of [Scope] OAuth-2.0 grants given to the [Person](crate::db::Person).
        scope, scopes: { "scopes" Scope },
    }
}

impl_default!(Application);
impl_display!(Application, json);
