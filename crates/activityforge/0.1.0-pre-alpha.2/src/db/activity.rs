//! Database activity records.

use activitystreams_vocabulary::create_item;

use crate::crypto::SymmetricKey;
use crate::db::{Db, TableEntry, Uuid};
use crate::{Activity as VocabActivity, Error, Result};

mod accept;
mod create;
mod follow;
mod grant;
mod like;

pub use accept::*;
pub use create::*;
pub use follow::*;
pub use grant::*;
pub use like::*;

create_item! {
    /// Represents the `Activity` records stored in the database.
    Activity
        boxed
        default: Self::Create(Box::default()),
    {
        Accept(Accept),
        Create(Create),
        Follow(Follow),
        Grant(Grant),
        Like(Like),
    }
}

impl Activity {
    /// Gets the [Activity] table entry.
    pub fn table_entry(&self) -> TableEntry {
        match self {
            Self::Accept(a) => a.table_entry(),
            Self::Create(a) => a.table_entry(),
            Self::Follow(a) => a.table_entry(),
            Self::Grant(a) => a.table_entry(),
            Self::Like(a) => a.table_entry(),
        }
    }

    /// Attempts to insert a [Activity] record in the database.
    pub async fn insert(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let uuid = self.insert_tx(&mut dbtx).await?;

        dbtx.commit()
            .await
            .map(|_| uuid)
            .map_err(|err| Error::db(format!("activity: {err}")))
    }

    /// Attempts to insert a [Activity] record in the database using a transaction.
    pub async fn insert_tx(
        &mut self,
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
    ) -> Result<Uuid> {
        match self {
            Self::Accept(a) => a.insert_tx(dbtx).await,
            Self::Create(a) => a.insert_tx(dbtx).await,
            Self::Follow(a) => a.insert_tx(dbtx).await,
            Self::Grant(a) => a.insert_tx(dbtx).await,
            Self::Like(a) => a.insert_tx(dbtx).await,
        }
    }

    /// Attempts to convert an [Activity] record into a JSON-LD object.
    pub async fn try_into_vocab(&self, db: &Db) -> Result<VocabActivity> {
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        let activity = self.try_into_vocab_tx(&mut dbtx, &db_key).await?;

        dbtx.commit()
            .await
            .map(|_| activity)
            .map_err(|err| Error::db(format!("activity: {err}")))
    }

    /// Attempts to convert an [Activity] record into a JSON-LD object.
    pub async fn try_into_vocab_tx(
        &self,
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<VocabActivity> {
        match self {
            Self::Accept(a) => a
                .try_into_vocab_tx(dbtx, db_key)
                .await
                .map(VocabActivity::accept),
            Self::Create(a) => a
                .try_into_vocab_tx(dbtx, db_key)
                .await
                .map(VocabActivity::create),
            Self::Follow(a) => a
                .try_into_vocab_tx(dbtx, db_key)
                .await
                .map(VocabActivity::follow),
            Self::Grant(a) => a
                .try_into_vocab_tx(dbtx, db_key)
                .await
                .map(VocabActivity::grant),
            Self::Like(a) => a.try_into_vocab().map(VocabActivity::like),
        }
    }
}
