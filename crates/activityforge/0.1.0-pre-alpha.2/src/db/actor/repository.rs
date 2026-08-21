use activitystreams_vocabulary::{
    CollectionItem, Iri as VocabIri, Iris, Item, Items, Key as VocabPublicKey, KeyItem, KeyItems,
    LinkItem, Multikey, MultikeyItem, MultikeyItems, OrderedCollectionItem, OrderedItems,
    field_access, impl_default, impl_display,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::crypto::{PemPublicKey, SymmetricKey};
use crate::db::{
    Db, Grant, Inbox, Iri, IriList, Key, Like, Name, OptionalBool, OptionalIri, Outbox, TableEntry,
    Transaction, Uuid, UuidList,
};
use crate::{
    Error, Like as VocabLike, PatchTrackerItem, Repository as VocabRepository, RepositoryItem,
    Result, Role, TeamItem, TicketTrackerItem, impl_sql_actor, impl_sql_list_field, util,
};

mod builder;

pub use builder::*;

/// Represents a [Repository Actor](https://forgefed.org/spec/#repository) database record.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Repository {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    inbox: Uuid,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    outbox: Uuid,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    clone_uris: Vec<Iri>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    push_uris: Vec<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    forks_id: Option<Iri>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    forks: Vec<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    likes_id: Option<Iri>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    likes: Vec<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    followers_id: Option<Iri>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    followers: Vec<Iri>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    key_ids: Vec<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    send_patches_to: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    tickets_tracked_by: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_archived: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    moved_to: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    mirrors: Option<Iri>,
    #[serde(skip_serializing_if = "Option::is_none")]
    team: Option<Iri>,
    is_private: bool,
}

