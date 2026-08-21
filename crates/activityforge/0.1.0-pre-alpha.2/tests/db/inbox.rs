use activityforge::db::{Inbox, Iri, TableEntry, TableType};

crate::db_test! {
    inbox => run_tests(db) {
        let person_uuid = db.rand_uuid();
        let inbox_id = Iri::try_from(format!("https://example.dev/api/v1/persons/{person_uuid}/inbox"))?;

        // dummy value for Inbox owner
        // in production, this entry should reference the owner table entry
        let mut inbox = Inbox::new()
            .with_id(inbox_id)
            .with_actor(TableEntry::create(TableType::Person, person_uuid));

        inbox.insert(db).await?;

        // dummy values for activities
        // in production, these entries should reference activity table entries
        inbox.add_activity(db, TableEntry::create(TableType::Grant, db.rand_uuid())).await?;

        inbox.add_activities(db, [TableEntry::create(TableType::Grant, db.rand_uuid())]).await?;

        let mut activities = inbox.activities().to_vec();
        activities.sort();

        let activity = activities.first().copied().unwrap();

        assert_eq!(Inbox::get(db, &inbox.uuid()).await.as_ref(), Ok(&inbox));

        // ensure adding a duplicate activity is an error.
        assert!(inbox.add_activity(db, activity).await.is_err());
        assert!(inbox.add_activities(db, activities.clone()).await.is_err());

        inbox.delete_activity(db, activity).await?;

        let deleted = inbox.delete_activities(db, activities.clone()).await?;

        assert_eq!(deleted.as_slice(), &activities[1..]);

        assert!(inbox.activities().is_empty());

        inbox.update_activities(db, activities.clone()).await?;

        assert_eq!(inbox.activities(), activities);
        assert_eq!(Inbox::get(db, &inbox.uuid()).await.as_ref(), Ok(&inbox));

        // clear all activities
        let empty_list: [TableEntry; 0] = [];

        inbox.update_activities(db, empty_list).await?;

        assert!(inbox.activities().is_empty());
        assert_eq!(Inbox::get(db, &inbox.uuid()).await.as_ref(), Ok(&inbox));

        let inbox_uuid = inbox.uuid();
        inbox.delete(db).await?;

        assert!(Inbox::get(db, &inbox_uuid).await.is_err());

        Ok(())
    }
}
