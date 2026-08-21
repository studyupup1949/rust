use activitystreams_vocabulary::Iri;

use activityforge::db::object::Outbox;
use activityforge::db::{Db, TableEntry, TableType};
use activityforge::{Error, Result};

/// Tests database operations on [Outbox] records.
pub async fn test_outbox(db: &Db) -> Result<()> {
    let outbox_id = Iri::try_from("https://example.dev/test_outbox/outbox")?;

    // dummy value for Outbox owner
    // in production, this entry should reference the owner table entry
    let mut outbox = Outbox::new()
        .with_id(outbox_id)
        .with_actor(TableEntry::create(TableType::Person, db.rand_uuid()));

    outbox
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting test outbox: {err}")))?;

    // dummy values for activities
    // in production, these entries should reference activity table entries

    outbox
        .add_activity(db, TableEntry::create(TableType::Grant, db.rand_uuid()))
        .await
        .map_err(|err| Error::sql(format!("error adding outbox activity: {err}")))?;

    outbox
        .add_activities(db, [TableEntry::create(TableType::Grant, db.rand_uuid())])
        .await
        .map_err(|err| Error::sql(format!("error adding outbox activity: {err}")))?;

    let mut activities = outbox.activities().to_vec();
    activities.sort();

    let activity = activities.first().copied().unwrap();

    assert_eq!(Outbox::get(db, &outbox.uuid()).await.as_ref(), Ok(&outbox));

    // ensure adding a duplicate activity is an error.
    assert!(outbox.add_activity(db, activity).await.is_err());
    assert!(outbox.add_activities(db, activities.clone()).await.is_err());

    outbox
        .delete_activity(db, activity)
        .await
        .map_err(|err| Error::sql(format!("error deleting outbox activity: {err}")))?;

    let deleted = outbox
        .delete_activities(db, activities.clone())
        .await
        .map_err(|err| Error::sql(format!("error deleting outbox activity: {err}")))?;

    assert_eq!(deleted.as_slice(), &activities[1..]);

    assert!(outbox.activities().is_empty());

    outbox
        .update_activities(db, activities.clone())
        .await
        .map_err(|err| Error::sql(format!("error updating outbox activities: {err}")))?;

    assert_eq!(outbox.activities(), activities);
    assert_eq!(Outbox::get(db, &outbox.uuid()).await.as_ref(), Ok(&outbox));

    // clear all activities
    let empty_list: [TableEntry; 0] = [];

    outbox
        .update_activities(db, empty_list)
        .await
        .map_err(|err| Error::sql(format!("error updating outbox activities: {err}")))?;

    assert!(outbox.activities().is_empty());
    assert_eq!(Outbox::get(db, &outbox.uuid()).await.as_ref(), Ok(&outbox));

    let outbox_uuid = outbox.uuid();
    outbox
        .delete(db)
        .await
        .map_err(|err| Error::sql(format!("error deleting outbox: {err}")))?;

    assert!(Outbox::get(db, &outbox_uuid).await.is_err());

    Ok(())
}