impl Repository {
    /// Creates a new [Repository].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            name: Name::new(),
            inbox: Uuid::nil(),
            outbox: Uuid::nil(),
            clone_uris: Vec::new(),
            push_uris: Vec::new(),
            forks_id: None,
            forks: Vec::new(),
            likes_id: None,
            likes: Vec::new(),
            followers_id: None,
            followers: Vec::new(),
            key_ids: Vec::new(),
            send_patches_to: None,
            tickets_tracked_by: None,
            is_archived: None,
            moved_to: None,
            mirrors: None,
            team: None,
            is_private: false,
        }
    }

    /// Creates a new [RepositoryBuilder] to create a [Repository].
    pub fn builder<I: Into<Iri>, N: Into<Name>>(id: I, name: N) -> Result<RepositoryBuilder> {
        RepositoryBuilder::new(id, name)
    }

    /// Performs checks for record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.id.is_empty() {
            Err(Error::sql("repository: empty ID"))
        } else if self.name.is_empty() {
            Err(Error::sql("repository: empty name"))
        } else if self.inbox.is_nil() {
            Err(Error::sql("repository: nil inbox"))
        } else if self.outbox.is_nil() {
            Err(Error::sql("repository: nil outbox"))
        } else {
            Ok(())
        }
    }

    /// Converts an [Repository](VocabRepository) JSON-LD object into an [Repository] record.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab(db: &Db, val: &VocabRepository) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let actor = Self::try_from_vocab_tx(&mut dbtx, &db_key, val).await?;

        dbtx.commit()
            .await
            .map(|_| actor)
            .map_err(|err| Error::db(format!("actor: {err}")))
    }

    /// Converts a [Repository](VocabRepository) JSON-LD object into an [Repository] record using a transaction.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        val: &VocabRepository,
    ) -> Result<Self> {
        let uuid = val
            .id()
            .ok_or(Error::db("repository: missing ID"))
            .map(|id| Self::TABLE.uuid_from_id(id))?;

        Self::try_from_vocab_with_uuid_tx(dbtx, db_key, val, uuid).await
    }

    /// Converts a [Repository](VocabRepository) JSON-LD record into a database [Repository] record.
    ///
    /// Useful for creating local records, e.g. parsed from a `Create` activity.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab_with_uuid(
        db: &Db,
        val: &VocabRepository,
        uuid: Uuid,
    ) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let repository = Self::try_from_vocab_with_uuid_tx(&mut dbtx, &db_key, val, uuid).await?;

        dbtx.commit()
            .await
            .map(|_| repository)
            .map_err(|err| Error::db(format!("repository: {err}")))
    }

    /// Converts a [Repository](VocabRepository) JSON-LD record into a database [Repository] record.
    ///
    /// Useful for creating local records, e.g. parsed from a `Create` activity.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    #[allow(deprecated)]
    pub async fn try_from_vocab_with_uuid_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        val: &VocabRepository,
        uuid: Uuid,
    ) -> Result<Self> {
        let actor = TableEntry::create(Self::TABLE, uuid);

        let id: Iri = val
            .id()
            .map(|id| id.into())
            .ok_or(Error::db("repository: missing id"))?;

        let name: Name = val
            .name()
            .map(|name| name.into())
            .ok_or(Error::db("repository: missing name"))?;

        let inbox_id = val
            .inbox()
            .map(|i| i.into())
            .ok_or(Error::db("repository: missing inbox"))
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
            .ok_or(Error::db("repository: missing outbox"))
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

        log::debug!("repository: commiting storage for ID: {id}");

        let clone_uris = match val.clone_uri() {
            Some(Iris::Single(iri)) => vec![Iri::from(iri)],
            Some(Iris::List(list)) => list.iter().map(Iri::from).collect(),
            None => Vec::new(),
        };

        let push_uris = match val.push_uri() {
            Some(Iris::Single(iri)) => vec![Iri::from(iri)],
            Some(Iris::List(list)) => list.iter().map(Iri::from).collect(),
            None => Vec::new(),
        };

        let mut repository = Self {
            uuid,
            id: id.clone(),
            name,
            inbox: inbox.uuid(),
            outbox: outbox.uuid(),
            clone_uris,
            push_uris,
            forks_id: None,
            forks: Vec::new(),
            followers_id,
            followers: Vec::new(),
            likes_id: None,
            likes: Vec::new(),
            key_ids,
            send_patches_to: None,
            tickets_tracked_by: None,
            is_archived: val.is_archived(),
            moved_to: None,
            mirrors: None,
            team: None,
            is_private: false,
        };

        if let Some(forks_collection_item) = val.forks() {
            match forks_collection_item {
                OrderedCollectionItem::Iri(iri) => repository.set_forks_id(iri.as_ref()),
                OrderedCollectionItem::Link(link) => repository.set_forks_id(link.href()),
                OrderedCollectionItem::OrderedCollection(forks_collection) => {
                    match forks_collection.ordered_items() {
                        Some(OrderedItems::Single(item)) => {
                            let fork_id = match item {
                                Item::Iri(iri) => iri.as_ref(),
                                Item::Link(link) => link.href(),
                                Item::Object(object) => object
                                    .id()
                                    .ok_or(Error::db("repository: missing fork ID"))?,
                            };
                            repository.set_forks([Iri::from(fork_id)])?;
                        }
                        Some(OrderedItems::List(list)) => {
                            let fork_ids = list
                                .iter()
                                .filter_map(|item| match item {
                                    Item::Iri(iri) => Some(Iri::from(iri.as_ref())),
                                    Item::Link(link) => Some(Iri::from(link.href())),
                                    Item::Object(object) => object.id().map(Iri::from),
                                })
                                .collect::<Vec<_>>();

                            if !fork_ids.is_empty() {
                                repository.set_forks(fork_ids)?;
                            }
                        }
                        None => (),
                    }
                }
            }
        }

        if let Some(likes_collection_item) = val.likes() {
            match likes_collection_item {
                CollectionItem::Iri(iri) => repository.set_likes_id(iri.as_ref()),
                CollectionItem::Link(link) => repository.set_likes_id(link.href()),
                CollectionItem::Collection(likes_collection) => match likes_collection.items() {
                    Some(Items::Single(Item::Object(object))) => {
                        let like = VocabLike::try_from(object.as_ref())?;
                        Like::try_from_vocab_tx(dbtx, &id, &like)
                            .await
                            .and_then(|l| repository.set_likes(vec![l.uuid()]))?;
                    }
                    Some(Items::List(list)) => {
                        let mut like_uuids = Vec::new();

                        for item in list.iter().filter(|o| o.is_object()) {
                            if let Ok(object) = item.as_object() {
                                let like = VocabLike::try_from(object)?;
                                Like::try_from_vocab_tx(dbtx, &id, &like)
                                    .await
                                    .map(|l| like_uuids.push(l.uuid()))?;
                            }
                        }

                        if !like_uuids.is_empty() {
                            repository.set_likes(like_uuids)?;
                        }
                    }
                    _ => (),
                },
            }
        }

        if let Some(send_patches_to_item) = val.send_patches_to() {
            match send_patches_to_item {
                PatchTrackerItem::PatchTracker(actor) => repository.set_send_patches_to(actor.id()),
                PatchTrackerItem::Iri(iri) => repository.set_send_patches_to(iri.as_ref()),
            }
        }

        if let Some(send_patches_to_item) = val.send_patches_to() {
            match send_patches_to_item {
                PatchTrackerItem::PatchTracker(actor) => repository.set_send_patches_to(actor.id()),
                PatchTrackerItem::Iri(iri) => repository.set_send_patches_to(iri.as_ref()),
            }
        }

        if let Some(tickets_tracked_by_item) = val.tickets_tracked_by() {
            match tickets_tracked_by_item {
                TicketTrackerItem::TicketTracker(actor) => {
                    repository.set_tickets_tracked_by(actor.id())
                }
                TicketTrackerItem::Iri(iri) => repository.set_tickets_tracked_by(iri.as_ref()),
            }
        }

        if let Some(moved_to_item) = val.moved_to() {
            match moved_to_item {
                LinkItem::Iri(iri) => repository.set_moved_to(iri.as_ref()),
                LinkItem::Link(link) => repository.set_moved_to(link.href()),
            }
        }

        if let Some(mirrors_item) = val.mirrors() {
            match mirrors_item {
                RepositoryItem::Repository(actor) => repository.set_mirrors(actor.id()),
                RepositoryItem::Iri(iri) => repository.set_mirrors(iri.as_ref()),
            }
        }

        if let Some(team_item) = val.team() {
            match team_item {
                TeamItem::Team(actor) => repository.set_team(actor.id()),
                TeamItem::Iri(iri) => repository.set_team(iri.as_ref()),
            }
        }

        repository.find_or_create_tx(dbtx).await.map(|_| repository)
    }

    /// Tries to convert an [Repository] record into an [Repository](VocabRepository) JSON-LD object.
    pub async fn try_into_vocab(&self, db: &Db) -> Result<VocabRepository> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let app = self.try_into_vocab_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| app)
            .map_err(|err| Error::db(format!("repository: {err}")))
    }

    /// Tries to convert an [Repository] record into an [Repository](VocabRepository) JSON-LD object using a transaction.
    #[allow(deprecated)]
    pub async fn try_into_vocab_tx(
        &self,
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<VocabRepository> {
        let inbox = Inbox::get_tx(dbtx, &self.inbox()).await?;
        let outbox = Outbox::get_tx(dbtx, &self.outbox()).await?;

        if self.clone_uris.is_empty() {
            return Err(Error::db("repository: empty clone URIs"));
        }

        if self.push_uris.is_empty() {
            return Err(Error::db("repository: empty push URIs"));
        }

        let clone_uris = self
            .clone_uris()
            .iter()
            .map(VocabIri::from)
            .collect::<Vec<_>>();

        let push_uris = self
            .push_uris()
            .iter()
            .map(VocabIri::from)
            .collect::<Vec<_>>();

        let mut repository = VocabRepository::new()
            .with_id(self.id())
            .with_name(self.name())
            .with_inbox(VocabIri::from(inbox.id()))
            .with_outbox(VocabIri::from(outbox.id()))
            .with_clone_uri(clone_uris)
            .with_push_uri(push_uris);

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

        if assertion_method.len() > 1 {
            repository.set_assertion_method(assertion_method);
        } else if !assertion_method.is_empty() {
            assertion_method
                .into_iter()
                .next()
                .ok_or(Error::db("repository: missing multikey info"))
                .map(|v| repository.set_assertion_method(v))?;
        }

        if public_key.len() > 1 {
            repository.set_public_key(public_key);
        } else if !public_key.is_empty() {
            public_key
                .into_iter()
                .next()
                .ok_or(Error::db("repository: missing PEM public key info"))
                .map(|v| repository.set_public_key(v))?;
        }

        if let Some(followers_id) = self.followers_id() {
            repository.set_followers(VocabIri::from(followers_id));
        }

        if let Some(forks_id) = self.forks_id() {
            repository.set_forks(VocabIri::from(forks_id));
        }

        if let Some(likes_id) = self.likes_id() {
            repository.set_likes(VocabIri::from(likes_id));
        }

        if let Some(send_patches_to) = self.send_patches_to() {
            repository.set_send_patches_to(VocabIri::from(send_patches_to));
        }

        if let Some(tickets_tracked_by) = self.tickets_tracked_by() {
            repository.set_tickets_tracked_by(VocabIri::from(tickets_tracked_by));
        }

        if let Some(is_archived) = self.is_archived() {
            repository.set_is_archived(is_archived);
        }

        if let Some(moved_to) = self.moved_to() {
            repository.set_moved_to(VocabIri::from(moved_to));
        }

        if let Some(mirrors) = self.mirrors() {
            repository.set_mirrors(VocabIri::from(mirrors));
        }

        if let Some(team) = self.team() {
            repository.set_team(VocabIri::from(team));
        }

        Ok(repository)
    }
}

