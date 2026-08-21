use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use chacha20::{ChaCha20Poly1305, KeyInit, aead::Aead};
use serde::{Deserialize, Serialize};
use sqlx::{Postgres, Row, Transaction};
use zeroize::Zeroizing;

use crate::db::{Db, Iri, KeyType, Nonce, Salt, SymmetricKey, TableEntry, Uuid};
use crate::{Error, Result, impl_sql_record, util};

/// Represents an actor's key information record.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    #[serde(skip)]
    key: Zeroizing<Vec<u8>>,
    key_type: KeyType,
    is_private: bool,
    actor: TableEntry,
}

impl Key {
    /// Creates a new [Key].
    pub fn new() -> Self {
        Self {
            uuid: Uuid::nil(),
            id: Iri::new(),
            key: Zeroizing::new(Vec::new()),
            key_type: KeyType::new(),
            is_private: false,
            actor: TableEntry::new(),
        }
    }

    /// Gets a reference to the key data.
    ///
    /// If `is_private` is set to `true`, represents private key data.
    /// Otherwise, represents the public key data.
    pub fn key(&self) -> &[u8] {
        self.key.as_ref()
    }

    /// Sets a reference to the key data.
    ///
    /// If `is_private` is set to `true`, represents private key data.
    /// Otherwise, represents the public key data.
    pub fn set_key<I: Into<Vec<u8>>>(&mut self, val: I) {
        self.key = Zeroizing::new(val.into());
    }

    /// Builder function that sets a reference to the key data.
    ///
    /// If `is_private` is set to `true`, represents private key data.
    /// Otherwise, represents the public key data.
    pub fn with_key<I: Into<Vec<u8>>>(self, val: I) -> Self {
        Self {
            key: Zeroizing::new(val.into()),
            ..self
        }
    }

    /// Encrypts the key data for the database.
    fn encrypt_key(uuid: &Uuid, key: &[u8], db_key: &SymmetricKey) -> Result<Vec<u8>> {
        let salt = Salt::hash(uuid.as_bytes());
        let db_key = SymmetricKey::hash_with_salt(db_key.as_ref(), salt)?;
        let nonce = Nonce::hash(uuid.as_bytes());

        let cipher = ChaCha20Poly1305::new_from_slice(db_key.as_ref())?;

        cipher.encrypt(nonce.as_nonce(), key).map_err(Error::from)
    }

    /// Decrypts the key data from the database.
    fn decrypt_key(uuid: &Uuid, key: &[u8], db_key: &SymmetricKey) -> Result<Zeroizing<Vec<u8>>> {
        let nonce = Nonce::hash(uuid.as_bytes());
        let salt = Salt::hash(uuid.as_bytes());
        let db_key = SymmetricKey::hash_with_salt(db_key.as_ref(), salt)?;

        let cipher = ChaCha20Poly1305::new_from_slice(db_key.as_ref())?;

        cipher
            .decrypt(nonce.as_nonce(), key)
            .map(Zeroizing::new)
            .map_err(Error::from)
    }

    /// Attempts to get a [Key] record by [Uuid].
    pub async fn get(db: &Db, uuid: &Uuid) -> Result<Self> {
        let pool = db.pool()?;
        let key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let factory = Self::get_tx(&mut dbtx, uuid, &key).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| factory)
    }

    /// Attempts to get a [Key] record by [Uuid] using a DB transaction.
    ///
    /// - `db_key`: symmetric encryption key for encrypting key data
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use activityforge::db::{Db, Key};
    ///
    /// # async fn test() -> activityforge::Result<()> {
    /// let db = Db::new();
    ///
    /// let pool = db.pool()?;
    /// let mut dbtx = pool.begin().await?;
    ///
    /// let db_key = db.key()?;
    /// // dummy value, in production use the expected key UUID
    /// let uuid = db.rand_uuid();
    ///
    /// let _key = Key::get_tx(&mut dbtx, &uuid, &db_key).await?;
    ///
    /// dbtx.commit().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_tx(
        dbtx: &mut Transaction<'_, Postgres>,
        uuid: &Uuid,
        db_key: &SymmetricKey,
    ) -> Result<Self> {
        util::check_uuid("key", uuid)?;

        sqlx::query("SELECT * FROM key WHERE uuid = $1")
            .bind(uuid)
            .fetch_one(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|row| {
                Ok(Self {
                    uuid: *uuid,
                    id: row.try_get::<Iri, &str>("id")?,
                    key: row
                        .try_get::<Vec<u8>, &str>("key")
                        .map_err(Error::from)
                        .and_then(|k| Self::decrypt_key(uuid, k.as_slice(), db_key))?,
                    key_type: row.try_get::<KeyType, &str>("key_type")?,
                    is_private: row.try_get::<bool, &str>("is_private")?,
                    actor: row.try_get::<TableEntry, &str>("actor")?,
                })
            })
    }

    /// Attempts to insert a [Key] record into the database.
    pub async fn insert(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let db_key = db.key()?;

        let uuid = self.insert_tx(&mut dbtx, &db_key).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| uuid)
    }

    /// Attempts to insert a [Key] record into the database using a [Transaction].
    ///
    /// - `db_key`: symmetric encryption key for encrypting key data
    ///
    /// # Example
    ///
    /// ```rust,no_run
    /// use activityforge::db::{Db, Key};
    ///
    /// # async fn test() -> activityforge::Result<()> {
    /// let db = Db::new();
    ///
    /// let pool = db.pool()?;
    /// let mut dbtx = pool.begin().await?;
    ///
    /// let db_key = db.key()?;
    ///
    /// // dummy value, in production fill out the key data
    /// let mut key = Key::new();
    ///
    /// key.insert_tx(&mut dbtx, &db_key).await?;
    ///
    /// dbtx.commit().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn insert_tx(
        &mut self,
        dbtx: &mut Transaction<'_, Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<Uuid> {
        let uuid = if self.uuid.is_nil() {
            util::rand_uuid()
        } else {
            self.uuid
        };

        let key = Self::encrypt_key(&uuid, self.key.as_slice(), db_key)?;

        let row = sqlx::query(
            "INSERT INTO key
            (uuid, id, key, key_type, is_private, actor)
            values ($1, $2, $3, $4, $5, $6)
            RETURNING uuid",
        )
        .bind(uuid)
        .bind(self.id.as_str())
        .bind(key.as_slice())
        .bind(self.key_type)
        .bind(self.is_private)
        .bind(self.actor())
        .fetch_one(&mut **dbtx)
        .await?;

        let uuid = row.try_get::<Uuid, &str>("uuid")?;

        if self.uuid.is_nil() {
            self.uuid = uuid;
        }

        Ok(uuid)
    }
}

field_access! {
    Key {
        /// Represents the UUID primary key for the record.
        uuid: Uuid,
    }
}

field_access! {
    Key {
        /// Represents the IRI used to fetch the [Key] record.
        id: as_ref { Iri },
    }
}

field_access! {
    Key {
        /// Represents the cryptographic algorithm associated with the key material.
        key_type: KeyType,
        /// Represents whether the [Key] record is for a private key.
        is_private: bool,
        /// Represents the actor who owns the key.
        actor: TableEntry,
    }
}

impl_default!(Key);
impl_display!(Key, json);
impl_sql_record!(Key);
