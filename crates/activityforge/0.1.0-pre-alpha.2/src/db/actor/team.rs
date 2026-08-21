use activitystreams_vocabulary::{
    Collection, DateTime, Iri as VocabIri, Item, Items, Key as VocabPublicKey, KeyItem, KeyItems,
    Multikey, MultikeyItem, MultikeyItems, Name as VocabName, ObjectType, field_access,
    impl_default, impl_display,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx::types::chrono::Utc;

use crate::crypto::{PemPublicKey, SymmetricKey};
use crate::db::{
    Collaborator, Db, Grant, Inbox, Iri, Key, Name, OptionalString, OptionalUuid, Outbox,
    RoleFilterList, TableEntry, Transaction, Uuid, UuidList,
};
use crate::{
    ActorType, Error, Result, Role, Team as VocabTeam, impl_sql_actor, impl_sql_list_field, util,
};

mod builder;
mod role_filter;

pub use builder::*;
pub use role_filter::RoleFilter;

/// Represents a [Team Actor](https://forgefed.org/spec/#team) database record.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Team {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    name: Name,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    content: Option<String>,
    #[serde(
        skip_serializing_if = "Option::is_none",
        serialize_with = "util::ser_uuid_opt",
        deserialize_with = "util::de_uuid_opt"
    )]
    context: Option<Uuid>,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    inbox: Uuid,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    outbox: Uuid,
    published: DateTime,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    members: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    subteams: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    oversees: Vec<Uuid>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    overseen_bys: Vec<Uuid>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    role_filters: Vec<RoleFilter>,
    #[serde(
        skip_serializing_if = "Vec::is_empty",
        serialize_with = "util::ser_uuid_list",
        deserialize_with = "util::de_uuid_list"
    )]
    key_ids: Vec<Uuid>,
}

