/// Helper macro to define the SQL table name.
#[macro_export]
macro_rules! impl_sql_table {
    ($ty:ident$(<$ty_gen:ident>)?) => {
        $crate::paste! {
            impl$(<$ty_gen:ident>)? $ty$(<$ty_gen>)? {
                #[doc = "Represents the [" $ty "] table type."]
                pub const TABLE: $crate::db::TableType = $crate::db::TableType::$ty;

                #[doc = "Gets [" $ty "] the table type."]
                pub const fn table(&self) -> $crate::db::TableType {
                    Self::TABLE
                }

                #[doc = "Gets the table entry for the [" $ty "]."]
                #[inline]
                pub fn table_entry(&self) -> $crate::db::TableEntry {
                    $crate::db::TableEntry::create(Self::TABLE, self.uuid)
                }
            }
        }
    };
}

/// Helper macro to define common functionality for SQL record types.
#[macro_export]
macro_rules! impl_sql_record {
    (
        $(#[$record_meta:meta])?
        $ty:ident$(<$ty_gen:ident>)? {
            $($field:ident: { $sql_field:literal $field_ty:ident },)+
        }
    ) => {
        $crate::paste! {
            $crate::impl_sql_table!($ty$(<$ty_gen>)?);

            impl$(<$ty_gen:ident>)? $ty$(<$ty_gen>)? {
                #[doc = "Attempts to get a [" $ty "] record by UUID."]
                pub async fn get(db: &$crate::db::Db, uuid: &$crate::db::Uuid) -> Result<Self> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    let factory = Self::get_tx(&mut dbtx, uuid).await?;

                    dbtx.commit().await.map_err(Error::from).map(|_| factory)
                }

                #[doc = "Attempts to get a [" $ty "] record by UUID using a DB transaction."]
                #[allow(clippy::needless_update)]
                pub async fn get_tx(
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    uuid: &$crate::db::Uuid,
                ) -> $crate::Result<Self> {
                    use ::sqlx::Row;

                    let table = Self::TABLE;

                    ::sqlx::query(format!("SELECT * FROM {table} WHERE uuid = $1").as_str())
                        .bind(uuid)
                        .fetch_one(&mut **dbtx)
                        .await
                        .map_err(Error::from)
                        .and_then(|row| {
                            Ok(Self {
                                uuid: *uuid,
                                $($field: row.try_get::<$field_ty, &str>($sql_field)?,)+
                                ..Default::default()
                            })
                        })
                }

                #[doc = "Attempts to insert a [" $ty "] record into the database."]
                pub async fn insert(&mut self, db: &$crate::db::Db) -> $crate::Result<$crate::db::Uuid> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    let uuid = self.insert_tx(&mut dbtx).await?;

                    dbtx.commit().await.map_err($crate::Error::from).map(|_| uuid)
                }

                #[doc = "Attempts to insert a [" $ty "] record into the database using a transaction."]
                pub async fn insert_tx(
                    &mut self,
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                ) -> $crate::Result<$crate::db::Uuid> {
                    use ::sqlx::Row;

                    self.check_db()?;

                    let table = self.table();
                    let uuid = if self.uuid.is_nil() {
                        util::rand_uuid()
                    } else {
                        self.uuid
                    };

                    let (places, fields): (Vec<String>, Vec<&str>) = ["uuid"]
                        .into_iter()
                        .chain([$(stringify!($sql_field),)+])
                        .enumerate()
                        .map(|(i, field)| (format!("${}", i + 1), field))
                        .unzip();

                    let fields = format!("({})", fields.join(", "));
                    let places = format!("({})", places.join(", "));

                    let query = format!("INSERT INTO {table} {fields} VALUES {places} RETURNING uuid");
                    log::debug!("factory: insert query: {query}");

                    let row = sqlx::query(query.as_str())
                    .bind(uuid)
                    $(.bind(self.$field()))+
                    .fetch_one(&mut **dbtx)
                    .await?;

                    let uuid = row.try_get::<$crate::db::Uuid, &str>("uuid")?;

                    if self.uuid.is_nil() {
                        self.uuid = uuid;
                    }

                    Ok(uuid)
                }

                #[doc = "Attempts to insert or update a [" $ty "] record into the database."]
                #[doc = ""]
                #[doc = "If an entry with the same UUID already exists, the fields of the entry are updated instead."]
                pub async fn insert_or_update(&mut self, db: &$crate::db::Db) -> $crate::Result<Uuid> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    let uuid = self.insert_or_update_tx(&mut dbtx).await?;

                    dbtx.commit().await.map(|_| uuid).map_err(Error::from)
                }

                #[doc = "Attempts to insert or update a [" $ty "] record into the database using a transaction."]
                #[doc = ""]
                #[doc = "If an entry with the same UUID already exists, the fields of the entry are updated instead."]
                pub async fn insert_or_update_tx(
                    &mut self,
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                ) -> $crate::Result<Uuid> {
                    self.check_db()?;

                    let table = self.table();
                    let uuid = if self.uuid().is_nil() {
                        $crate::util::rand_uuid()
                    } else {
                        self.uuid()
                    };

                    let (places, fields): (Vec<String>, Vec<&str>) = ["uuid"]
                        .into_iter()
                        .chain([$(stringify!($sql_field),)+])
                        .enumerate()
                        .map(|(i, field)| (format!("${}", i + 1), field))
                        .unzip();

                    let fields = format!("({})", fields.join(", "));
                    let places = format!("({})", places.join(", "));

                    // attempt to insert a new row into the table.
                    // if a row with the same UUID already exists, update the row instead
                    ::sqlx::query(format!(r#"
                        INSERT INTO {table} {fields} VALUES {places}
                            ON CONFLICT (uuid) DO UPDATE SET {fields} = {places}
                    "#).as_str())
                        .bind(uuid)
                        $(.bind(self.$field()))+
                        .execute(&mut **dbtx)
                        .await?;

                    if self.uuid.is_nil() {
                        self.uuid = uuid;
                    }

                    Ok(uuid)
                }

                #[doc = "Attempts to update a [" $ty "] record in the database."]
                pub async fn update(&self, db: &$crate::db::Db) -> $crate::Result<()> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    self.update_tx(&mut dbtx).await?;

                    dbtx.commit().await.map(|_| ()).map_err($crate::Error::from)
                }

                #[doc = "Attempts to update a [" $ty "] record in the database using a transaction."]
                pub async fn update_tx(&self, dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>) -> $crate::Result<()> {
                    self.check_db()?;

                    let table = self.table();

                    let (places, fields): (Vec<String>, Vec<&str>) = [
                        $(stringify!($sql_field),)+
                    ].into_iter().enumerate().map(|(i, field)| {
                        (format!("${}", i + 2), field)
                    }).unzip();

                    let fields = format!("({})", fields.join(", "));
                    let places = format!("({})", places.join(", "));

                    sqlx::query(format!("UPDATE {table} SET {fields} = {places} WHERE uuid = $1").as_str())
                        .bind(self.uuid)
                        $(.bind(self.$field()))+
                        .execute(&mut **dbtx)
                        .await
                        .map(|_| ())
                        .map_err(|err| $crate::Error::sql(format!("{table}: error updating record: {err}")))
                }

                #[doc = "Attempts to find a [" $ty "] by ID."]
                pub async fn find(
                    db: &$crate::db::Db,
                    id: &$crate::db::Uuid,
                ) -> $crate::Result<Option<Self>> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    let actor = Self::find_tx(&mut dbtx, id).await?;

                    dbtx
                        .commit()
                        .await
                        .map(|_| actor)
                        .map_err(|err| $crate::Error::db(format!("{}: {err}", Self::TABLE)))
                }

                #[doc = "Attempts to find a [" $ty "] by ID using a transaction."]
                pub async fn find_tx(
                    dbtx: &mut $crate::db::Transaction<'_>,
                    id: &$crate::db::Uuid,
                ) -> $crate::Result<Option<Self>> {
                    let table = Self::TABLE;
                    ::sqlx::query(format!("SELECT * FROM {table} WHERE uuid = $1").as_str())
                        .bind(id)
                        .fetch_optional(&mut **dbtx)
                        .await
                        .map_err(Error::from)
                        .and_then(|row| {
                            if let Some(row) = row {
                                Self::from_row(&row)
                                    .map(Some)
                                    .map_err(|err| Error::db(format!("{table}: {err}")))
                            } else {
                                Ok(None)
                            }
                        })
                }
            }
        }

        $crate::impl_sql_delete!($ty$(<$ty_gen>)?);
    };
}

