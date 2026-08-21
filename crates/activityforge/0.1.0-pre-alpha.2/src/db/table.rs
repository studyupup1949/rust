use activitystreams_vocabulary::{field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};

use crate::db::{TableType, Uuid};
use crate::{Error, Result, util};

/// Represents a table entry reference.
#[derive(
    Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Deserialize, Serialize, sqlx::Type,
)]
#[sqlx(type_name = "table_entry")]
pub struct TableEntry {
    entry_type: TableType,
    #[serde(serialize_with = "util::ser_uuid", deserialize_with = "util::de_uuid")]
    id: Uuid,
}

impl TableEntry {
    /// Creates a new [TableEntry].
    pub const fn new() -> Self {
        Self {
            entry_type: TableType::new(),
            id: Uuid::nil(),
        }
    }

    /// Gets the table type.
    #[inline]
    pub const fn table(&self) -> TableType {
        self.entry_type
    }

    /// Creates a new [TableEntry] from the type and UUID.
    #[inline]
    pub const fn create(entry_type: TableType, id: Uuid) -> Self {
        Self { entry_type, id }
    }

    /// Gets whether the [TableEntry] is empty.
    pub const fn is_empty(&self) -> bool {
        self.id.is_nil()
    }

    /// Attempts to get a [TableEntry] from a SQL row.
    pub fn from_sql<'s, R: sqlx::Row>(row: &R, table_key: &'s str, id_key: &'s str) -> Result<Self>
    where
        TableType: for<'d> sqlx::Decode<'d, <R as sqlx::Row>::Database>,
        TableType: sqlx::Type<<R as sqlx::Row>::Database>,
        Uuid: for<'d> sqlx::Decode<'d, <R as sqlx::Row>::Database>,
        Uuid: sqlx::Type<<R as sqlx::Row>::Database>,
        &'s str: sqlx::ColumnIndex<R>,
    {
        Ok(Self {
            entry_type: row.try_get::<TableType, &str>(table_key)?,
            id: row.try_get::<Uuid, &str>(id_key)?,
        })
    }

    /// Attempts to get an optional [TableEntry] from a SQL row.
    pub fn option_from_sql<'s, R: sqlx::Row>(
        row: &R,
        table_key: &'s str,
        id_key: &'s str,
    ) -> Result<Option<Self>>
    where
        Option<TableType>: for<'d> sqlx::Decode<'d, <R as sqlx::Row>::Database>,
        Option<TableType>: sqlx::Type<<R as sqlx::Row>::Database>,
        Option<Uuid>: for<'d> sqlx::Decode<'d, <R as sqlx::Row>::Database>,
        Option<Uuid>: sqlx::Type<<R as sqlx::Row>::Database>,
        &'s str: sqlx::ColumnIndex<R>,
    {
        let entry_type = row.try_get::<Option<TableType>, &str>(table_key)?;
        let id = row.try_get::<Option<Uuid>, &str>(id_key)?;

        match (entry_type, id) {
            (Some(entry_type), Some(id)) => Ok(Some(Self { entry_type, id })),
            (None, None) => Ok(None),
            (None, _) => Err(Error::sql(format!("{table_key}: missing table type entry"))),
            (_, None) => Err(Error::sql(format!("{id_key}: missing table UUID entry"))),
        }
    }

    /// Attempts to get a [TableEntry] from a SQL row.
    pub fn list_from_sql<'s, R: sqlx::Row>(
        row: &R,
        table_key: &'s str,
        id_key: &'s str,
    ) -> Result<Vec<Self>>
    where
        Vec<TableType>: for<'d> sqlx::Decode<'d, <R as sqlx::Row>::Database>,
        Vec<TableType>: sqlx::Type<<R as sqlx::Row>::Database>,
        Vec<Uuid>: for<'d> sqlx::Decode<'d, <R as sqlx::Row>::Database>,
        Vec<Uuid>: sqlx::Type<<R as sqlx::Row>::Database>,
        &'s str: sqlx::ColumnIndex<R>,
    {
        let tables = row.try_get::<Vec<TableType>, &str>(table_key)?;
        let ids = row.try_get::<Vec<Uuid>, &str>(id_key)?;

        let tables_len = tables.len();
        let ids_len = ids.len();

        if tables_len != ids_len {
            Err(Error::sql(format!(
                "table_entry: length mismatch of table type entries: {tables_len}, and table id entries: {ids_len}"
            )))
        } else {
            Ok(tables
                .into_iter()
                .zip(ids)
                .map(|(table, id)| Self::create(table, id))
                .collect())
        }
    }
}

field_access! {
    TableEntry {
        /// Represents the SQL table containing the referenced entry.
        entry_type: TableType,
        /// Represents the UUID of the referenced table entry.
        id: Uuid,
    }
}

impl_default!(TableEntry);
impl_display!(TableEntry, json);
