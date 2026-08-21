use activityforge::crypto::KeyType;
use activityforge::db::{Follower, Iri, Key, Name, Person, TicketTracker};

crate::db_test! {
    ticket_tracker => run_tests(db) {
        let host = Iri::try_from("https://example.dev/api/v1")?;

        let tracker_uuid = db.rand_uuid();
        let tracker_id = Iri::try_from(format!("{host}/ticket_trackers/{tracker_uuid}"))?;

        let tracker_name = Name::try_from("test ticket tracker")?;

        let follower_person_uuid = db.rand_uuid();
        let follower_person_id = Person::TABLE.id_from_uuid(&host, follower_person_uuid)?;
        let follower_person_name = Name::try_from("test ticket tracker follower")?;

        let follower_person = Person::builder(follower_person_id, follower_person_name)
            .and_then(|p| p.uuid(follower_person_uuid))?
            .build(db)
            .await?;

        let follower_uuid = db.rand_uuid();
        let follower_id = Follower::TABLE.id_from_uuid(&host, follower_uuid)?;
        let follower = Follower::new()
            .with_id(follower_id)
            .with_actor(follower_person.table_entry());

        let tracker_key_uuid = db.rand_uuid();
        let tracker_key_id = Key::TABLE.id_from_uuid(&host, tracker_key_uuid)?;
        let mut tracker_key_data = [0u8; 64];
        rand::fill(&mut tracker_key_data);

        let mut tracker_key = Key::new()
            .with_uuid(tracker_key_uuid)
            .with_id(tracker_key_id)
            .with_key_type(KeyType::Ed25519)
            .with_key(tracker_key_data)
            .with_is_private(true);

        let tracker_summary = "ticket tracker summary";
        let tracker_content = "ticket tracker content";

        let mut ticket_tracker = TicketTracker::builder(tracker_id, tracker_name)
            .and_then(|b| b.uuid(tracker_uuid))
            .and_then(|b| b.keys([tracker_key.clone()]))?
            .followers([follower])
            .summary(tracker_summary)
            .content(tracker_content)
            .build(db)
            .await?;

        assert_eq!(TicketTracker::get(db, &tracker_uuid).await?, ticket_tracker);

        tracker_key.set_actor_id(ticket_tracker.id());
        tracker_key.set_actor(ticket_tracker.table_entry());

        assert_eq!(Key::get(db, &tracker_key.uuid()).await?, tracker_key);

        let edited_summary = "edited summary";
        ticket_tracker.set_summary(edited_summary);
        ticket_tracker.update(db).await?;

        assert_eq!(ticket_tracker.summary(), Some(edited_summary));
        assert_eq!(TicketTracker::get(db, &tracker_uuid).await?, ticket_tracker);

        let edited_content = "edited content";
        ticket_tracker.set_content(edited_content);
        ticket_tracker.update(db).await?;

        assert_eq!(ticket_tracker.content(), Some(edited_content));
        assert_eq!(TicketTracker::get(db, &tracker_uuid).await?, ticket_tracker);

        ticket_tracker.delete(db).await?;
        assert!(TicketTracker::get(db, &tracker_uuid).await.is_err());

        Ok(())
    }
}
