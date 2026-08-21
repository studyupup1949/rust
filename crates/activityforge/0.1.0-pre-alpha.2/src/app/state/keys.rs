use activitystreams_vocabulary::{Key as VocabPublicKey, Multikey};

use http::Method;

use crate::crypto::{HttpPublicKey, KeyType, PemPublicKey, PrivateKey};
use crate::db::{Iri, Key, TableType};
use crate::{Error, Result, util};

use super::AppState;

impl AppState {
    /// Create private key records for a local `Actor` entry.
    pub async fn create_keys<K, I>(
        &self,
        actor_id: &Iri,
        key_types: I,
    ) -> Result<(Vec<Multikey>, Option<VocabPublicKey>)>
    where
        K: Into<KeyType>,
        I: IntoIterator<Item = K>,
    {
        let mut key_types = key_types.into_iter().map(|i| i.into()).collect::<Vec<_>>();
        key_types.sort();
        key_types.dedup();

        if key_types.is_empty() {
            return Err(Error::http("create_keys: empty algorithms"));
        }

        let mut multikeys = Vec::with_capacity(key_types.len());
        let mut pemkey = None;

        let db = self.db().await;
        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        for key_type in key_types.into_iter() {
            let key_id = TableType::Key
                .id_from_uuid(self.uri(), util::rand_uuid())
                .map_err(|err| {
                    Error::http(format!("create_keys: error creating factory key ID: {err}"))
                })?;

            let mut privkey = PrivateKey::random(key_type)
                .and_then(Key::try_from)
                .map(|k| k.with_id(key_id).with_actor_id(actor_id))
                .map_err(|err| {
                    Error::http(format!("create_keys: error creating factory key: {err}"))
                })?;

            match key_type {
                KeyType::Ecdsa256 | KeyType::Ecdsa384 | KeyType::Ed25519 => {
                    let multikey = Multikey::try_from(&privkey).map_err(|err| {
                        Error::http(format!("create_keys: error creating multikey: {err}"))
                    })?;

                    multikeys.push(multikey);
                }
                KeyType::Rsa2048 => {
                    let key = VocabPublicKey::try_from(&privkey).map_err(|err| {
                        Error::http(format!("create_keys: error creating PEM key: {err}"))
                    })?;

                    pemkey = Some(key);
                }
                _ => {
                    return Err(Error::http(format!(
                        "create_Keys: invalid algorithm: {key_type}"
                    )));
                }
            }

            privkey
                .find_or_create_tx(&mut dbtx, &db_key)
                .await
                .map_err(|err| {
                    Error::db(format!(
                        "create_keys: error creating {key_type} key entry: {err}"
                    ))
                })?;
        }

        multikeys.shrink_to_fit();

        dbtx.commit()
            .await
            .map(|_| (multikeys, pemkey))
            .map_err(|err| Error::db(format!("create_keys: error commiting transaction: {err}")))
    }

    /// Attempts to fetch a public key from a remote server.
    ///
    /// Should only be called for a public key not currently in the database.
    ///
    /// Duplicate keys will return an error.
    pub async fn fetch_key(&self, key_id: &Iri) -> Result<HttpPublicKey> {
        log::debug!("fetch_key: looking up cached key by ID: {key_id}");

        let res = Key::find_by_key_id(&*self.db().await, key_id).await?;
        match res {
            Some(key) => {
                log::debug!("fetch_key: found key for ID: {key_id}");
                HttpPublicKey::try_from(key)
            }
            None => {
                log::debug!("fetch_key: no key found for ID: {key_id}");

                let res = self
                    .signed_request::<()>(Method::GET, key_id, None)
                    .await?
                    .text()
                    .await?;

                let mut key = serde_json::from_str::<Multikey>(&res)
                    .map_err(Error::from)
                    .and_then(Key::try_from)
                    .or_else(|_| {
                        serde_json::from_str::<PemPublicKey>(&res)
                            .map_err(Error::from)
                            .and_then(Key::try_from)
                    })
                    .map_err(|err| {
                        Error::crypto(format!(
                            "no valid public key found from remote server: {err}"
                        ))
                    })?;

                key.find_or_create(&*self.db().await).await?;

                HttpPublicKey::try_from(key)
            }
        }
    }
}
