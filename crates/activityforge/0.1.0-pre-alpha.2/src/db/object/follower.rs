use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::{Db, Iri, TableEntry, TableEntryList, Transaction, Uuid};
use crate::{Error, Result, impl_sql_list_field, impl_sql_object, util};

/// Represents a an entry from a [Followers Collection](https://www.w3.org/TR/activitypub/#followers).
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Follower {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    actor: TableEntry,
    following: Vec<TableEntry>,
}

impl Follower {
    /// Creates a new [Follower].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            actor: TableEntry::new(),
            following: Vec::new(),
        }
    }

    /// Adds a `following` entry to the list.
    pub fn add_following_entry(&mut self, entry: TableEntry) -> Result<()> {
        if self.following.contains(&entry) {
            Err(Error::db(format!(
                "follower: duplicate following entry: {entry}"
            )))
        } else {
            self.following.push(entry);
            Ok(())
        }
    }

    /// Performs checks for record invariants.
    pub fn check_db(&self) -> Result<()> {
        if self.actor.is_empty() {
            Err(Error::sql("follower: empty actor"))
        } else if self.following.is_empty() {
            Err(Error::sql("follower: empty following list"))
        } else {
            Ok(())
        }
    }

    /// Finds a [Follower] record by an actor [TableEntry].
    pub async fn find_by_actor(db: &Db, actor: TableEntry) -> Result<Option<Self>> {
        let pool = db.pool()?;
        let mut dbtx = pool.begin().await?;

        let follower = Self::find_by_actor_tx(&mut dbtx, actor).await?;

        dbtx.commit()
            .await
            .map(|_| follower)
            .map_err(|err| Error::db(format!("follower: {err}")))
    }

    /// Finds a [Follower] record by an actor [TableEntry] using a transaction.
    pub async fn find_by_actor_tx(
        dbtx: &mut Transaction<'_>,
        actor: TableEntry,
    ) -> Result<Option<Self>> {
        let table = Self::TABLE;

        sqlx::query(format!("SELECT * FROM {table} WHERE actor = $1").as_str())
            .bind(actor)
            .fetch_optional(&mut **dbtx)
            .await
            .map_err(|err| Error::db(format!("follower: {err}")))
            .and_then(|row| {
                if let Some(row) = row {
                    Self::from_row(&row)
                        .map(Some)
                        .map_err(|err| Error::db(format!("follower: {err}")))
                } else {
                    Ok(None)
                }
            })
    }
}

field_access! {
    Follower {
        /// Represents the [Uuid] primary key of the table entry.
        uuid: Uuid,
        /// References the actor following the other actors.
        actor: TableEntry,
    }
}

field_access! {
    Follower {
        /// Represents the IRI used to fetch the [Follower].
        id: as_ref { Iri },
    }
}

impl_sql_object! {
    Follower {
        id: { "id" Iri },
        actor: { "actor" TableEntry },
        following: { "following" TableEntryList },
    }
}

impl_sql_list_field! {
    Follower {
        /// Represents the list of actors the [Follower] is following.
        follow, following: { "following" TableEntry },
    }
}

impl_default!(Follower);
impl_display!(Follower, json);
