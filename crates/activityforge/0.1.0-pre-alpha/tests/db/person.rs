use activitystreams_vocabulary::{Iri, Name};

use activityforge::db::actor::Person;
use activityforge::db::object::{Inbox, Key, Outbox};
use activityforge::db::{Db, KeyType, TableEntry, TableType};
use activityforge::{Error, Result};

/// Tests database operations on [Person] records.
pub async fn test_person(db: &Db) -> Result<()> {
    let person_id = Iri::try_from("https://example.dev/test_person")?;
    let person_uuid = db.rand_uuid();

    let person_inbox_id = Iri::try_from(format!("{person_id}/inbox"))?;

    let mut person_inbox = Inbox::new()
        .with_id(person_inbox_id)
        .with_actor(TableEntry::create(TableType::Person, person_uuid));

    let person_inbox_uuid = person_inbox
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting person inbox: {err}")))?;

    let person_outbox_id = Iri::try_from(format!("{person_id}/outbox"))?;

    let mut person_outbox = Outbox::new()
        .with_id(person_outbox_id)
        .with_actor(TableEntry::create(TableType::Person, person_uuid));

    let person_outbox_uuid = person_outbox
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting person outbox: {err}")))?;

    let person_name = Name::try_from("person")?;

    let mut person = Person::new()
        .with_uuid(person_uuid)
        .with_id(person_id.clone())
        .with_name(person_name)
        .with_inbox(person_inbox_uuid)
        .with_outbox(person_outbox_uuid);

    let person_uuid = person
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting person record: {err}")))?;

    let person_key_id = Iri::try_from(format!("{person_id}/keys#main-key"))?;
    let mut person_key_data = [0u8; 64];
    rand::fill(&mut person_key_data);

    let mut person_key = Key::new()
        .with_id(person_key_id)
        .with_key_type(KeyType::Ed25519)
        .with_key(person_key_data)
        .with_is_private(true)
        .with_actor(TableEntry::create(TableType::Person, person_uuid));

    let person_key_uuid = person_key
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting key record: {err}")))?;

    assert!(!person_key_uuid.is_nil());

    let ret_key = Key::get(db, &person_key_uuid)
        .await
        .map_err(|err| Error::sql(format!("error getting key record: {err}")))?;

    assert_eq!(ret_key, person_key);

    person.set_key_ids([person_key_uuid]);

    person
        .update(db)
        .await
        .map_err(|err| Error::sql(format!("error updating person record: {err}")))?;

    assert_eq!(Person::get(db, &person_uuid).await?, person);

    person
        .delete(db)
        .await
        .map_err(|err| Error::sql(format!("error deleting person record: {err}")))?;

    assert!(Person::get(db, &person_uuid).await.is_err());

    Ok(())
}
