use activityforge::db::{Follower, Iri, Name, Person, TableType};

crate::db_test! {
    follower => run_tests(db) {
        let host = Iri::try_from("https://example.dev")?;

        let followed_person_uuid = db.rand_uuid();
        let followed_person_id = TableType::Person.id_from_uuid(&host, followed_person_uuid)?;
        let followed_person_name = Name::try_from("followed_person")?;

        let followed_person = Person::builder(followed_person_id.clone(), followed_person_name)
            .and_then(|p| p.uuid(followed_person_uuid))?
            .build(db)
            .await?;

        let follower_person_uuid = db.rand_uuid();
        let follower_person_id = TableType::Person.id_from_uuid(&host, follower_person_uuid)?;
        let follower_person_name = Name::try_from("follower_person")?;

        let follower_person = Person::builder(follower_person_id, follower_person_name)
            .and_then(|p| p.uuid(follower_person_uuid))?
            .build(db)
            .await?;

        let follower_uuid = db.rand_uuid();
        let follower_id = TableType::Follower.id_from_uuid(&host, follower_uuid)?;
        let mut follower = Follower::new()
            .with_uuid(follower_uuid)
            .with_id(follower_id)
            .with_actor(follower_person.table_entry())
            .with_following([followed_person.table_entry()])?;

        follower.insert(db).await?;

        assert_eq!(Follower::get(db, &follower.uuid()).await?, follower);

        follower.delete(db).await?;
        followed_person.delete(db).await?;
        follower_person.delete(db).await?;

        assert!(Person::get(db, &followed_person_uuid).await.is_err());

        Ok(())
    }
}
