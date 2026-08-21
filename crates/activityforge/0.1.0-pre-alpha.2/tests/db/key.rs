use activitystreams_vocabulary::Iri;

use activityforge::crypto::KeyType;
use activityforge::db::{Key, TableEntry, TableType};

crate::db_test! {
    key => run_tests(db) {
        let actor_uuid = db.rand_uuid();
        let key_uuid = db.rand_uuid();
        let key_id = Iri::try_from(format!("https://example.dev/api/v1/persons/{actor_uuid}/keys/{key_uuid}"))?;

        let mut key_data = [0u8; 64];
        rand::fill(&mut key_data);

        let mut key = Key::new()
            .with_uuid(key_uuid)
            .with_id(key_id)
            .with_key_type(KeyType::Ed25519)
            .with_key(key_data)
            .with_is_private(true)
            .with_actor(TableEntry::create(TableType::Person, actor_uuid));

        assert_eq!(key.key(), key_data.as_ref());

        key.insert(db).await?;

        assert!(!key_uuid.is_nil());

        assert_eq!(Key::get(db, &key_uuid).await?, key);

        key.delete(db).await?;

        assert!(Key::get(db, &key_uuid).await.is_err());

        Ok(())
    }
}
