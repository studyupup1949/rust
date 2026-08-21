use activitystreams_vocabulary::{
    Iri as VocabIri, Key as VocabPublicKey, MultibaseHeader, MultibasePublicKey, Multikey,
    MultikeyPublicKey, PublicKeyPem, field_access, impl_default, impl_display,
};
use chacha20::{ChaCha20Poly1305, KeyInit, aead::Aead};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use zeroize::{Zeroize, ZeroizeOnDrop};

use sqlx::{Postgres, Row, Transaction};

use crate::crypto::{
    HttpPrivateKey, HttpPublicKey, KeyType, Nonce, PemPublicKey, PrivateKey, PublicKey, Salt,
    SymmetricKey,
};
use crate::db::{Db, Iri, TableEntry, Uuid};
use crate::{Error, Result, impl_sql_delete, impl_sql_table, util};

/// SQLx wrapper for zeroizing byte list.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Zeroize, ZeroizeOnDrop, sqlx::Type)]
#[sqlx(transparent)]
pub struct KeyBytes(Vec<u8>);

impl KeyBytes {
    /// Creates a new [KeyBytes].
    pub const fn new() -> Self {
        Self(Vec::new())
    }

    /// Gets a reference to the inner bytes.
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Converts bytes into [KeyBytes].
    pub fn from_bytes<I: IntoIterator<Item = u8>>(bytes: I) -> Self {
        Self(bytes.into_iter().collect())
    }

    /// Encrypts the key data for the database.
    pub fn encrypt(&self, uuid: &Uuid, db_key: &SymmetricKey) -> Result<Self> {
        let salt = Salt::hash(uuid.as_bytes());
        let db_key = SymmetricKey::hash_with_salt(db_key.as_ref(), salt)?;
        let nonce = Nonce::hash(uuid.as_bytes());

        let cipher = ChaCha20Poly1305::new_from_slice(db_key.as_ref())?;

        cipher
            .encrypt(&nonce.as_nonce(), self.as_bytes())
            .map(Self)
            .map_err(Error::from)
    }

    /// Decrypts the key data from the database.
    pub fn decrypt(&self, uuid: &Uuid, db_key: &SymmetricKey) -> Result<Self> {
        let nonce = Nonce::hash(uuid.as_bytes());
        let salt = Salt::hash(uuid.as_bytes());
        let db_key = SymmetricKey::hash_with_salt(db_key.as_ref(), salt)?;

        let cipher = ChaCha20Poly1305::new_from_slice(db_key.as_ref())?;

        cipher
            .decrypt(&nonce.as_nonce(), self.as_bytes())
            .map(Self)
            .map_err(Error::from)
    }
}

impl AsRef<[u8]> for KeyBytes {
    fn as_ref(&self) -> &[u8] {
        self.as_bytes()
    }
}

impl From<KeyBytes> for Vec<u8> {
    fn from(val: KeyBytes) -> Self {
        val.0.clone()
    }
}

impl From<&KeyBytes> for Vec<u8> {
    fn from(val: &KeyBytes) -> Self {
        val.as_bytes().to_vec()
    }
}

impl_default!(KeyBytes);