/// Helper macro to define SQL find-or-create function for record types.
#[macro_export]
macro_rules! impl_sql_find_or_create {
    (
        $ty:ident$(<$ty_gen:ident>)?
    ) => {
        $crate::impl_sql_find_or_create!($ty$(<$ty_gen:ident>)?: id);
    };

    (
        $ty:ident$(<$ty_gen:ident>)?: $field:ident
    ) => {
        $crate::paste! {
            impl$(<$ty_gen:ident>)? $ty$(<$ty_gen>)? {
                #[doc = "Attempts to find the [" $ty "] by ID."]
                pub async fn find_or_create(&mut self, db: &$crate::db::Db) -> $crate::Result<$crate::db::Uuid> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    let uuid = self.find_or_create_tx(&mut dbtx).await?;

                    dbtx.commit().await.map(|_| uuid).map_err($crate::Error::from)
                }

                #[doc = "Attempts to find the [" $ty "] by ID using a transaction."]
                pub async fn find_or_create_tx(
                    &mut self,
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                ) -> $crate::Result<$crate::db::Uuid> {
                    use ::sqlx::FromRow;

                    let table = Self::TABLE;
                    let field = stringify!($field);

                    if let Some(row) = ::sqlx::query(format!("SELECT * FROM {table} WHERE {field} = $1").as_str())
                        .bind(self.$field())
                        .fetch_optional(&mut **dbtx)
                        .await
                        .map_err($crate::Error::from)? {
                            Self::from_row(&row)
                                .map(|s| {
                                    *self = s;
                                    self.uuid
                                })
                                .map_err(Error::from)
                    } else {
                        self.insert_or_update_tx(dbtx).await
                    }
                }
            }
        }
    };
}

