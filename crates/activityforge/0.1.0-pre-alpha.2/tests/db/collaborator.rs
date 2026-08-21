use activityforge::db::{Collaborator, Iri, TableType};
use activityforge::{CollabRelationship, Role};

crate::db_test! {
    collaborator => run_tests(db) {
        let host = Iri::try_from("https://example.dev")?;

        let factory_uuid = db.rand_uuid();
        let subject = TableType::Factory.id_from_uuid(&host, factory_uuid)?;

        let person_uuid = db.rand_uuid();
        let object = TableType::Person.id_from_uuid(&host, person_uuid)?;

        let mut collaborator = Collaborator::new()
            .with_subject(subject)
            .with_relationship(CollabRelationship::HasCollaborator)
            .with_object(object)
            .with_tag(Role::Admin);

        let collaborator_uuid = collaborator.insert(db).await?;

        assert_eq!(
            Collaborator::get(db, &collaborator_uuid).await.as_ref(),
            Ok(&collaborator)
        );

        collaborator.set_relationship(CollabRelationship::HasMember);
        collaborator.update(db).await?;

        collaborator.set_tag(Role::Maintain);
        collaborator.update(db).await?;

        assert_eq!(
            Collaborator::get(db, &collaborator_uuid).await.as_ref(),
            Ok(&collaborator)
        );
        assert_eq!(collaborator.tag(), Role::Maintain);

        collaborator.delete(db).await?;

        assert!(Collaborator::get(db, &collaborator_uuid).await.is_err());

        Ok(())
    }
}
