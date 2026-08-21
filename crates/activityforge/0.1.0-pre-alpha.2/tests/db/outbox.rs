use activitystreams_vocabulary::Iri;

use activityforge::db::object::Outbox;
use activityforge::db::{TableEntry, TableType};

crate::db_test! {
    outbox => run_tests(db) {
        let person_uuid = db.rand_uuid();
        let outbox_id = Iri::try_from("https://example.dev/api/v1/persons/{perosn_uuid}/outbox")?;

        // dummy value for Outbox owner
        // in production, this entry should reference the owner table entry
        let mut outbox = Outbox::new()
            .with_id(outbox_id)
            .with_actor(TableEntry::create(TableType::Person, person_uuid));

        outbox.insert(db).await?;

        // dummy values for activities
        // in production, these entries should reference activity table entries
        outbox.add_activity(db, TableEntry::create(TableType::Grant, db.rand_uuid())).await?;

        outbox.add_activities(db, [TableEntry::create(TableType::Grant, db.rand_uuid())]).await?;

        let mut activities = outbox.activities().to_vec();
        activities.sort();

        let activity = activities.first().copied().unwrap();

        assert_eq!(Outbox::get(db, &outbox.uuid()).await.as_ref(), Ok(&outbox));

        // ensure adding a duplicate activity is an error.
        assert!(outbox.add_activity(db, activity).await.is_err());
        assert!(outbox.add_activities(db, activities.clone()).await.is_err());

        outbox.delete_activity(db, activity).await?;

        let deleted = outbox.delete_activities(db, activities.clone()).await?;

        assert_eq!(deleted.as_slice(), &activities[1..]);

        assert!(outbox.activities().is_empty());

        outbox.update_activities(db, activities.clone()).await?;

        assert_eq!(outbox.activities(), activities);
        assert_eq!(Outbox::get(db, &outbox.uuid()).await.as_ref(), Ok(&outbox));

        // clear all activities
        let empty_list: [TableEntry; 0] = [];
        outbox.update_activities(db, empty_list).await?;

        assert!(outbox.activities().is_empty());
        assert_eq!(Outbox::get(db, &outbox.uuid()).await.as_ref(), Ok(&outbox));

        let outbox_uuid = outbox.uuid();
        outbox.delete(db).await?;

        assert!(Outbox::get(db, &outbox_uuid).await.is_err());

        Ok(())
    }
}
