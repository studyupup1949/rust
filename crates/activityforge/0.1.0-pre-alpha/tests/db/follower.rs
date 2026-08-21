use activitystreams_vocabulary::{Iri, Name};

use activityforge::db::actor::Person;
use activityforge::db::object::{Follower, Inbox, Outbox};
use activityforge::db::{Db, TableEntry, TableType};
use activityforge::{Error, Result};

/// Tests database operations on [Follower] records.
pub async fn test_follower(db: &Db) -> Result<()> {
    let followed_person_id = Iri::try_from("https://example.dev/test_followed_person")?;
    let followed_person_uuid = db.rand_uuid();

    let followed_person_inbox_id = Iri::try_from(format!("{followed_person_id}/inbox"))?;

    let mut followed_person_inbox = Inbox::new()
        .with_id(followed_person_inbox_id)
        .with_actor(TableEntry::create(TableType::Person, followed_person_uuid));

    let followed_person_inbox_uuid = followed_person_inbox
        .insert(&db)
        .await
        .map_err(|err| Error::sql(format!("error inserting followed_person inbox: {err}")))?;

    let followed_person_outbox_id = Iri::try_from(format!("{followed_person_id}/outbox"))?;

    let mut followed_person_outbox = Outbox::new()
        .with_id(followed_person_outbox_id)
        .with_actor(TableEntry::create(TableType::Person, followed_person_uuid));

    let followed_person_outbox_uuid = followed_person_outbox
        .insert(&db)
        .await
        .map_err(|err| Error::sql(format!("error inserting followed_person outbox: {err}")))?;

    let followed_person_name = Name::try_from("followed_person")?;

    let mut followed_person = Person::new()
        .with_uuid(followed_person_uuid)
        .with_id(followed_person_id.clone())
        .with_name(followed_person_name)
        .with_inbox(followed_person_inbox_uuid)
        .with_outbox(followed_person_outbox_uuid);

    let followed_person_uuid = followed_person
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting followed_person record: {err}")))?;

    let follower_person_id = Iri::try_from("https://example.dev/test_follower_person")?;
    let follower_person_uuid = db.rand_uuid();

    let follower_person_inbox_id = Iri::try_from(format!("{follower_person_id}/inbox"))?;

    let mut follower_person_inbox = Inbox::new()
        .with_id(follower_person_inbox_id)
        .with_actor(TableEntry::create(TableType::Person, follower_person_uuid));

    let follower_person_inbox_uuid = follower_person_inbox
        .insert(&db)
        .await
        .map_err(|err| Error::sql(format!("error inserting follower_person inbox: {err}")))?;

    let follower_person_outbox_id = Iri::try_from(format!("{follower_person_id}/outbox"))?;

    let mut follower_person_outbox = Outbox::new()
        .with_id(follower_person_outbox_id)
        .with_actor(TableEntry::create(TableType::Person, follower_person_uuid));

    let follower_person_outbox_uuid = follower_person_outbox
        .insert(&db)
        .await
        .map_err(|err| Error::sql(format!("error inserting follower_person outbox: {err}")))?;

    let follower_person_name = Name::try_from("follower_person")?;

    let mut follower_person = Person::new()
        .with_uuid(follower_person_uuid)
        .with_id(follower_person_id.clone())
        .with_name(follower_person_name)
        .with_inbox(follower_person_inbox_uuid)
        .with_outbox(follower_person_outbox_uuid);

    let follower_person_uuid = follower_person
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting follower_person record: {err}")))?;

    let mut follower = Follower::new()
        .with_actor(TableEntry::create(TableType::Person, follower_person_uuid))
        .with_following([TableEntry::create(TableType::Person, followed_person_uuid)])?;

    follower
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting follower record: {err}")))?;

    follower
        .delete(db)
        .await
        .map_err(|err| Error::sql(format!("error deleting follower record: {err}")))?;

    followed_person
        .delete(db)
        .await
        .map_err(|err| Error::sql(format!("error deleting followed_person record: {err}")))?;

    follower_person
        .delete(db)
        .await
        .map_err(|err| Error::sql(format!("error deleting follower_person record: {err}")))?;

    assert!(Person::get(db, &followed_person_uuid).await.is_err());

    Ok(())
}