/// Helper macro to implement a method for looking up actor SQL records by key ID.
#[macro_export]
macro_rules! impl_sql_find_by_key_id {
    ($ty:ident$(<$ty_gen:ident>)? {
        $($field:ident: { $sql_field:literal $field_ty:ident },)+
    }) => {
        $crate::paste! {
            impl$(<$ty_gen>)? $ty$(<$ty_gen>)? {
                #[doc = "Attempts to fetch a [" $ty "] by key ID."]
                pub async fn find_by_key_id(db: &$crate::db::Db, key_id: &$crate::db::Iri) -> Result<Option<Self>> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    let rec = Self::find_by_key_id_tx(&mut dbtx, key_id).await?;

                    dbtx.commit().await.map(|_| rec).map_err($crate::Error::from)
                }

                #[doc = "Attempts to fetch a [" $ty "] by key ID using a transaction."]
                #[allow(clippy::needless_update)]
                pub async fn find_by_key_id_tx(
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    key_id: &$crate::db::Iri,
                ) -> Result<Option<Self>> {
                    use ::sqlx::Row;

                    let table = Self::TABLE;

                    ::log::debug!("{table}: looking up record for key ID: {key_id}");

                    ::sqlx::query(
                        format!(r#"
                            SELECT * FROM {table}
                                WHERE EXISTS (
                                    SELECT 1 FROM key
                                    WHERE uuid = ANY ({table}.key_ids)
                                    AND actor_id = {table}.id
                                    AND id = $1
                                )
                        "#)
                        .as_str()
                    )
                        .bind(key_id)
                        .fetch_optional(&mut **dbtx)
                        .await
                        .map_err(Error::from)
                        .and_then(|row| {
                            if let Some(row) = row {
                                Ok(Some(Self {
                                    uuid: row.try_get::<$crate::db::Uuid, &str>("uuid")?,
                                    $($field: row.try_get::<$field_ty, &str>($sql_field)?,)+
                                    ..Default::default()
                                }))
                            } else {
                                Ok(None)
                            }
                        })
                }
            }
        }
    };
}

