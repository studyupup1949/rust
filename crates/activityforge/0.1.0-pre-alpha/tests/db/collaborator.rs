use activityforge::db::object::Collaborator;
use activityforge::db::{Db, TableEntry, TableType};
use activityforge::{Result, Role};

/// Tests database operations on [Collaborator] records.
pub async fn test_collaborator(db: &Db) -> Result<()> {
    let mut collaborator = Collaborator::new()
        .with_subject(TableEntry::create(TableType::Factory, db.rand_uuid()))
        .with_object(TableEntry::create(TableType::Person, db.rand_uuid()))
        .with_instrument(Role::Admin);

    let collaborator_uuid = collaborator.insert(db).await?;

    assert_eq!(
        Collaborator::get(db, &collaborator_uuid).await.as_ref(),
        Ok(&collaborator)
    );

    collaborator.set_instrument(Role::Delegate);

    collaborator.update(db).await?;

    assert_eq!(
        Collaborator::get(db, &collaborator_uuid).await.as_ref(),
        Ok(&collaborator)
    );
    assert_eq!(collaborator.instrument(), Role::Delegate);

    collaborator.delete(db).await?;

    assert!(Collaborator::get(db, &collaborator_uuid).await.is_err());

    Ok(())
}