field_access! {
    Repository {
        /// Main [Uuid] for the [Repository] record.
        uuid: Uuid,
        /// Represents the [Uuid] of the [Inbox](crate::db::Inbox) record.
        inbox: Uuid,
        /// Represents the [Uuid] of the [Outbox](crate::db::Outbox) record.
        outbox: Uuid,
        /// Represents whether the repository is private.
        is_private: bool,
    }
}

field_access! {
    Repository {
        /// ActivityPub [Iri] for external actors to reference the [Repository].
        id: as_ref { Iri },
        /// Represents the human-readable name of the [Repository].
        name: as_ref { Name },
    }
}

field_access! {
    Repository {
        /// Represents whether the [Repository] is archived.
        is_archived: option { bool },
    }
}

field_access! {
    Repository {
        /// References the actor record for the [Repository] [PatchTracker](crate::db::PatchTracker).
        ///
        /// If nil, the [Repository] tracks its own patches.
        send_patches_to: option_ref { Iri },
        /// References the actor record for the [Repository] [TicketTracker](crate::db::TicketTracker).
        ///
        /// If nil, the [Repository] tracks its own patches.
        tickets_tracked_by: option_ref { Iri },
        /// ActivityPub [Iri] to fetch the [Repository] forks list.
        forks_id: option_ref { Iri },
        /// ActivityPub [Iri] to fetch the [Repository] followers list.
        followers_id: option_ref { Iri },
        /// ActivityPub [Iri] to fetch the [Repository] likes list.
        likes_id: option_ref { Iri },
        /// Represents the new [Repository] record for a [Repository] that is archived.
        moved_to: option_ref { Iri },
        /// References the [Repository] record mirrored by this [Repostory].
        ///
        /// The referenced [Repository] can be a minimal record with just the `id` field filled out.
        mirrors: option_ref { Iri },
        /// References to the [Team](crate::Team) record.
        team: option_ref { Iri },
    }
}