/// Helper macro to implement activity database record methods.
#[macro_export]
macro_rules! impl_sql_activity {
    ($ty:ident$(<$ty_gen:ident>)? {
        $($field:ident: { $sql_field:literal $field_ty:ident },)+
    }) => {
        $crate::impl_sql_record! {
            $ty$(<$ty_gen>)? {
                $($field: { $sql_field $field_ty },)+
            }
        }

        $crate::impl_sql_find_or_create!($ty$(<$ty_gen>)?);
    };
}

/// Helper macro to implement actor database record methods.
#[macro_export]
macro_rules! impl_sql_actor {
    ($ty:ident$(<$ty_gen:ident>)? {
        $($field:ident: { $sql_field:literal $field_ty:ident },)+
    }) => {
        $crate::impl_sql_record! {
            $ty$(<$ty_gen>)? {
                $($field: { $sql_field $field_ty },)+
            }
        }

        $crate::impl_sql_find_or_create!($ty$(<$ty_gen>)?);

        $crate::impl_sql_find_by_key_id! {
            $ty$(<$ty_gen>)? {
                $($field: { $sql_field $field_ty },)+
            }
        }

        $crate::paste! {
            impl$(<$ty_gen>)? $ty$(<$ty_gen>)? {
                #[doc = "Attempts to find a [" $ty "] by ID."]
                pub async fn find_by_id(
                    db: &$crate::db::Db,
                    id: &$crate::db::Iri,
                ) -> $crate::Result<Option<Self>> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    let actor = Self::find_by_id_tx(&mut dbtx, id).await?;

                    dbtx.commit().await.map(|_| actor).map_err($crate::Error::from)
                }

                #[doc = "Attempts to find a [" $ty "] by ID using a transaction."]
                pub async fn find_by_id_tx(
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    id: &$crate::db::Iri,
                ) -> $crate::Result<Option<Self>> {
                    let table = Self::TABLE;
                    ::sqlx::query(format!("SELECT * FROM {table} WHERE id = $1").as_str())
                        .bind(id)
                        .fetch_optional(&mut **dbtx)
                        .await
                        .map_err(Error::from)
                        .and_then(|row| {
                            if let Some(row) = row {
                                Self::from_row(&row).map(Some).map_err(Error::from)
                            } else {
                                Ok(None)
                            }
                        })
                }

                #[doc = "Attempts to find a [" $ty "] by name."]
                pub async fn find_by_name(
                    db: &$crate::db::Db,
                    name: &$crate::db::Name,
                ) -> $crate::Result<Option<Self>> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    let actor = Self::find_by_name_tx(&mut dbtx, name).await?;

                    dbtx.commit().await.map(|_| actor).map_err($crate::Error::from)
                }

                #[doc = "Attempts to find a [" $ty "] by name using a transaction."]
                pub async fn find_by_name_tx(
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    name: &$crate::db::Name,
                ) -> $crate::Result<Option<Self>> {
                    let table = Self::TABLE;
                    ::sqlx::query(format!("SELECT * FROM {table} WHERE name = $1").as_str())
                        .bind(name)
                        .fetch_optional(&mut **dbtx)
                        .await
                        .map_err(Error::from)
                        .and_then(|row| {
                            if let Some(row) = row {
                                Self::from_row(&row).map(Some).map_err(Error::from)
                            } else {
                                Ok(None)
                            }
                        })
                }

                #[doc = "Adds a [Key](crate::db::Key) to [" $ty "]."]
                pub async fn add_key(
                    &mut self,
                    db: &$crate::db::Db,
                    key: $crate::db::Key,
                ) -> $crate::Result<$crate::db::Uuid> {
                    let pool = db.pool()?;
                    let mut dbtx = pool.begin().await?;

                    let db_key = db.key()?;
                    let uuid = self.add_key_tx(&mut dbtx, &db_key, key).await?;

                    dbtx.commit().await.map(|_| uuid).map_err($crate::Error::from)
                }

                #[doc = "Adds a [Key](crate::db::Key) to [" $ty "] using a transaction."]
                pub async fn add_key_tx(
                    &mut self,
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    db_key: &$crate::crypto::SymmetricKey,
                    mut key: $crate::db::Key,
                ) -> $crate::Result<$crate::db::Uuid> {
                    key.set_actor_id(self.id());
                    key.set_actor(self.table_entry());

                    let uuid = key.insert_tx(dbtx, db_key).await?;

                    self.add_key_id_tx(dbtx, uuid).await.map(|_| uuid)
                }

                #[doc = "Gets all the [Key](crate::db::Key)s associated with [" $ty "]."]
                pub async fn keys(&self, db: &$crate::db::Db) -> $crate::Result<Vec<$crate::db::Key>> {
                    let pool = db.pool()?;
                    let mut dbtx = pool.begin().await?;

                    let db_key = db.key()?;
                    let keys = self.keys_tx(&mut dbtx, &db_key).await?;

                    dbtx.commit().await.map(|_| keys).map_err($crate::Error::from)
                }

                #[doc = "Gets all the [Key](crate::db::Key)s associated with [" $ty "] using a transaction."]
                pub async fn keys_tx(
                    &self,
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    db_key: &$crate::crypto::SymmetricKey,
                ) -> $crate::Result<Vec<$crate::db::Key>> {
                    $crate::db::Key::find_by_actor_tx(dbtx, db_key, self.table_entry()).await
                }

                #[doc = "Attempts to find a [" $ty "] [Key](crate::db::Key) by key type."]
                pub async fn find_key_by_type(
                    &self,
                    db: &$crate::db::Db,
                    key_type: $crate::crypto::KeyType,
                ) -> $crate::Result<Option<$crate::db::Key>> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    let db_key = db.key()?;

                    let key = self.find_key_by_type_tx(&mut dbtx, &db_key, key_type).await?;

                    dbtx.commit().await.map(|_| key).map_err($crate::Error::from)
                }

                #[doc = "Attempts to find a [" $ty "] [Key](crate::db::Key) by key type using a transaction."]
                pub async fn find_key_by_type_tx(
                    &self,
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    db_key: &$crate::crypto::SymmetricKey,
                    key_type: $crate::crypto::KeyType,
                ) -> $crate::Result<Option<$crate::db::Key>> {
                    $crate::db::Key::find_by_actor_and_type_tx(dbtx, db_key, self.table_entry(), key_type).await
                }
            }
        }
    };
}

