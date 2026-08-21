use activitystreams_vocabulary::Iri;

use activityforge::db::object::Key;
use activityforge::db::{Db, KeyType, TableEntry, TableType};
use activityforge::{Error, Result};

/// Tests database operations on [Key] records.
pub async fn test_key(db: &Db) -> Result<()> {
    let key_id = Iri::try_from("https://example.dev/test_key#main-key")?;
    let mut key_data = [0u8; 64];
    rand::fill(&mut key_data);

    let mut key = Key::new()
        .with_id(key_id)
        .with_key_type(KeyType::Ed25519)
        .with_key(key_data)
        .with_is_private(true)
        .with_actor(TableEntry::create(TableType::Person, db.rand_uuid()));

    assert_eq!(key.key(), key_data.as_ref());

    let key_uuid = key
        .insert(db)
        .await
        .map_err(|err| Error::sql(format!("error inserting key record: {err}")))?;

    assert!(!key_uuid.is_nil());

    let ret_key = Key::get(db, &key_uuid)
        .await
        .map_err(|err| Error::sql(format!("error getting key record: {err}")))?;

    assert_eq!(ret_key, key);

    key.delete(db)
        .await
        .map_err(|err| Error::sql(format!("error deleting key: {err}")))?;

    assert!(Key::get(db, &key_uuid).await.is_err());

    Ok(())
}
