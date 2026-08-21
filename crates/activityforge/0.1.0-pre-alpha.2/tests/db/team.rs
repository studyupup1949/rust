use activityforge::crypto::KeyType;
use activityforge::db::{Collaborator, Iri, Key, Name, Person, RoleFilter, TableType, Team};
use activityforge::{CollabRelationship, FilterKey, Role};

crate::db_test! {
    team => run_tests(db) {
        let host = Iri::try_from("https://example.dev")?;
        let person_uuid = db.rand_uuid();
        let person_id = TableType::Person.id_from_uuid(&host, person_uuid)?;

        let person_name = Name::try_from("team_member")?;

        let person_key_uuid = db.rand_uuid();
        let person_key_id = TableType::Key.id_from_uuid(&host, person_key_uuid)?;
        let mut person_key_data = [0u8; 64];
        rand::fill(&mut person_key_data);

        let mut person_key = Key::new()
            .with_uuid(person_key_uuid)
            .with_id(person_key_id)
            .with_key_type(KeyType::Ed25519)
            .with_key(person_key_data)
            .with_is_private(true);

        let person = Person::builder(person_id, person_name)
            .and_then(|p| p.uuid(person_uuid))
            .and_then(|p| p.keys([person_key.clone()]))?
            .build(db)
            .await?;

        person_key.set_actor_id(person.id());
        person_key.set_actor(person.table_entry());

        assert_eq!(Key::get(db, &person_key_uuid).await?, person_key);
        assert_eq!(Person::get(db, &person_uuid).await?, person);

        let team_uuid = db.rand_uuid();
        let team_name = Name::try_from("test_team")?;
        let team_id = TableType::Team.id_from_uuid(&host, team_uuid)?;

        let team_key_uuid = db.rand_uuid();
        let team_key_id = TableType::Key.id_from_uuid(&host, team_key_uuid)?;
        let mut team_key_data = [0u8; 64];
        rand::fill(&mut team_key_data);

        let mut team_key = Key::new()
            .with_uuid(team_key_uuid)
            .with_id(team_key_id)
            .with_key_type(KeyType::Ed25519)
            .with_key(team_key_data)
            .with_is_private(true);

        let member_uuid = db.rand_uuid();
        let member_id = TableType::Collaborator.id_from_uuid(&host, member_uuid)?;
        let member = Collaborator::new()
            .with_uuid(member_uuid)
            .with_id(member_id)
            .with_subject(&team_id)
            .with_relationship(CollabRelationship::HasMember)
            .with_object(person.id())
            .with_tag(Role::Maintain);

        let role_filters = [
            RoleFilter::create(FilterKey::Subteams, Role::Admin),
            RoleFilter::create(FilterKey::Oversees, Role::Admin),
            RoleFilter::create(FilterKey::OverseenBy, Role::Visit),
            RoleFilter::create(FilterKey::Parent, Role::Visit),
        ];

        let mut team = Team::builder(team_id, team_name)
            .and_then(|b| b.uuid(team_uuid))
            .and_then(|b| b.keys([team_key.clone()]))?
            .role_filters(role_filters)
            .members([member])
            .build(db)
            .await?;

        assert!(!team_key_uuid.is_nil());

        team_key.set_actor_id(team.id());
        team_key.set_actor(team.table_entry());

        assert_eq!(Key::get(db, &team_key_uuid).await?, team_key);

        assert_eq!(Team::get(db, &team_uuid).await?, team);
        assert_eq!(Team::find_by_key_id(db, team_key.id()).await?.as_ref(), Some(&team));

        let team_summary = "test team summary";
        team.set_summary(team_summary);
        team.update(db).await?;

        assert_eq!(team.summary(), Some(team_summary));

        let team_content = "test team content";
        team.set_content(team_content);
        team.update(db).await?;

        assert_eq!(team.content(), Some(team_content));

        assert_eq!(Team::get(db, &team_uuid).await?, team);

        team.delete(db).await?;
        team_key.delete(db).await?;

        person.delete(db).await?;
        person_key.delete(db).await?;

        assert!(Person::get(db, &person_uuid).await.is_err());
        assert!(Team::get(db, &team_uuid).await.is_err());
        assert!(Key::get(db, &team_key_uuid).await.is_err());
        assert!(Key::get(db, &person_key_uuid).await.is_err());

        Ok(())
    }
}