/// Helper macro to implement actor mailbox database record methods.
#[macro_export]
macro_rules! impl_sql_mailbox {
    ($ty:ident$(<$ty_gen:ident>)? {
        $($field:ident: { $sql_field:literal $field_ty:ident },)+
    }) => {
        $crate::impl_sql_record! {
            $ty$(<$ty_gen>)? {
                $($field: { $sql_field $field_ty },)+
            }
        }

        $crate::impl_sql_find_or_create!($ty$(<$ty_gen>)?);

        $crate::paste! {
            impl$(<$ty_gen>)? $ty$(<$ty_gen>)? {
                #[doc = "Finds a [" $ty "] record by actor [TableEntry](crate::db::TableEntry)."]
                pub async fn find_by_actor(
                    db: &$crate::db::Db,
                    actor: &$crate::db::TableEntry,
                ) -> $crate::Result<Option<Self>> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    let mailbox = Self::find_by_actor_tx(&mut dbtx, actor).await?;

                    dbtx
                        .commit()
                        .await
                        .map(|_| mailbox)
                        .map_err(|err| $crate::Error::db(format!("find_by_actor: {err}")))
                }

                #[doc = "Finds a [" $ty "] record by actor [TableEntry](crate::db::TableEntry) using a transaction."]
                pub async fn find_by_actor_tx(
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    actor: &$crate::db::TableEntry,
                ) -> $crate::Result<Option<Self>> {
                    let table = Self::TABLE;

                    ::sqlx::query(format!("SELECT * FROM {table} WHERE actor = $1").as_str())
                        .bind(actor)
                        .fetch_optional(&mut **dbtx)
                        .await
                        .map_err(|err| Error::db(format!("find_by_actor: {err}")))
                        .and_then(|row| {
                            if let Some(row) = row {
                                Self::from_row(&row)
                                    .map(Some)
                                    .map_err(|err| $crate::Error::db(format!("find_by_actor: {err}")))
                            } else {
                                Ok(None)
                            }
                        })
                }
            }
        }
    };
}