/// Represents an actor's key information record.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct Key {
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    uuid: Uuid,
    id: Iri,
    actor_id: Iri,
    #[serde(skip)]
    key: KeyBytes,
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
            actor_id: Iri::new(),
            key: KeyBytes::new(),
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
        self.key = KeyBytes::from_bytes(val.into());
    }

    /// Builder function that sets a reference to the key data.
    ///
    /// If `is_private` is set to `true`, represents private key data.
    /// Otherwise, represents the public key data.
    pub fn with_key<I: Into<Vec<u8>>>(self, val: I) -> Self {
        Self {
            key: KeyBytes::from_bytes(val.into()),
            ..self
        }
    }

    /// Encrypts the key data for storage.
    pub fn encrypt_key(&self, db_key: &SymmetricKey) -> Result<KeyBytes> {
        self.key.encrypt(&self.uuid, db_key)
    }

    /// Decrypts the key data from storage.
    pub fn decrypt_key(&self, db_key: &SymmetricKey) -> Result<KeyBytes> {
        self.key.decrypt(&self.uuid, db_key)
    }

    /// Attempts to get a [Key] record by [Uuid].
    pub async fn get(db: &Db, uuid: &Uuid) -> Result<Self> {
        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let key = Self::get_tx(&mut dbtx, uuid, &db_key).await?;

        dbtx.commit().await.map(|_| key).map_err(Error::from)
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
                Self::from_row(&row)
                    .map_err(Error::from)
                    .and_then(|k| k.decrypt_key(db_key).map(|key| k.with_key(key)))
            })
    }

    /// Attempts to find the [Key] record by key ID.
    ///
    /// If no record exists, returns `Ok(None)`.
    pub async fn find_by_key_id(db: &Db, key_id: &Iri) -> Result<Option<Self>> {
        log::debug!("find_by_key_id: looking up ID: {key_id}");

        let pool = db.pool()?;
        let db_key = db.key()?;

        let mut dbtx = pool.begin().await?;

        let key = Self::find_by_key_id_tx(&mut dbtx, key_id, &db_key).await?;

        dbtx.commit().await.map(|_| key).map_err(Error::from)
    }

    /// Attempts to find the [Key] record by key ID using a transaction.
    ///
    /// If no record exists, returns `Ok(None)`.
    pub async fn find_by_key_id_tx(
        dbtx: &mut Transaction<'_, Postgres>,
        key_id: &Iri,
        db_key: &SymmetricKey,
    ) -> Result<Option<Self>> {
        log::debug!("find_by_key_id_tx: looking up ID: {key_id}");

        sqlx::query("SELECT * FROM key WHERE id = $1")
            .bind(key_id)
            .fetch_optional(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|opt_row| {
                if let Some(row) = opt_row {
                    Self::from_row(&row)
                        .map_err(Error::from)
                        .and_then(|k| k.decrypt_key(db_key).map(|key| k.with_key(key)))
                        .map(Some)
                } else {
                    Ok(None)
                }
            })
    }

    /// Attempts to find a [Key] associated with the actor table entry.
    pub async fn find_by_actor(db: &Db, actor: TableEntry) -> Result<Vec<Self>> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let db_key = db.key()?;

        let keys = Self::find_by_actor_tx(&mut dbtx, &db_key, actor).await?;

        dbtx.commit().await.map(|_| keys).map_err(Error::from)
    }

    /// Attempts to find a [Key] associated with the actor table entry using a transaciton.
    pub async fn find_by_actor_tx(
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
        actor: TableEntry,
    ) -> Result<Vec<Self>> {
        let table = Self::TABLE;

        ::sqlx::query(format!(r#"SELECT * FROM {table} WHERE actor = $1"#).as_str())
            .bind(actor)
            .fetch_all(&mut **dbtx)
            .await
            .map_err(Error::from)
            .and_then(|rows| {
                rows.into_iter()
                    .map(|row| {
                        Self::from_row(&row)
                            .map_err(Error::from)
                            .and_then(|k| k.decrypt_key(db_key).map(|key| k.with_key(key)))
                    })
                    .collect::<Result<Vec<Self>>>()
            })
    }

    /// Attempts to find a [Key] associated with the actor table entry.
    ///
    /// Filters the results by specific key type.
    pub async fn find_by_actor_and_type(
        db: &Db,
        actor: TableEntry,
        key_type: KeyType,
    ) -> Result<Option<Self>> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let db_key = db.key()?;

        let key = Self::find_by_actor_and_type_tx(&mut dbtx, &db_key, actor, key_type).await?;

        dbtx.commit().await.map(|_| key).map_err(Error::from)
    }

    /// Attempts to find a [Key] associated with the actor table entry using a transaction.
    ///
    /// Filters the results by specific key type.
    pub async fn find_by_actor_and_type_tx(
        dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
        db_key: &SymmetricKey,
        actor: TableEntry,
        key_type: KeyType,
    ) -> Result<Option<Self>> {
        let table = Self::TABLE;

        ::sqlx::query(
            format!(r#"SELECT * FROM {table} WHERE actor = $1 AND key_type = $2"#).as_str(),
        )
        .bind(actor)
        .bind(key_type)
        .fetch_optional(&mut **dbtx)
        .await
        .map_err(Error::from)
        .and_then(|row| {
            if let Some(row) = row {
                Self::from_row(&row)
                    .map_err(Error::from)
                    .and_then(|k| k.decrypt_key(db_key).map(|key| k.with_key(key)))
                    .map(Some)
            } else {
                Ok(None)
            }
        })
    }

    /// Attempts to insert a [Key] record into the database.
    pub async fn insert(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let db_key = db.key()?;

        let uuid = self.insert_tx(&mut dbtx, &db_key).await?;

        dbtx.commit().await.map(|_| uuid).map_err(Error::from)
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
        if self.uuid.is_nil() {
            self.uuid = util::rand_uuid();
        }

        let key = self.encrypt_key(db_key)?;

        let row = sqlx::query(
            "INSERT INTO key
            (uuid, id, actor_id, key, key_type, is_private, actor)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING uuid",
        )
        .bind(self.uuid())
        .bind(self.id())
        .bind(self.actor_id())
        .bind(key)
        .bind(self.key_type())
        .bind(self.is_private())
        .bind(self.actor())
        .fetch_one(&mut **dbtx)
        .await?;

        let uuid = row.try_get::<Uuid, &str>("uuid")?;

        if self.uuid.is_nil() {
            self.uuid = uuid;
        }

        Ok(uuid)
    }

    /// Attempts to insert or update a [Key] record into the database.
    pub async fn insert_or_update(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let db_key = db.key()?;

        let uuid = self.insert_or_update_tx(&mut dbtx, &db_key).await?;

        dbtx.commit().await.map_err(Error::from).map(|_| uuid)
    }

    /// Attempts to insert or update a [Key] record into the database using a [Transaction].
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
    /// key.insert_or_update_tx(&mut dbtx, &db_key).await?;
    ///
    /// dbtx.commit().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn insert_or_update_tx(
        &mut self,
        dbtx: &mut Transaction<'_, Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<Uuid> {
        if self.uuid.is_nil() {
            self.uuid = Self::TABLE.uuid_from_id(self.id());
        }

        let key = self.encrypt_key(db_key)?;

        let fields = "(uuid, id, actor_id, key, key_type, is_private, actor)";
        let places = "($1, $2, $3, $4, $5, $6, $7)";

        sqlx::query(
            format!(
                r#"
            INSERT INTO key {fields} VALUES {places}
                ON CONFLICT (id) DO UPDATE SET {fields} = {places}
            "#
            )
            .as_str(),
        )
        .bind(self.uuid())
        .bind(self.id())
        .bind(self.actor_id())
        .bind(key)
        .bind(self.key_type())
        .bind(self.is_private())
        .bind(self.actor())
        .execute(&mut **dbtx)
        .await
        .map(|_| self.uuid)
        .map_err(Error::from)
    }

    /// Attempts to find or create a [Key] record in the database.
    pub async fn find_or_create(&mut self, db: &Db) -> Result<Uuid> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let db_key = db.key()?;

        let uuid = self.find_or_create_tx(&mut dbtx, &db_key).await?;

        dbtx.commit().await.map(|_| uuid).map_err(Error::from)
    }

    /// Attempts to find or create a [Key] record in the database using a [Transaction].
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
    /// key.find_or_create_tx(&mut dbtx, &db_key).await?;
    ///
    /// dbtx.commit().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn find_or_create_tx(
        &mut self,
        dbtx: &mut Transaction<'_, Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<Uuid> {
        if let Some(key) = Self::find_by_key_id_tx(dbtx, self.id(), db_key).await? {
            *self = key;
            Ok(self.uuid())
        } else {
            if self.uuid.is_nil() {
                self.uuid = Self::TABLE.uuid_from_id(self.id());
            }

            let key = self.encrypt_key(db_key)?;

            let fields = "(uuid, id, actor_id, key, key_type, is_private, actor)";
            let places = "($1, $2, $3, $4, $5, $6, $7)";

            sqlx::query(format!("INSERT INTO key {fields} VALUES {places} RETURNING uuid").as_str())
                .bind(self.uuid())
                .bind(self.id())
                .bind(self.actor_id())
                .bind(key)
                .bind(self.key_type())
                .bind(self.is_private())
                .bind(self.actor())
                .execute(&mut **dbtx)
                .await
                .map(|_| self.uuid)
                .map_err(Error::from)
        }
    }

    /// Attempts to update a [Key] record into the database.
    pub async fn update(&mut self, db: &Db) -> Result<()> {
        let pool = db.pool()?;

        let mut dbtx = pool.begin().await?;

        let db_key = db.key()?;

        self.update_tx(&mut dbtx, &db_key).await?;

        dbtx.commit().await.map(|_| ()).map_err(Error::from)
    }

    /// Attempts to insert or update a [Key] record into the database using a [Transaction].
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
    /// key.update_tx(&mut dbtx, &db_key).await?;
    ///
    /// dbtx.commit().await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn update_tx(
        &mut self,
        dbtx: &mut Transaction<'_, Postgres>,
        db_key: &SymmetricKey,
    ) -> Result<()> {
        util::check_uuid("key", &self.uuid)?;

        let key = self.encrypt_key(db_key)?;

        let fields = "(id, actor_id, key, key_type, is_private, actor)";
        let places = "($2, $3, $4, $5, $6, $7)";

        sqlx::query(format!(r#"UPDATE SET {fields} = {places} WHERE uuid = $1"#).as_str())
            .bind(self.uuid)
            .bind(self.id())
            .bind(self.actor_id())
            .bind(key)
            .bind(self.key_type())
            .bind(self.is_private())
            .bind(self.actor())
            .execute(&mut **dbtx)
            .await
            .map(|_| ())
            .map_err(Error::from)
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
        /// Represents the IRI used to fetch the [Key] actor's record.
        actor_id: as_ref { Iri },
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

impl_sql_delete!(Key);
impl_sql_table!(Key);

impl_default!(Key);
impl_display!(Key, json);

impl TryFrom<PemPublicKey> for Key {
    type Error = Error;

    fn try_from(val: PemPublicKey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&PemPublicKey> for Key {
    type Error = Error;

    fn try_from(val: &PemPublicKey) -> Result<Self> {
        let key = val.public_key_pem();

        key.to_bytes().map(|k| {
            Self::new()
                .with_id(val.id())
                .with_actor_id(val.owner())
                .with_key_type(key.algorithm())
                .with_key(k)
                .with_is_private(false)
        })
    }
}

impl TryFrom<Key> for PemPublicKey {
    type Error = Error;

    fn try_from(val: Key) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Key> for PemPublicKey {
    type Error = Error;

    fn try_from(val: &Key) -> Result<Self> {
        if val.is_private() {
            PrivateKey::from_bytes(val.key_type(), val.key()).map(|p| {
                Self::new()
                    .with_id(val.id())
                    .with_owner(val.actor_id())
                    .with_public_key_pem(p.public_key())
            })
        } else {
            PublicKey::from_bytes(val.key_type(), val.key()).map(|k| {
                Self::new()
                    .with_id(val.id())
                    .with_owner(val.actor_id())
                    .with_public_key_pem(k)
            })
        }
    }
}

impl TryFrom<Key> for VocabPublicKey {
    type Error = Error;

    fn try_from(val: Key) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Key> for VocabPublicKey {
    type Error = Error;

    fn try_from(val: &Key) -> Result<Self> {
        let id = VocabIri::from(val.id());
        let owner = VocabIri::from(val.actor_id());

        let key_type = val.key_type();

        let public_key_der = if val.is_private() {
            PrivateKey::from_bytes(key_type, val.key()).and_then(|p| p.public_key().to_der())
        } else {
            PublicKey::from_bytes(key_type, val.key()).and_then(|p| p.to_der())
        }?;

        let public_key = PublicKeyPem::new().with_key(public_key_der);

        Ok(Self::new()
            .with_id(id)
            .with_owner(owner)
            .with_public_key_pem(public_key))
    }
}

impl TryFrom<Multikey> for Key {
    type Error = Error;

    fn try_from(val: Multikey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Multikey> for Key {
    type Error = Error;

    fn try_from(val: &Multikey) -> Result<Self> {
        match val.public_key_multibase().key() {
            MultikeyPublicKey::Ecdsa256(key) => Ok(Self::new()
                .with_id(val.id())
                .with_actor_id(val.controller())
                .with_key_type(KeyType::Ecdsa256)
                .with_key(key)
                .with_is_private(false)),
            MultikeyPublicKey::Ecdsa384(key) => Ok(Self::new()
                .with_id(val.id())
                .with_actor_id(val.controller())
                .with_key_type(KeyType::Ecdsa384)
                .with_key(key)
                .with_is_private(false)),
            MultikeyPublicKey::Ed25519(key) => Ok(Self::new()
                .with_id(val.id())
                .with_actor_id(val.controller())
                .with_key_type(KeyType::Ed25519)
                .with_key(key)
                .with_is_private(false)),
            _ => Err(Error::crypto("unsupported key type")),
        }
    }
}

impl TryFrom<Key> for Multikey {
    type Error = Error;

    fn try_from(val: Key) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Key> for Multikey {
    type Error = Error;

    fn try_from(val: &Key) -> Result<Self> {
        let key_type = val.key_type();

        let pubkey = if val.is_private() {
            PrivateKey::from_bytes(key_type, val.key()).map(|k| k.public_key())?
        } else {
            PublicKey::from_bytes(key_type, val.key())?
        };

        let multikey_pubkey = match key_type {
            KeyType::Ecdsa256 => pubkey.to_ecdsa256_bytes().map(MultikeyPublicKey::Ecdsa256),
            KeyType::Ecdsa384 => pubkey.to_ecdsa384_bytes().map(MultikeyPublicKey::Ecdsa384),
            KeyType::Ed25519 => pubkey.to_ed25519_bytes().map(MultikeyPublicKey::Ed25519),
            _ => Err(Error::crypto(format!(
                "key: unsupported multikey algorithm: {key_type}"
            ))),
        }?;

        let multibase = MultibasePublicKey::new()
            .with_header(MultibaseHeader::Base64UrlNoPad)
            .with_key(multikey_pubkey);

        Ok(Multikey::new()
            .with_id(val.id())
            .with_controller(val.actor_id())
            .with_public_key_multibase(multibase))
    }
}

impl TryFrom<HttpPublicKey> for Key {
    type Error = Error;

    fn try_from(val: HttpPublicKey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&HttpPublicKey> for Key {
    type Error = Error;

    fn try_from(val: &HttpPublicKey) -> Result<Self> {
        Key::try_from(val.key()).and_then(|k| Iri::try_from(val.key_id()).map(|id| k.with_id(id)))
    }
}

impl TryFrom<Key> for HttpPublicKey {
    type Error = Error;

    fn try_from(val: Key) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Key> for HttpPublicKey {
    type Error = Error;

    fn try_from(val: &Key) -> Result<Self> {
        PublicKey::try_from(val).map(|k| Self::new(val.id().as_str(), k))
    }
}

impl TryFrom<Key> for HttpPrivateKey {
    type Error = Error;

    fn try_from(val: Key) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Key> for HttpPrivateKey {
    type Error = Error;

    fn try_from(val: &Key) -> Result<Self> {
        PrivateKey::try_from(val).map(|k| Self::new(val.id().as_str(), k))
    }
}

impl TryFrom<PublicKey> for Key {
    type Error = Error;

    fn try_from(val: PublicKey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&PublicKey> for Key {
    type Error = Error;

    fn try_from(val: &PublicKey) -> Result<Self> {
        val.to_bytes()
            .map(|k| Key::new().with_key_type(val.algorithm()).with_key(k))
    }
}

impl TryFrom<Key> for PublicKey {
    type Error = Error;

    fn try_from(val: Key) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Key> for PublicKey {
    type Error = Error;

    fn try_from(val: &Key) -> Result<Self> {
        if val.is_private() {
            PrivateKey::try_from(val).map(|k| k.public_key())
        } else {
            PublicKey::from_bytes(val.key_type(), val.key())
        }
    }
}

impl TryFrom<Key> for PrivateKey {
    type Error = Error;

    fn try_from(val: Key) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&Key> for PrivateKey {
    type Error = Error;

    fn try_from(val: &Key) -> Result<Self> {
        if val.is_private() {
            PrivateKey::from_bytes(val.key_type(), val.key())
        } else {
            Err(Error::db("key: key is not a private key"))
        }
    }
}

impl TryFrom<PrivateKey> for Key {
    type Error = Error;

    fn try_from(val: PrivateKey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&PrivateKey> for Key {
    type Error = Error;

    fn try_from(val: &PrivateKey) -> Result<Self> {
        val.to_bytes().map(|k| {
            Self::new()
                .with_is_private(true)
                .with_key(k)
                .with_key_type(val.algorithm())
        })
    }
}

impl TryFrom<HttpPrivateKey> for Key {
    type Error = Error;

    fn try_from(val: HttpPrivateKey) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&HttpPrivateKey> for Key {
    type Error = Error;

    fn try_from(val: &HttpPrivateKey) -> Result<Self> {
        Self::try_from(val.key()).and_then(|k| Iri::try_from(val.key_id()).map(|id| k.with_id(id)))
    }
}

impl TryFrom<jwt::jwk::Jwk> for Key {
    type Error = Error;

    fn try_from(val: jwt::jwk::Jwk) -> Result<Self> {
        (&val).try_into()
    }
}

impl TryFrom<&jwt::jwk::Jwk> for Key {
    type Error = Error;

    fn try_from(val: &jwt::jwk::Jwk) -> Result<Self> {
        PublicKey::try_from(val)
            .and_then(Self::try_from)
            .map(|key| {
                if let Some(id) = val
                    .common
                    .key_id
                    .as_deref()
                    .and_then(|id| Iri::try_from(id).ok())
                {
                    key.with_id(id)
                } else {
                    key
                }
            })
    }
}
