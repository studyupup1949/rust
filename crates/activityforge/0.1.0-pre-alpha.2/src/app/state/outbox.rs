use crate::db::{Activity, Create, Grant, Like, Outbox, TableType};
use crate::{Activity as VocabActivity, Error, Result};

use super::AppState;

impl AppState {
    /// Adds a [Outbox] activity to the database.
    pub async fn add_outbox_activity(
        &self,
        outbox: &mut Outbox,
        activity: &mut Activity,
    ) -> Result<()> {
        let db = self.db().await;
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        activity.insert_tx(&mut dbtx).await?;

        outbox
            .add_activity_tx(&mut dbtx, activity.table_entry())
            .await?;

        dbtx.commit()
            .await
            .map(|_| ())
            .map_err(|err| Error::db(format!("add_outbox_activity: {err}")))
    }

    /// Gets all [Activity] records for an [Outbox].
    pub async fn outbox_activities(&self, outbox: &Outbox) -> Result<Vec<VocabActivity>> {
        let mut activities = Vec::with_capacity(outbox.activities().len());

        let db = self.db().await;
        let pool = db.pool()?;
        let db_key = db.key()?;
        let mut dbtx = pool.begin().await?;

        for entry in outbox.activities() {
            let activity = match entry.table() {
                TableType::Create => Create::get_tx(&mut dbtx, &entry.id())
                    .await?
                    .try_into_vocab_tx(&mut dbtx, &db_key)
                    .await
                    .map(VocabActivity::create),
                TableType::Grant => Grant::get_tx(&mut dbtx, &entry.id())
                    .await?
                    .try_into_vocab_tx(&mut dbtx, &db_key)
                    .await
                    .map(VocabActivity::grant),
                TableType::Like => Like::get_tx(&mut dbtx, &entry.id())
                    .await?
                    .try_into_vocab()
                    .map(VocabActivity::like),
                _ => Err(Error::db(format!("outbox: invalid activity: {entry}"))),
            }?;

            activities.push(activity);
        }

        dbtx.commit()
            .await
            .map(|_| activities)
            .map_err(|err| Error::db(format!("outbox: {err}")))
    }
}