/// Helper macro to implement object database record methods.
#[macro_export]
macro_rules! impl_sql_object {
    ($ty:ident$(<$ty_gen:ident>)? {
        $($field:ident: { $sql_field:literal $field_ty:ident },)+
    }) => {
        $crate::impl_sql_record! {
            $ty$(<$ty_gen>)? {
                $($field: { $sql_field $field_ty },)+
            }
        }

        $crate::impl_sql_find_or_create!($ty$(<$ty_gen>)?);
    };
}

/// Helper macro to implement a method for deleting a SQL record from the database.
#[macro_export]
macro_rules! impl_sql_delete {
    ($ty:ident$(<$ty_gen:ident>)?) => {
        $crate::paste! {
            impl$(<$ty_gen>)? $ty$(<$ty_gen>)? {
                /// Attempts to delete the record from the database.
                pub async fn delete(self, db: &$crate::db::Db) -> $crate::Result<()> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    self.delete_tx(&mut dbtx).await?;

                    dbtx.commit().await.map(|_| ()).map_err($crate::Error::from)
                }

                /// Attempts to delete the record from the database using a transaction.
                pub async fn delete_tx(self, dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>) -> $crate::Result<()> {
                    let table = self.table();
                    let uuid = self.uuid();

                    $crate::util::check_uuid(table.as_str(), &uuid)?;

                    ::sqlx::query(format!("DELETE FROM {table} WHERE uuid = $1").as_str())
                        .bind(uuid)
                        .execute(&mut **dbtx)
                        .await
                        .map(|_| ())
                        .map_err(|err| $crate::Error::sql(format!("{table}: error deleting record: {err}")))
                }
            }
        }
    }
}

