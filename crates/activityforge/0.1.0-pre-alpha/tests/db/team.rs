use activitystreams_vocabulary::{Iri, Name};

use activityforge::db::actor::{Person, Team};
use activityforge::db::object::{Inbox, Key, Outbox};
use activityforge::db::{Db, KeyType, TableEntry, TableType};
use activityforge::{Error, Result};

/// Tests database operations on [Team] records.
pub async fn test_team(db: &Db) -> Result<()> {
    let person_id = Iri::try_from("https://example.dev/test_team_member")?;
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

    let person_name = Name::try_from("team_member")?;

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

    let team_uuid = db.rand_uuid();
    let team_name = Name::try_from("test_team")?;
    let team_id = Iri::try_from(format!("https://example.dev/{team_name}"))?;

    let team_inbox_id = Iri::try_from(format!("{team_id}/inbox"))?;

    let mut team_inbox = Inbox::new()
        .with_id(team_inbox_id)
        .with_actor(TableEntry::create(TableType::Person, team_uuid));

    let team_inbox_uuid = team_inbox
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting team inbox: {err}")))?;

    let team_outbox_id = Iri::try_from(format!("{team_id}/outbox"))?;

    let mut team_outbox = Outbox::new()
        .with_id(team_outbox_id)
        .with_actor(TableEntry::create(TableType::Person, team_uuid));

    let team_outbox_uuid = team_outbox
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting team outbox: {err}")))?;

    let mut team = Team::new()
        .with_uuid(team_uuid)
        .with_id(team_id.clone())
        .with_name(team_name)
        .with_inbox(team_inbox_uuid)
        .with_outbox(team_outbox_uuid)
        .with_members([person_uuid]);

    let team_uuid = team
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting team record: {err}")))?;

    assert_eq!(Team::get(db, &team_uuid).await?, team);

    let team_key_id = Iri::try_from(format!("{team_id}/keys#main-key"))?;
    let mut team_key_data = [0u8; 64];
    rand::fill(&mut team_key_data);

    let mut team_key = Key::new()
        .with_id(team_key_id)
        .with_key_type(KeyType::Ed25519)
        .with_key(team_key_data)
        .with_is_private(true)
        .with_actor(TableEntry::create(TableType::Person, team_uuid));

    let team_key_uuid = team_key
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting key record: {err}")))?;

    assert!(!team_key_uuid.is_nil());

    let ret_key = Key::get(db, &team_key_uuid)
        .await
        .map_err(|err| Error::sql(format!("error getting key record: {err}")))?;

    assert_eq!(ret_key, team_key);

    team.set_key_ids([team_key_uuid]);

    team.update(db)
        .await
        .map_err(|err| Error::sql(format!("error updating team record: {err}")))?;

    assert_eq!(Team::get(db, &team_uuid).await?, team);

    team.delete(db)
        .await
        .map_err(|err| Error::sql(format!("error deleting team record: {err}")))?;

    person
        .delete(db)
        .await
        .map_err(|err| Error::sql(format!("error deleting person record: {err}")))?;

    assert!(Person::get(db, &person_uuid).await.is_err());

    Ok(())
}