impl_sql_actor! {
    Repository {
        id: { "id"  Iri },
        name: { "name"  Name },
        inbox: { "inbox"  Uuid },
        outbox: { "outbox"  Uuid },
        clone_uris: { "clone_uris"  IriList },
        push_uris: { "push_uris"  IriList },
        forks_id: { "forks_id" OptionalIri },
        forks: { "forks"  IriList },
        likes_id: { "likes_id" OptionalIri },
        likes: { "likes"  UuidList },
        followers_id: { "followers_id"  OptionalIri },
        followers: { "followers"  IriList },
        key_ids: { "key_ids"  UuidList },
        send_patches_to: { "send_patches_to"  OptionalIri },
        tickets_tracked_by: { "tickets_tracked_by"  OptionalIri },
        is_archived: { "is_archived"  OptionalBool },
        moved_to: { "moved_to"  OptionalIri },
        mirrors: { "mirrors"  OptionalIri },
        team: { "team"  OptionalIri },
        is_private: { "is_private" bool },
    }
}

impl_sql_list_field! {
    Repository {
        /// Represents a list URIs to clone the [Repository].
        clone_uri: { "clone_uris" Iri },
        /// Represents a list URIs to push to the [Repository].
        push_uri: { "push_uris" Iri },
        /// References to other [Repository] records that are forked from this repository.
        ///
        /// List of [Uuid]s that should reference the [Repository] records for forks of this [Repository].
        ///
        /// Can be minimal records with just the `id` field filled out.
        fork: { "forks" Iri },
        /// Represents a list of references to the [Repository]'s [Follower](crate::db::Follower) records.
        follower: { "followers" Iri },
        /// Represents a list of references to the [Repository]'s [Key](crate::db::Key) records.
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
        key_id: { "key_ids" Uuid },
        /// Represents a list of references to the [Repository]'s [Like](crate::db::Like) records.
        like: { "likes" Uuid },
    }
}

impl_default!(Repository);
impl_display!(Repository, json);