impl Team {
    /// Creates a new [Team].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            name: Name::new(),
            summary: None,
            content: None,
            context: None,
            inbox: Uuid::nil(),
            outbox: Uuid::nil(),
            published: DateTime::default(),
            members: Vec::new(),
            subteams: Vec::new(),
            oversees: Vec::new(),
            overseen_bys: Vec::new(),
            role_filters: Vec::new(),
            key_ids: Vec::new(),
        }
    }

    /// Creates a new [TeamBuilder] to create a new [Team].
    pub fn builder<I: Into<Iri>, N: Into<Name>>(id: I, name: N) -> Result<TeamBuilder> {
        TeamBuilder::new(id, name)
    }

    /// Performs checks for record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.id.is_empty() {
            Err(Error::sql("team: empty ID"))
        } else if self.name.as_str().is_empty() {
            Err(Error::sql("team: empty name"))
        } else if self.inbox.is_nil() {
            Err(Error::sql("team: nil inbox"))
        } else if self.outbox.is_nil() {
            Err(Error::sql("team: nil outbox"))
        } else if let Some(context) = self.context.as_ref()
            && !self.uuid.is_nil()
            && &self.uuid == context
        {
            Err(Error::sql("team: context references this Team"))
        } else if !self.uuid.is_nil() && self.subteams.contains(&self.uuid) {
            Err(Error::sql(
                "team: subteams contains a reference to this Team",
            ))
        } else if !self.uuid.is_nil() && self.oversees.contains(&self.uuid) {
            Err(Error::sql(
                "team: oversees contains a reference to this Team",
            ))
        } else if !self.uuid.is_nil() && self.overseen_bys.contains(&self.uuid) {
            Err(Error::sql(
                "team: overseen_by contains a reference to this Team",
            ))
        } else {
            Ok(())
        }
    }

    /// Converts a [Team](VocabTeam) JSON-LD record into a database [Team] record.
    ///
    /// Useful for creating records fetched from remote instances.
    ///
    /// Creates a random UUID for local storage.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab(db: &Db, val: &VocabTeam) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let team = Self::try_from_vocab_tx(&mut dbtx, &db_key, val).await?;

        dbtx.commit()
            .await
            .map(|_| team)
            .map_err(|err| Error::db(format!("team: {err}")))
    }

    /// Converts a [Team](VocabTeam) JSON-LD record into a database [Team] record using a transaction.
    ///
    /// Useful for creating records fetched from remote instances.
    ///
    /// Creates a random UUID for local storage.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        val: &VocabTeam,
    ) -> Result<Self> {
        let uuid = val
            .id()
            .ok_or(Error::db("team: missing ID"))
            .map(|id| Self::TABLE.uuid_from_id(id))?;

        Self::try_from_vocab_with_uuid_tx(dbtx, db_key, val, uuid).await
    }

    /// Converts a [Team](VocabTeam) JSON-LD record into a database [Team] record.
    ///
    /// Useful for creating local records, e.g. parsed from a `Create` activity.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    pub async fn try_from_vocab_with_uuid(db: &Db, val: &VocabTeam, uuid: Uuid) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let team = Self::try_from_vocab_with_uuid_tx(&mut dbtx, &db_key, val, uuid).await?;

        dbtx.commit()
            .await
            .map(|_| team)
            .map_err(|err| Error::db(format!("team: {err}")))
    }

    /// Converts a [Team](VocabTeam) JSON-LD record into a database [Team] record.
    ///
    /// Useful for creating local records, e.g. parsed from a `Create` activity.
    ///
    /// Does not attempt to fetch records referenced using an [Iri] or [Link](activitystreams_vocabulary::Link).
    #[allow(deprecated)]
    pub async fn try_from_vocab_with_uuid_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        val: &VocabTeam,
        uuid: Uuid,
    ) -> Result<Self> {
        let actor = TableEntry::create(Self::TABLE, uuid);

        let id: Iri = val
            .id()
            .map(|id| id.into())
            .ok_or(Error::db("team: missing id"))?;

        let name: Name = val
            .name()
            .map(|name| name.into())
            .ok_or(Error::db("team: missing name"))?;

        let inbox_id = val
            .inbox()
            .map(|i| i.into())
            .ok_or(Error::db("team: missing inbox"))
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
            .ok_or(Error::db("team: missing outbox"))
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

        log::debug!("team: commiting storage for ID: {id}");

        let mut team = Self {
            uuid,
            id,
            name,
            summary: val.summary().map(|v| v.to_string()),
            content: val.content().map(|v| v.to_string()),
            context: None,
            inbox: inbox.uuid(),
            outbox: outbox.uuid(),
            published: val.published().unwrap_or(Utc::now().into()),
            members: Vec::new(),
            subteams: Vec::new(),
            oversees: Vec::new(),
            overseen_bys: Vec::new(),
            role_filters: val.role_filter().map(|f| f.into()).unwrap_or_default(),
            key_ids,
        };

        if let Some(members_items) = val.members() {
            let mut members = Vec::new();

            match members_items.items() {
                Some(Items::Single(item)) => {
                    let member = Collaborator::try_from_item_tx(dbtx, item).await?;
                    if member.subject() == team.id() {
                        members.push(member.uuid())
                    } else {
                        return Err(Error::db(format!(
                            "team: invalid member ID: {}, expected: {}",
                            member.subject(),
                            team.id()
                        )));
                    }
                }
                Some(Items::List(list)) => {
                    let item = list
                        .iter()
                        .find(|i| {
                            i.as_object()
                                .map(|o| o.kind().contains(ObjectType::Relationship))
                                .unwrap_or_default()
                        })
                        .ok_or(Error::db("team: missing valid member Item"))?;

                    let member = Collaborator::try_from_item_tx(dbtx, item).await?;
                    if member.subject() == team.id() {
                        members.push(member.uuid())
                    } else {
                        return Err(Error::db(format!(
                            "team: invalid member ID: {}, expected: {}",
                            member.subject(),
                            team.id()
                        )));
                    }
                }
                None => return Err(Error::db("team: missing member items")),
            }

            team.set_members(members)?;
        }

        if let Some(subteam_collection) = val.subteams() {
            let subteam_items = subteam_collection
                .items()
                .ok_or(Error::db("team: missing subteam items"))?;

            let subteams = Self::try_from_component_items_tx(dbtx, subteam_items)
                .await
                .map(|s| s.into_iter().map(|subteam| subteam.uuid()))?;

            team.set_subteams(subteams)?;
        }

        if let Some(oversee_collection) = val.oversees() {
            let oversee_items = oversee_collection
                .items()
                .ok_or(Error::db("team: missing oversee items"))?;

            let oversees = Self::try_from_component_items_tx(dbtx, oversee_items)
                .await
                .map(|s| s.into_iter().map(|oversee| oversee.uuid()))?;

            team.set_oversees(oversees)?;
        }

        if let Some(overseen_collection) = val.overseen_by() {
            let overseen_items = overseen_collection
                .items()
                .ok_or(Error::db("team: missing oversee items"))?;

            let overseen_by = Self::try_from_component_items_tx(dbtx, overseen_items)
                .await
                .map(|s| s.into_iter().map(|oversee| oversee.uuid()))?;

            team.set_overseen_bys(overseen_by)?;
        }

        team.find_or_create_tx(dbtx).await.map(|_| team)
    }

    /// Attempts to convert an [Item] into a [Team] database record.
    pub async fn try_from_item(db: &Db, item: &Item) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let team = Self::try_from_item_tx(&mut dbtx, &db_key, item).await?;

        dbtx.commit()
            .await
            .map(|_| team)
            .map_err(|err| Error::db(format!("team: {err}")))
    }

    /// Attempts to convert an [Item] into a [Team] database record using a transaction.
    pub async fn try_from_item_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        item: &Item,
    ) -> Result<Self> {
        match item {
            Item::Object(object) => {
                let team = VocabTeam::try_from(object.as_ref())
                    .map_err(|err| Error::db(format!("team: {err}")))?;
                Self::try_from_vocab_tx(dbtx, db_key, &team).await
            }
            Item::Iri(iri) => {
                let team = VocabTeam::new().with_id(iri.as_ref().clone());
                Self::try_from_vocab_tx(dbtx, db_key, &team).await
            }
            Item::Link(link) => {
                let team = VocabTeam::new().with_id(link.href().clone());
                Self::try_from_vocab_tx(dbtx, db_key, &team).await
            }
        }
    }

    /// Attempts to convert an [Item] into a [Team] database record.
    pub async fn try_from_component_item(db: &Db, item: &Item) -> Result<Self> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let team = Self::try_from_component_item_tx(&mut dbtx, item).await?;

        dbtx.commit()
            .await
            .map(|_| team)
            .map_err(|err| Error::db(format!("team: {err}")))
    }

    /// Attempts to convert an [Item] into a [Team] database record using a transaction.
    pub async fn try_from_component_item_tx(
        dbtx: &mut Transaction<'_>,
        item: &Item,
    ) -> Result<Self> {
        match item {
            Item::Object(object) => {
                let id = object
                    .id()
                    .ok_or(Error::db("team: missing component team ID"))?;
                let mut team = Self::new().with_id(id);
                team.find_or_create_tx(dbtx).await?;
                Ok(team)
            }
            Item::Iri(iri) => {
                let mut team = Self::new().with_id(iri.as_ref());
                team.find_or_create_tx(dbtx).await?;
                Ok(team)
            }
            Item::Link(link) => {
                let mut team = Self::new().with_id(link.href());
                team.find_or_create_tx(dbtx).await?;
                Ok(team)
            }
        }
    }

    /// Attempts to convert an [Items] collection into a list of [Team] database record.
    pub async fn try_from_items(db: &Db, items: &Items) -> Result<Vec<Self>> {
        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let teams = Self::try_from_items_tx(&mut dbtx, &db_key, items).await?;

        dbtx.commit()
            .await
            .map(|_| teams)
            .map_err(|err| Error::db(format!("team: {err}")))
    }

    /// Attempts to convert an [Items] collection into a list of [Team] database record using a transaction.
    pub async fn try_from_items_tx(
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
        items: &Items,
    ) -> Result<Vec<Self>> {
        match items {
            Items::Single(item) => Self::try_from_item_tx(dbtx, db_key, item)
                .await
                .map(|s| vec![s]),
            Items::List(list) => {
                let mut teams = Vec::new();

                for item in list.iter().filter(|i| {
                    let is_team = i
                        .as_object()
                        .map(|o| o.kind().contains(ActorType::Team))
                        .unwrap_or_default();

                    is_team || i.is_iri() || i.is_link()
                }) {
                    let team = Self::try_from_item_tx(dbtx, db_key, item).await?;
                    teams.push(team);
                }

                Ok(teams)
            }
        }
    }

    /// Attempts to convert an [Items] collection into a list of [Team] database record.
    pub async fn try_from_component_items(db: &Db, items: &Items) -> Result<Vec<Self>> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let teams = Self::try_from_component_items_tx(&mut dbtx, items).await?;

        dbtx.commit()
            .await
            .map(|_| teams)
            .map_err(|err| Error::db(format!("team: {err}")))
    }

    /// Attempts to convert an [Items] collection into a list of [Team] database record using a transaction.
    pub async fn try_from_component_items_tx(
        dbtx: &mut Transaction<'_>,
        items: &Items,
    ) -> Result<Vec<Self>> {
        match items {
            Items::Single(item) => Self::try_from_component_item_tx(dbtx, item)
                .await
                .map(|s| vec![s]),
            Items::List(list) => {
                let mut teams = Vec::new();

                for item in list.iter().filter(|i| {
                    let is_team = i
                        .as_object()
                        .map(|o| o.kind().contains(ActorType::Team))
                        .unwrap_or_default();

                    is_team || i.is_iri() || i.is_link()
                }) {
                    let team = Self::try_from_component_item_tx(dbtx, item).await?;
                    teams.push(team);
                }

                Ok(teams)
            }
        }
    }

    /// Attempts to convert a [Team] database record into a JSON-LD [Team](VocabTeam).
    pub async fn try_into_vocab(&self, db: &Db) -> Result<VocabTeam> {
        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let team = self.try_into_vocab_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| team)
            .map_err(|err| Error::db(format!("team: {err}")))
    }

    /// Attempts to convert a [Team] database record into a JSON-LD [Team](VocabTeam) using a transaction.
    #[allow(deprecated)]
    pub async fn try_into_vocab_tx(
        &self,
        dbtx: &mut Transaction<'_>,
        db_key: &SymmetricKey,
    ) -> Result<VocabTeam> {
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

        let mut team = VocabTeam::new()
            .with_id(id)
            .with_name(name)
            .with_inbox(inbox)
            .with_outbox(outbox)
            .with_published(*self.published());

        if let Some(summary) = self.summary() {
            team.set_summary(summary);
        }

        if let Some(content) = self.content() {
            team.set_content(content);
        }

        if !self.members().is_empty() {
            let mut members = Vec::with_capacity(self.members().len());

            for member_id in self.members() {
                Collaborator::get_tx(dbtx, member_id)
                    .await
                    .map(|m| VocabIri::from(m.id()))
                    .map(|m| members.push(m))?;
            }

            let collection = Collection::new()
                .with_total_items(members.len() as u64)
                .with_items(members);

            team.set_members(collection);
        }

        if !self.subteams().is_empty() {
            let mut subteams = Vec::with_capacity(self.subteams().len());

            for subteam_id in self.subteams() {
                Collaborator::get_tx(dbtx, subteam_id)
                    .await
                    .map(|m| VocabIri::from(m.id()))
                    .map(|m| subteams.push(m))?;
            }

            let collection = Collection::new()
                .with_total_items(subteams.len() as u64)
                .with_items(subteams);

            team.set_subteams(collection);
        }

        if !self.oversees().is_empty() {
            let mut oversees = Vec::with_capacity(self.oversees().len());

            for oversee_id in self.oversees() {
                Team::get_tx(dbtx, oversee_id)
                    .await
                    .map(|m| VocabIri::from(m.id()))
                    .map(|m| oversees.push(m))?;
            }

            let collection = Collection::new()
                .with_total_items(oversees.len() as u64)
                .with_items(oversees);

            team.set_oversees(collection);
        }

        if !self.overseen_bys().is_empty() {
            let mut overseen_by = Vec::with_capacity(self.overseen_bys().len());

            for overseen_id in self.overseen_bys() {
                Team::get_tx(dbtx, overseen_id)
                    .await
                    .map(|m| VocabIri::from(m.id()))
                    .map(|m| overseen_by.push(m))?;
            }

            let collection = Collection::new()
                .with_total_items(overseen_by.len() as u64)
                .with_items(overseen_by);

            team.set_overseen_by(collection);
        }

        if !self.role_filters().is_empty() {
            team.set_role_filter(self.role_filters());
        }

        if assertion_method.len() > 1 {
            team.set_assertion_method(assertion_method);
        } else if !assertion_method.is_empty() {
            assertion_method
                .into_iter()
                .next()
                .ok_or(Error::db("team: missing multikey info"))
                .map(|v| team.set_assertion_method(v))?;
        }

        if public_key.len() > 1 {
            team.set_public_key(public_key);
        } else if !public_key.is_empty() {
            public_key
                .into_iter()
                .next()
                .ok_or(Error::db("team: missing PEM public key info"))
                .map(|v| team.set_public_key(v))?;
        }

        Ok(team)
    }
}

