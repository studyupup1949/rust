use activitystreams_vocabulary::Iri;

use activityforge::db::object::Inbox;
use activityforge::db::{Db, TableEntry, TableType};
use activityforge::{Error, Result};

/// Tests database operations on [Inbox] records.
pub async fn test_inbox(db: &Db) -> Result<()> {
    let inbox_id = Iri::try_from("https://example.dev/test_inbox/inbox")?;

    // dummy value for Inbox owner
    // in production, this entry should reference the owner table entry
    let mut inbox = Inbox::new()
        .with_id(inbox_id)
        .with_actor(TableEntry::create(TableType::Person, db.rand_uuid()));

    inbox
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting test inbox: {err}")))?;

    // dummy values for activities
    // in production, these entries should reference activity table entries

    inbox
        .add_activity(db, TableEntry::create(TableType::Grant, db.rand_uuid()))
        .await
        .map_err(|err| Error::sql(format!("error adding inbox activity: {err}")))?;

    inbox
        .add_activities(db, [TableEntry::create(TableType::Grant, db.rand_uuid())])
        .await
        .map_err(|err| Error::sql(format!("error adding inbox activity: {err}")))?;

    let mut activities = inbox.activities().to_vec();
    activities.sort();

    let activity = activities.first().copied().unwrap();

    assert_eq!(Inbox::get(db, &inbox.uuid()).await.as_ref(), Ok(&inbox));

    // ensure adding a duplicate activity is an error.
    assert!(inbox.add_activity(db, activity).await.is_err());
    assert!(inbox.add_activities(db, activities.clone()).await.is_err());

    inbox
        .delete_activity(db, activity)
        .await
        .map_err(|err| Error::sql(format!("error deleting inbox activity: {err}")))?;

    let deleted = inbox
        .delete_activities(db, activities.clone())
        .await
        .map_err(|err| Error::sql(format!("error deleting inbox activity: {err}")))?;

    assert_eq!(deleted.as_slice(), &activities[1..]);

    assert!(inbox.activities().is_empty());

    inbox
        .update_activities(db, activities.clone())
        .await
        .map_err(|err| Error::sql(format!("error updating inbox activities: {err}")))?;

    assert_eq!(inbox.activities(), activities);
    assert_eq!(Inbox::get(db, &inbox.uuid()).await.as_ref(), Ok(&inbox));

    // clear all activities
    let empty_list: [TableEntry; 0] = [];

    inbox
        .update_activities(db, empty_list)
        .await
        .map_err(|err| Error::sql(format!("error updating inbox activities: {err}")))?;

    assert!(inbox.activities().is_empty());
    assert_eq!(Inbox::get(db, &inbox.uuid()).await.as_ref(), Ok(&inbox));

    let inbox_uuid = inbox.uuid();
    inbox
        .delete(db)
        .await
        .map_err(|err| Error::sql(format!("error deleting inbox: {err}")))?;

    assert!(Inbox::get(db, &inbox_uuid).await.is_err());

    Ok(())
}