/// Helper function to implement SQL functions for field list types.
#[macro_export]
macro_rules! impl_sql_list_field {
    ($ty:ident {
        $(#[$field_meta:meta])*
        $field:ident: { $sql_field:literal $field_ty:ident }$(,)?
    }) => {
        $crate::paste! {
            $crate::impl_sql_list_field! {
                $ty {
                    $(#[$field_meta])*
                    $field, [<$field s>]: { $sql_field $field_ty },
                }
            }
        }
    };

    ($ty:ident {
        $(#[$field_meta:meta])*
        $field:ident, $fields:ident: { $sql_field:literal $field_ty:ident } $(,)?
    }) => {
        impl $ty {
            $crate::paste! {
                #[doc = "Gets a reference to the " $field " list."]
                $(
                #[doc = ""]
                #[$field_meta]
                )*
                pub fn $fields(&self) -> &[$field_ty] {
                    self.$fields.as_slice()
                }

                #[doc = "Sets the " $field " list."]
                $(
                #[doc = ""]
                #[$field_meta]
                )*
                pub fn [<set_ $fields>]<T, I>(&mut self, list: I) -> $crate::Result<()>
                    where T: Into<$field_ty>,
                          I: IntoIterator<Item = T>,
                {
                    let table = self.table();
                    let field = stringify!($fields);

                    $crate::util::dedup_list(
                        format!("{table}: {field}").as_str(),
                        list.into_iter().map(|i| i.into()).collect::<Vec<_>>(),
                    ).map(|list| {
                        self.$fields = list.into();
                    })
                }

                #[doc = "Builder function that sets the " $field " list."]
                $(
                #[doc = ""]
                #[$field_meta]
                )*
                pub fn [<with_ $fields>]<T, I>(self, list: I) -> $crate::Result<Self>
                    where T: Into<$field_ty>,
                          I: IntoIterator<Item = T>,
                {
                    let table = self.table();
                    let field = stringify!($fields);

                    $crate::util::dedup_list(
                        format!("{table}: {field}").as_str(),
                        list.into_iter().map(|i| i.into()).collect::<Vec<_>>(),
                    ).map(|list| Self {
                        $fields: list.into(),
                        ..self
                    })
                }

                #[doc = "Attempts to add a [" $ty "] " $field " in the database."]
                pub async fn [<add_ $field>](&mut self, db: &$crate::db::Db, val: $field_ty) -> $crate::Result<()> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    self.[<add_ $field _tx>](&mut dbtx, val).await?;

                    dbtx.commit().await.map(|_| ()).map_err($crate::Error::from)
                }

                #[doc = "Attempts to add a [" $ty "] " $field " in the database using a SQL transaction."]
                pub async fn [<add_ $field _tx>](
                    &mut self,
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    val: $field_ty,
                ) -> $crate::Result<()> {
                    let table = self.table();
                    let field = stringify!($fields);

                    $crate::util::check_uuid(table.as_str(), &self.uuid)?;

                    if self.$fields.contains(&val) {
                        return Err($crate::Error::sql(format!("{table}: duplicate {field}: {val}")));
                    }

                    self.$fields.push(val);

                    sqlx::query(format!("UPDATE {table} SET {field} = $2 WHERE uuid = $1").as_str())
                        .bind(self.uuid)
                        .bind(self.$fields.as_slice())
                        .execute(&mut **dbtx)
                        .await
                        .map(|_| ())
                        .map_err(|err| $crate::Error::sql(format!("{table}: error adding {field}: {err}")))
                }

                #[doc = "Attempts to add [" $ty "] " $field " list items in the database."]
                pub async fn [<add_ $fields>]<T, I>(&mut self, db: &$crate::db::Db, val: I) -> $crate::Result<()>
                    where T: Into<$field_ty>,
                          I: IntoIterator<Item = T>,
                {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    self.[<add_ $fields _tx>](&mut dbtx, val).await?;

                    dbtx.commit().await.map(|_| ()).map_err($crate::Error::from)
                }

                #[doc = "Attempts to add [" $ty "] " $field " list items in the database using a SQL transaction."]
                pub async fn [<add_ $fields _tx>]<T, I>(
                    &mut self,
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    list: I,
                ) -> Result<()>
                    where T: Into<$field_ty>,
                          I: IntoIterator<Item = T>,
                {
                    let table = self.table();
                    let field = stringify!($fields);

                    $crate::util::check_uuid(table.as_str(), &self.uuid)?;

                    let mut list = $crate::util::dedup_list(
                        format!("{table}: {field}").as_str(),
                        list.into_iter().map(|i| i.into()).collect::<Vec<_>>(),
                    )?;

                    for val in list.iter() {
                        if self.$fields.contains(val) {
                            return Err($crate::Error::sql(format!("{table}: duplicate {field}: {val}")));
                        }
                    }

                    self.$fields.append(&mut list);

                    sqlx::query(format!("UPDATE {table} SET {field} = $2 WHERE uuid = $1").as_str())
                        .bind(self.uuid)
                        .bind(self.$fields())
                        .execute(&mut **dbtx)
                        .await
                        .map(|_| ())
                        .map_err(|err| $crate::Error::sql(format!("{table}: error adding {field}: {err}")))
                }

                #[doc = "Attempts to update [" $ty "] " $field " list items in the database."]
                #[doc = ""]
                #[doc = "Fully resets the " $field " list to the provided value."]
                pub async fn [<update_ $fields>]<T, I>(
                    &mut self,
                    db: &$crate::db::Db,
                    list: I,
                ) -> $crate::Result<()>
                    where T: Into<$field_ty>,
                          I: IntoIterator<Item = T>,
                {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    self.[<update_ $fields _tx>](&mut dbtx, list).await?;

                    dbtx.commit().await.map(|_| ()).map_err($crate::Error::from)
                }

                #[doc = "Attempts to update [" $ty "] " $field " list items in the database using a SQL transaction."]
                #[doc = ""]
                #[doc = "Fully resets the " $field " list to the provided value."]
                pub async fn [<update_ $fields _tx>]<T, I>(
                    &mut self,
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    list: I,
                ) -> Result<()>
                    where T: Into<$field_ty>,
                          I: IntoIterator<Item = T>,
                {
                    let table = self.table();
                    let field = stringify!($sql_field);

                    $crate::util::check_uuid(table.as_str(), &self.uuid)?;

                    let fields = $crate::util::dedup_list(
                        format!("{table}: {field}").as_str(),
                        list.into_iter().map(|i| i.into()).collect::<Vec<_>>(),
                    )?;

                    sqlx::query(format!("UPDATE {table} SET {field} = $2 WHERE uuid = $1").as_str())
                        .bind(self.uuid)
                        .bind(fields.as_slice())
                        .execute(&mut **dbtx)
                        .await
                        .map_err(|err| $crate::Error::sql(format!("{table}: error updating {field}: {err}")))
                        .and_then(|_| self.[<set_ $fields>](fields))
                }

                #[doc = "Attempts to delete a [" $ty "] " $field " in the database."]
                pub async fn [<delete_ $field>](&mut self, db: &$crate::db::Db, val: $field_ty) -> $crate::Result<()> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    self.[<delete_ $field _tx>](&mut dbtx, val).await?;

                    dbtx.commit().await.map(|_| ()).map_err($crate::Error::from)
                }

                #[doc = "Attempts to delete a [" $ty "] " $field " in the database using a SQL transaction."]
                pub async fn [<delete_ $field _tx>](
                    &mut self,
                    dbtx: &mut ::sqlx::Transaction<'_, ::sqlx::postgres::Postgres>,
                    val: $field_ty,
                ) -> $crate::Result<()> {
                    let table = self.table();
                    let field = stringify!($fields);

                    $crate::util::check_uuid(table.as_str(), &self.uuid)?;

                    if self.$fields.extract_if(.., |f| f == &val).count() == 0 {
                        log::debug!("{table}: no {field} records deleted");
                        Ok(())
                    } else {
                        sqlx::query(format!("UPDATE {table} SET {field} = $2 WHERE uuid = $1").as_str())
                            .bind(self.uuid)
                            .bind(self.$fields())
                            .execute(&mut **dbtx)
                            .await
                            .map(|_| ())
                            .map_err(|err| $crate::Error::sql(format!("{table}: error adding {field}: {err}")))
                    }
                }

                #[doc = "Attempts to delete [" $ty "] " $field " list items in the database."]
                pub async fn [<delete_ $fields>]<T, I>(&mut self, db: &$crate::db::Db, val: I) -> $crate::Result<Vec<$field_ty>>
                    where T: Into<$field_ty>,
                          I: IntoIterator<Item = T>,
                {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    let deleted = self.[<delete_ $fields _tx>](&mut dbtx, val).await?;

                    dbtx.commit().await.map(|_| deleted).map_err($crate::Error::from)
                }

                #[doc = "Attempts to delete [" $ty "] " $field " list items in the database using a SQL transaction."]
                pub async fn [<delete_ $fields _tx>]<T, I>(
                    &mut self,
                    dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
                    list: I,
                ) -> Result<Vec<$field_ty>>
                    where T: Into<$field_ty>,
                          I: IntoIterator<Item = T>,
                {
                    let table = self.table();
                    let field = stringify!($fields);

                    $crate::util::check_uuid(table.as_str(), &self.uuid)?;

                    let list = $crate::util::dedup_list(
                        format!("{table}: {field}").as_str(),
                        list.into_iter().map(|i| i.into()).collect::<Vec<_>>(),
                    )?;

                    let deleted = self.$fields.extract_if(.., |f| list.contains(f)).collect::<Vec<_>>();

                    sqlx::query(format!("UPDATE {table} SET {field} = $2 WHERE uuid = $1").as_str())
                        .bind(self.uuid)
                        .bind(self.$fields())
                        .execute(&mut **dbtx)
                        .await
                        .map(|_| deleted)
                        .map_err(|err| $crate::Error::sql(format!("{table}: error adding {field}: {err}")))
                }
            }
        }
    };

    ($ty:ident { $(
        $(#[$field_meta:meta])*
        $field:ident: { $sql_field:literal $field_ty:ident } $(,)?
    )+}) => {
        $crate::paste! {
            $crate::impl_sql_list_field! {
                $ty {
                   $($field, [<$field s>]: { $sql_field $field_ty },)+
                }
            }
        }
    };

    ($ty:ident { $(
        $(#[$field_meta:meta])*
        $field:ident, $fields:ident: { $sql_field:literal $field_ty:ident } $(,)?
    )+ }) => {
        $(
            $crate::impl_sql_list_field! {
                $ty {
                    $(#[$field_meta])*
                    $field: { $sql_field $field_ty },
                }
            }
        )+
    };
}