field_access! {
    Team {
        /// Represents the [Uuid] primary key of the table entry.
        uuid: Uuid,
        /// Represents the [Uuid] of the [Inbox](crate::db::Inbox) record.
        inbox: Uuid,
        /// Represents the [Uuid] of the [Outbox](crate::db::Outbox) record.
        outbox: Uuid,
    }
}

field_access! {
    Team {
        /// Represents the [Uuid] of the parent [Team] of this [Team].
        ///
        /// The current [Team] inherits access to resources from the parent [Team] referenced by `context`.
        context: option { Uuid },
    }
}

field_access! {
    Team {
        /// Represents the IRI used to fetch the [Team] record.
        id: as_ref { Iri },
        /// Represents the human-readable [Team] name.
        name: as_ref { Name },
        /// Represents the timestamp for the [Team]'s creation.
        published: as_ref { DateTime },
    }
}

field_access! {
    Team {
        /// Represents the [Team]'s content description.
        content: option_deref { &str, String },
        /// Represents the [Team]'s summary.
        summary: option_deref { &str, String },
    }
}

impl_sql_list_field! {
    Team {
        /// References the list of [Collaborator](crate::db::Collaborator) members.
        member: { "members" Uuid },
        /// References the list of [subteams](Self).
        subteam: { "subteams" Uuid },
        /// References the list of [Team]s this [Team] oversees.
        oversee: { "oversees" Uuid },
        /// References the list of [Team]s overseen by this [Team].
        overseen_by: { "overseen_by" Uuid },
        /// References the list of [RoleFilter] entries for [Team] members.
        ///
        /// Allows for fine-grained access control for individual [Team] members to shared resources.
        role_filter: { "role_filter" RoleFilter },
        /// Represents a list of references to the [Factory]'s [Key](crate::db::Key) records.
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
    }
}

impl_default!(Team);
impl_display!(Team, json);

impl_sql_actor! {
    Team {
        id: { "id"  Iri },
        name: { "name"  Name },
        summary: { "summary"  OptionalString },
        content: { "content"  OptionalString },
        context: { "context"  OptionalUuid },
        inbox: { "inbox"  Uuid },
        outbox: { "outbox"  Uuid },
        published: { "published"  DateTime },
        members: { "members"  UuidList },
        subteams: { "subteams"  UuidList },
        oversees: { "oversees"  UuidList },
        overseen_bys: { "overseen_by"  UuidList },
        role_filters: { "role_filter"  RoleFilterList },
        key_ids: { "key_ids"  UuidList },
    }
}
