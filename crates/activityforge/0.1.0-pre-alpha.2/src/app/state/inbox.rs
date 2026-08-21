use crate::db::{Accept, Activity, Create, Follow, Grant, Inbox, Like, TableType};
use crate::{Activity as VocabActivity, Error, Result};

use super::AppState;

impl AppState {
    /// Adds a [Inbox] activity to the database.
    pub async fn add_inbox_activity(
        &self,
        inbox: &mut Inbox,
        activity: &mut Activity,
    ) -> Result<()> {
        let db = self.db().await;
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        activity.insert_tx(&mut dbtx).await?;

        inbox
            .add_activity_tx(&mut dbtx, activity.table_entry())
            .await?;

        dbtx.commit()
            .await
            .map(|_| ())
            .map_err(|err| Error::db(format!("add_inbox_activity: {err}")))
    }

    /// Gets all [Activity] records for an [Inbox].
    pub async fn inbox_activities(&self, inbox: &Inbox) -> Result<Vec<VocabActivity>> {
        let mut activities = Vec::with_capacity(inbox.activities().len());

        let db = self.db().await;
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        for entry in inbox.activities() {
            let activity = match entry.table() {
                TableType::Accept => Accept::get_tx(&mut dbtx, &entry.id())
                    .await?
                    .try_into_vocab_tx(&mut dbtx, &db_key)
                    .await
                    .map(VocabActivity::accept),
                TableType::Create => Create::get_tx(&mut dbtx, &entry.id())
                    .await?
                    .try_into_vocab_tx(&mut dbtx, &db_key)
                    .await
                    .map(VocabActivity::create),
                TableType::Follow => Follow::get_tx(&mut dbtx, &entry.id())
                    .await?
                    .try_into_vocab_tx(&mut dbtx, &db_key)
                    .await
                    .map(VocabActivity::follow),
                TableType::Grant => Grant::get_tx(&mut dbtx, &entry.id())
                    .await?
                    .try_into_vocab_tx(&mut dbtx, &db_key)
                    .await
                    .map(VocabActivity::grant),
                TableType::Like => Like::get_tx(&mut dbtx, &entry.id())
                    .await?
                    .try_into_vocab()
                    .map(VocabActivity::like),
                _ => Err(Error::db(format!("inbox: invalid activity: {entry}"))),
            }?;

            activities.push(activity);
        }

        dbtx.commit()
            .await
            .map(|_| activities)
            .map_err(|err| Error::db(format!("inbox: {err}")))
    }
}
