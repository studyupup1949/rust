use activitystreams_vocabulary::{Iri, Name};

use activityforge::db::actor::{Factory, Person};
use activityforge::db::object::{Collaborator, Inbox, Key, Outbox};
use activityforge::db::{ActorType, Db, KeyType, TableEntry, TableType};
use activityforge::{Error, Result, Role};

/// Tests database operations on [Factory] records.
pub async fn test_factory(db: &Db) -> Result<()> {
    let factory_uuid = db.rand_uuid();
    let factory_id =
        Iri::try_from(format!("https://example.dev/factories/{factory_uuid}")).unwrap();

    let factory_name = Name::try_from("default_factory").unwrap();

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

    let factory_inbox_id = Iri::try_from(format!("{factory_id}/inbox"))?;

    let mut factory_inbox = Inbox::new()
        .with_id(factory_inbox_id)
        .with_actor(TableEntry::create(TableType::Factory, factory_uuid));

    let factory_inbox_uuid = factory_inbox.insert(&db).await?;

    let factory_outbox_id = Iri::try_from(format!("{factory_id}/outbox"))?;

    let mut factory_outbox = Outbox::new()
        .with_id(factory_outbox_id)
        .with_actor(TableEntry::create(TableType::Factory, factory_uuid));

    let factory_outbox_uuid = factory_outbox.insert(&db).await?;

    let admin_id = Iri::try_from("https://example.dev/admin")?;
    let admin_uuid = db.rand_uuid();

    let admin_inbox_id = Iri::try_from(format!("{admin_id}/inbox"))?;

    let mut admin_inbox = Inbox::new()
        .with_id(admin_inbox_id)
        .with_actor(TableEntry::create(TableType::Person, admin_uuid));

    let admin_inbox_uuid = admin_inbox
        .insert(&db)
        .await
        .map_err(|err| Error::sql(format!("error inserting admin inbox: {err}")))?;

    let admin_outbox_id = Iri::try_from(format!("{admin_id}/outbox"))?;

    let mut admin_outbox = Outbox::new()
        .with_id(admin_outbox_id)
        .with_actor(TableEntry::create(TableType::Person, admin_uuid));

    let admin_outbox_uuid = admin_outbox
        .insert(&db)
        .await
        .map_err(|err| Error::sql(format!("error inserting admin outbox: {err}")))?;

    let admin_name = Name::try_from("admin")?;
    let mut admin = Person::new()
        .with_uuid(admin_uuid)
        .with_id(admin_id.clone())
        .with_name(admin_name)
        .with_inbox(admin_inbox_uuid)
        .with_outbox(admin_outbox_uuid);
    admin
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting admin record: {err}")))?;

    let admin_key_id = Iri::try_from(format!("{admin_id}/keys#main-key"))?;
    let mut admin_key_data = [0u8; 64];
    rand::fill(&mut admin_key_data);

    let mut admin_key = Key::new()
        .with_id(admin_key_id)
        .with_key_type(KeyType::Ed25519)
        .with_key(admin_key_data)
        .with_is_private(true)
        .with_actor(TableEntry::create(TableType::Person, admin_uuid));

    let admin_key_uuid = admin_key
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting key record: {err}")))?;

    assert!(!admin_key_uuid.is_nil());

    let ret_key = Key::get(db, &admin_key_uuid)
        .await
        .map_err(|err| Error::sql(format!("error getting key record: {err}")))?;

    assert_eq!(ret_key, admin_key);

    let mut collaborator = Collaborator::new()
        .with_subject(TableEntry::create(TableType::Factory, factory_uuid))
        .with_object(TableEntry::create(TableType::Person, admin_uuid))
        .with_instrument(Role::Admin);

    let collaborator_uuid = collaborator
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting collaborator record: {err}")))?;

    let factory = Factory::new()
        .with_uuid(factory_uuid)
        .with_id(factory_id.clone())
        .with_name(factory_name)
        .with_available_actor_types(available_actor_types)
        .with_inbox(factory_inbox_uuid)
        .with_outbox(factory_outbox_uuid)
        .with_collaborators([collaborator_uuid]);

    let insert_factory_uuid = factory
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting factory record: {err}")))?;

    assert_eq!(insert_factory_uuid, factory_uuid);

    let ret_factory = Factory::get(db, &factory_uuid)
        .await
        .map_err(|err| Error::sql(format!("error getting factory record: {err}")))?;

    assert_eq!(ret_factory, factory);

    let factory_key_id = Iri::try_from(format!("{factory_id}/keys#main-key"))?;
    let mut factory_key_data = [0u8; 64];
    rand::fill(&mut factory_key_data);

    let mut factory_key = Key::new()
        .with_id(factory_key_id)
        .with_key_type(KeyType::Ed25519)
        .with_key(factory_key_data)
        .with_is_private(true)
        .with_actor(TableEntry::create(TableType::Factory, factory_uuid));

    let factory_key_uuid = factory_key
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting key record: {err}")))?;

    assert!(!factory_key_uuid.is_nil());

    let ret_key = Key::get(db, &factory_key_uuid)
        .await
        .map_err(|err| Error::sql(format!("error getting key record: {err}")))?;

    assert_eq!(ret_key, factory_key.with_uuid(factory_key_uuid));

    Ok(())
}
