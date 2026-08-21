use activityforge::crypto::KeyType;
use activityforge::db::{Inbox, Iri, Key, Name, Outbox, Person};

crate::db_test! {
    person => run_tests(db) {
        let host = "https://example.dev/api/v1";

        let person_uuid = db.rand_uuid();
        let person_id = Iri::try_from(format!("{host}/persons/{person_uuid}"))?;
        let person_name = Name::try_from("test_person")?;

        let person_key_uuid = db.rand_uuid();
        let person_key_id = Iri::try_from(format!("{host}/keys/{person_key_uuid}"))?;

        let mut person_key_data = [0u8; 64];
        rand::fill(&mut person_key_data);

        let mut person_key = Key::new()
            .with_uuid(person_key_uuid)
            .with_id(person_key_id)
            .with_key_type(KeyType::Ed25519)
            .with_key(person_key_data)
            .with_is_private(true);

        let person = Person::builder(person_id, person_name)
            .and_then(|b| b.uuid(person_uuid))
            .and_then(|b| b.keys([person_key.clone()]))?
            .build(db)
            .await?;

        person_key.set_actor_id(person.id());
        person_key.set_actor(person.table_entry());

        assert_eq!(Key::get(db, &person_key_uuid).await?, person_key);

        assert_eq!(Person::get(db, &person_uuid).await?, person);
        assert_eq!(Person::find_by_key_id(db, person_key.id()).await?.as_ref(), Some(&person));

        assert_eq!(Inbox::get(db, &person.inbox()).await.map(|i| i.actor())?, person.table_entry());
        assert_eq!(Outbox::get(db, &person.outbox()).await.map(|i| i.actor())?, person.table_entry());

        person.delete(db).await?;

        assert!(Person::get(db, &person_uuid).await.is_err());

        Ok(())
    }
}
