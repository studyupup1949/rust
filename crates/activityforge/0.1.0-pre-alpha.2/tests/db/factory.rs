use activityforge::crypto::KeyType;
use activityforge::db::{ActorType, Collaborator, Factory, Iri, Key, Name, Person, TableType};
use activityforge::{CollabRelationship, Role};

crate::db_test! {
    factory => run_tests(db) {
        let host = Iri::try_from("https://example.dev")?;
        let admin_uuid = db.rand_uuid();
        let admin_id = TableType::Person.id_from_uuid(&host, admin_uuid)?;
        let admin_name = Name::try_from("factory_admin")?;

        let admin_key_uuid = db.rand_uuid();
        let admin_key_id = TableType::Key.id_from_uuid(&admin_id, admin_key_uuid)?;
        let mut admin_key_data = [0u8; 64];
        rand::fill(&mut admin_key_data);

        let mut admin_key = Key::new()
            .with_uuid(admin_key_uuid)
            .with_id(admin_key_id)
            .with_key_type(KeyType::Ed25519)
            .with_key(admin_key_data)
            .with_is_private(true);

        let admin = Person::builder(admin_id, admin_name)
            .and_then(|b| b.uuid(admin_uuid))
            .and_then(|b| b.keys([admin_key.clone()]))?
            .build(db)
            .await?;

        admin_key.set_actor_id(admin.id());
        admin_key.set_actor(admin.table_entry());

        assert_eq!(Key::get(db, &admin_key_uuid).await?, admin_key);

        let factory_uuid = db.rand_uuid();
        let factory_id = TableType::Factory.id_from_uuid(&host, factory_uuid)?;
        let factory_name = Name::try_from("default_factory")?;

        let collaborator_uuid = db.rand_uuid();
        let collaborator_id = TableType::Collaborator.id_from_uuid(&host, collaborator_uuid)?;
        let collaborator = Collaborator::new()
            .with_uuid(collaborator_uuid)
            .with_id(collaborator_id)
            .with_relationship(CollabRelationship::HasCollaborator)
            .with_object(admin.id())
            .with_tag(Role::Admin);

        let available_actor_types = [
            ActorType::Repository,
            ActorType::PatchTracker,
            ActorType::ReleaseTracker,
            ActorType::Roadmap,
            ActorType::TicketTracker,
            ActorType::Project,
            ActorType::Team,
            ActorType::Workflow,
        ];

        let factory_key_uuid = db.rand_uuid();
        let factory_key_id = TableType::Key.id_from_uuid(&factory_id, factory_key_uuid)?;
        let factory_key_type = KeyType::Ed25519;

        let mut factory_key_data = [0u8; 64];
        rand::fill(&mut factory_key_data);

        let mut factory_key = Key::new()
            .with_uuid(factory_key_uuid)
            .with_id(factory_key_id.clone())
            .with_key_type(factory_key_type)
            .with_key(factory_key_data)
            .with_is_private(true);

        let factory = Factory::builder(factory_id, factory_name)
            .and_then(|b| b.uuid(factory_uuid))
            .and_then(|b| b.keys([factory_key.clone()]))?
            .available_actor_types(available_actor_types)
            .collaborators([collaborator.clone()])
            .build(db)
            .await?;

        assert_eq!(factory.uuid(), factory_uuid);
        assert_eq!(factory.key_ids().first().unwrap(), &factory_key_uuid);

        assert!(factory.collaborators().contains(&collaborator_uuid));

        let ret_factory = Factory::get(db, &factory_uuid).await?;
        assert_eq!(ret_factory, factory);
        assert_eq!(Factory::find_by_key_id(db, factory_key.id()).await?.as_ref(), Some(&factory));

        factory_key.set_actor_id(factory.id());
        factory_key.set_actor(factory.table_entry());

        assert_eq!(Key::get(db, &factory_key_uuid).await?, factory_key);

        let ret_collaborator = Collaborator::get(db, &collaborator_uuid).await?;
        assert_eq!(
            ret_collaborator,
            collaborator.with_subject(factory.id())
        );

        Ok(())
    }
}
