/// Helper macro to define common functionality for SQL record types.
#[macro_export]
macro_rules! impl_sql_record {
    ($ty:ident$(<$ty_gen:ident>)?) => {
        impl$(<$ty_gen>)? $ty$(<$ty_gen>)? {
            pub const fn table(&self) -> $crate::db::TableType {
                $crate::db::TableType::$ty
            }

            /// Attempts to delete the record from the database.
            pub async fn delete(self, db: &Db) -> Result<()> {
                let pool = db.pool()?;

                let mut dbtx = pool.begin().await?;

                self.delete_tx(&mut dbtx).await?;

                dbtx.commit().await.map(|_| ()).map_err(Error::from)
            }

            /// Attempts to delete the record from the database using a [Transaction].
            pub async fn delete_tx(self, dbtx: &mut Transaction<'_, Postgres>) -> Result<()> {
                let table = self.table();
                let uuid = self.uuid();

                util::check_uuid(table.as_str(), &uuid)?;

                sqlx::query(format!("DELETE FROM {table} WHERE uuid = $1").as_str())
                    .bind(uuid)
                    .execute(&mut **dbtx)
                    .await
                    .map(|_| ())
                    .map_err(|err| Error::sql(format!("{table}: error deleting record: {err}")))
            }
        }
    };
}

/// Helper function to implement SQL functions for field list types.
#[macro_export]
macro_rules! impl_sql_list_field {
    ($ty:ident {
        $(#[$field_meta:meta])*
        $field:ident: $sql_field:literal $field_ty:ident $(,)?
    }) => {
        $crate::paste! {
            $crate::impl_sql_list_field! {
                $ty {
                    $(#[$field_meta])*
                    $field, [<$field s>]: $sql_field $field_ty,
                }
            }
        }
    };

    ($ty:ident {
        $(#[$field_meta:meta])*
        $field:ident, $fields:ident: $sql_field:literal $field_ty:ident $(,)?
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
                        self.$fields = list;
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
                        $fields: list,
                        ..self
                    })
                }

                #[doc = "Attempts to add a [" $ty "] " $field " in the database."]
                pub async fn [<add_ $field>](&mut self, db: &Db, val: $field_ty) -> $crate::Result<()> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    self.[<add_ $field _tx>](&mut dbtx, val).await?;

                    dbtx.commit().await.map(|_| ()).map_err($crate::Error::from)
                }

                #[doc = "Attempts to add a [" $ty "] " $field " in the database using a SQL transaction."]
                pub async fn [<add_ $field _tx>](
                    &mut self,
                    dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
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
                pub async fn [<add_ $fields>]<T, I>(&mut self, db: &Db, val: I) -> $crate::Result<()>
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
                    dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
                    list: I,
                ) -> Result<()>
                    where T: Into<$field_ty>,
                          I: IntoIterator<Item = T>,
                {
                    let table = self.table();
                    let field = stringify!($fields);

                    $crate::util::check_uuid(table.as_str(), &self.uuid)?;

                    let mut list = $crate::util::dedup_list(format!("{table}: {field}").as_str(), list.into_iter().map(|i| i.into()).collect::<Vec<_>>())?;

                    for val in list.iter() {
                        if self.$fields.contains(val) {
                            return Err($crate::Error::sql(format!("{table}: duplicate {field}: {val}")));
                        }
                    }

                    self.$fields.append(&mut list);

                    sqlx::query(format!("UPDATE {table} SET {field} = $2 WHERE uuid = $1").as_str())
                        .bind(self.uuid)
                        .bind(self.$fields.as_slice())
                        .execute(&mut **dbtx)
                        .await
                        .map(|_| ())
                        .map_err(|err| $crate::Error::sql(format!("{table}: error adding {field}: {err}")))
                }

                #[doc = "Attempts to delete a [" $ty "] " $field " in the database."]
                pub async fn [<delete_ $field>](&mut self, db: &Db, val: $field_ty) -> $crate::Result<()> {
                    let pool = db.pool()?;

                    let mut dbtx = pool.begin().await?;

                    self.[<delete_ $field _tx>](&mut dbtx, val).await?;

                    dbtx.commit().await.map(|_| ()).map_err($crate::Error::from)
                }

                #[doc = "Attempts to delete a [" $ty "] " $field " in the database using a SQL transaction."]
                pub async fn [<delete_ $field _tx>](
                    &mut self,
                    dbtx: &mut sqlx::Transaction<'_, sqlx::postgres::Postgres>,
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
                            .bind(self.$fields.as_slice())
                            .execute(&mut **dbtx)
                            .await
                            .map(|_| ())
                            .map_err(|err| $crate::Error::sql(format!("{table}: error adding {field}: {err}")))
                    }
                }

                #[doc = "Attempts to delete [" $ty "] " $field " list items in the database."]
                pub async fn [<delete_ $fields>]<T, I>(&mut self, db: &Db, val: I) -> $crate::Result<Vec<$field_ty>>
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

                    let list = $crate::util::dedup_list(format!("{table}: {field}").as_str(), list.into_iter().map(|i| i.into()).collect::<Vec<_>>())?;

                    let deleted = self.$fields.extract_if(.., |f| list.contains(f)).collect::<Vec<_>>();

                    sqlx::query(format!("UPDATE {table} SET {field} = $2 WHERE uuid = $1").as_str())
                        .bind(self.uuid)
                        .bind(self.$fields.as_slice())
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
        $field:ident: $sql_field:literal $field_ty:ident $(,)?
    )+}) => {
        $crate::paste! {
            $crate::impl_sql_list_field! {
                $ty {
                   $($field, [<$field s>]: $sql_field $field_ty,)+
                }
            }
        }
    };

    ($ty:ident { $(
        $(#[$field_meta:meta])*
        $field:ident, $fields:ident: $sql_field:literal $field_ty:ident $(,)?
    )+ }) => {
        $(
            $crate::impl_sql_list_field! {
                $ty {
                    $(#[$field_meta])*
                    $field: $sql_field $field_ty,
                }
            }
        )+
    };
}
