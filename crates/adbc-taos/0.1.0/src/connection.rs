//! Connection implementation for ADBC-Taos driver.
//!
//! The `TaosConnection` wraps a TDengine connection and provides methods
//! for creating statements and executing metadata queries.

#![allow(refining_impl_trait)]

use std::sync::Arc;

use adbc_core::{Connection, Optionable, options::{OptionConnection, OptionValue}};
use arrow_array::RecordBatch;
use arrow_schema::Schema;

use super::reader::VecRecordBatchReader;
use super::statement::TaosStatement;
use super::utils::Runtime;
use taos_client::AsyncQueryable;
use taos_client::Taos;

/// Connection wrapper for TDengine.
///
/// Wraps a TDengine `Taos` connection and manages async runtime bridging.
pub struct TaosConnection {
    rt: Arc<Runtime>,
    conn: Arc<Taos>,
    version: String,
}

impl TaosConnection {
    /// Creates a new connection wrapper.
    ///
    /// # Arguments
    /// * `rt` - Async runtime for bridging async TDengine client
    /// * `conn` - TDengine connection to wrap
    pub fn new(rt: Arc<Runtime>, conn: Arc<Taos>) -> Self {
        let conn_ref = Arc::clone(&conn);
        let version = rt
            .block_on(async { conn_ref.server_version().await })
            .unwrap_or(std::borrow::Cow::Borrowed("unknown"));

        Self { rt, conn, version: version.into_owned() }
    }

    /// Returns the TDengine server version.
    pub fn server_version(&self) -> &str {
        &self.version
    }

    /// Returns a reference to the underlying TDengine connection.
    pub fn inner(&self) -> &Taos {
        &self.conn
    }

    /// Gets TAGS metadata for a TDengine supertable.
    ///
    /// # Arguments
    /// * `catalog` - Database name (catalog in ADBC terms)
    /// * `table_name` - Supertable name
    ///
    /// # Returns
    /// RecordBatchReader containing tag names and types
    ///
    /// # Example
    /// ```ignore
    /// let tags_reader = conn.get_table_tags("my_db", "my_supertable")?;
    /// ```
    pub fn get_table_tags(
        &self,
        catalog: Option<&str>,
        table_name: &str,
    ) -> adbc_core::error::Result<Box<dyn arrow_array::RecordBatchReader + Send>> {
        let db = catalog.unwrap_or("log");

        // Query tags from TDengine using DESCRIBE with table name that includes tags
        // For supertables, we can use: DESCRIBE <db>.<table> which returns both columns and tags
        // The result has a special column 'field' that indicates if it's a tag
        let tags_query = format!(
            "SELECT field_name, field_type, note FROM information_schema.ins_tags \
             WHERE db_name = '{}' AND table_name = '{}'",
            db, table_name
        );

        let result_set = self.rt.block_on(async {
            self.conn.use_database(db).await?;
            self.conn.query(&tags_query).await
        }).map_err(|e| adbc_core::error::Error::with_message_and_status(
            format!("Failed to query tags for {}.{}: {}", db, table_name, e),
            adbc_core::error::Status::Internal,
        ))?;

        // Build Arrow schema for tags result
        let schema = arrow_schema::Schema::new(vec![
            arrow_schema::Field::new("tag_name", arrow_schema::DataType::Utf8, false),
            arrow_schema::Field::new("tag_type", arrow_schema::DataType::Utf8, false),
            arrow_schema::Field::new("tag_comment", arrow_schema::DataType::Utf8, true),
        ]);

        let reader = super::reader::TaosRecordBatchReader::new(result_set, schema);
        Ok(Box::new(reader))
    }

    /// Checks if a table is a supertable.
    ///
    /// # Arguments
    /// * `catalog` - Database name
    /// * `table_name` - Table name
    ///
    /// # Returns
    /// true if the table is a supertable, false otherwise
    pub fn is_supertable(
        &self,
        catalog: Option<&str>,
        table_name: &str,
    ) -> adbc_core::error::Result<bool> {
        let db = catalog.unwrap_or("log");

        let is_super = self.rt.block_on(async {
            self.conn.use_database(db).await?;
            let mut result = self.conn.query("SHOW TABLES").await?;
            use taos_client::sync::Fetchable;

            for row in result.rows().flatten() {
                let values = row.into_values();
                if values.len() < 2 {
                    continue;
                }
                // SHOW TABLES returns table_name and table_type
                if let (Some(taos_client::Value::VarChar(name)), Some(taos_client::Value::VarChar(ttype))) =
                    (values.first(), values.get(1))
                    && name.as_str() == table_name && ttype.as_str() == "SUPER TABLE" {
                        return Ok(true);
                    }
            }
            Ok::<_, taos_client::Error>(false)
        }).map_err(|e| adbc_core::error::Error::with_message_and_status(
            format!("Failed to check if table is supertable: {}", e),
            adbc_core::error::Status::Internal,
        ))?;

        Ok(is_super)
    }
}

impl Optionable for TaosConnection {
    type Option = OptionConnection;

    fn set_option(&mut self, _key: Self::Option, _value: OptionValue) -> adbc_core::error::Result<()> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Connection options not implemented",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_option_string(&self, _key: Self::Option) -> adbc_core::error::Result<String> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Connection options not implemented",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_option_bytes(&self, _key: Self::Option) -> adbc_core::error::Result<Vec<u8>> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Connection options not implemented",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_option_double(&self, _key: Self::Option) -> adbc_core::error::Result<f64> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Connection options not implemented",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_option_int(&self, _key: Self::Option) -> adbc_core::error::Result<i64> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Connection options not implemented",
            adbc_core::error::Status::NotImplemented,
        ))
    }
}

impl Connection for TaosConnection {
    type StatementType = TaosStatement;

    fn new_statement(&mut self) -> adbc_core::error::Result<Self::StatementType> {
        Ok(TaosStatement::new(self.rt.clone(), self.conn.clone()))
    }

    fn cancel(&mut self) -> adbc_core::error::Result<()> {
        // TDengine doesn't support query cancellation
        Err(adbc_core::error::Error::with_message_and_status(
            "Query cancellation not supported",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_info(
        &self,
        _codes: Option<std::collections::HashSet<adbc_core::options::InfoCode>>,
    ) -> adbc_core::error::Result<impl arrow_array::RecordBatchReader + Send> {
        use adbc_core::options::InfoCode;

        // Build info data as RecordBatches
        let info_code_array = arrow_array::UInt32Array::from(vec![
            InfoCode::VendorName as u32,
            InfoCode::VendorVersion as u32,
            InfoCode::DriverName as u32,
            InfoCode::DriverVersion as u32,
            InfoCode::DriverAdbcVersion as u32,
        ]);

        let info_value_array = arrow_array::StringArray::from(vec![
            "TDengine",
            &self.version,
            "ADBC-Taos",
            env!("CARGO_PKG_VERSION"),
            "0.1.0",
        ]);

        let schema = arrow_schema::Schema::new(vec![
            arrow_schema::Field::new("info_code", arrow_schema::DataType::UInt32, false),
            arrow_schema::Field::new("info_value", arrow_schema::DataType::Utf8, false),
        ]);

        let batch = RecordBatch::try_new(Arc::new(schema), vec![
            Arc::new(info_code_array),
            Arc::new(info_value_array),
        ])
            .map_err(|e| adbc_core::error::Error::with_message_and_status(
                format!("Failed to create info batch: {}", e),
                adbc_core::error::Status::Internal,
            ))?;

        let schema = arrow_schema::Schema::new(vec![
            arrow_schema::Field::new("info_code", arrow_schema::DataType::UInt32, false),
            arrow_schema::Field::new("info_value", arrow_schema::DataType::Utf8, false),
        ]);

        Ok(VecRecordBatchReader::new(vec![batch], schema))
    }

    fn get_objects(
        &self,
        depth: adbc_core::options::ObjectDepth,
        catalog: Option<&str>,
        _db_schema: Option<&str>,
        table_name: Option<&str>,
        table_type: Option<Vec<&str>>,
        column_name: Option<&str>,
    ) -> adbc_core::error::Result<Box<dyn arrow_array::RecordBatchReader + Send>> {
        use adbc_core::options::ObjectDepth;
        use arrow_array::{ArrayRef, StringArray};
        use arrow_schema::{DataType, Field};
        use std::ffi::c_int;

        // Convert depth to c_int for comparison
        let depth_val: c_int = depth.into();
        let catalogs_depth: c_int = ObjectDepth::Catalogs.into();
        let schemas_depth: c_int = ObjectDepth::Schemas.into();
        let tables_depth: c_int = ObjectDepth::Tables.into();

        // Get databases
        let databases: Vec<String> = self.rt.block_on(async {
            let mut result = self.conn.query("SHOW DATABASES").await?;
            use taos_client::sync::Fetchable;
            let mut dbs = Vec::new();
            for row in result.rows().flatten() {
                let values = row.into_values();
                if let Some(taos_client::Value::VarChar(name)) = values.first() {
                    dbs.push(name.clone());
                }
            }
            Ok::<_, taos_client::Error>(dbs)
        }).map_err(|e| adbc_core::error::Error::with_message_and_status(
            format!("Failed to list databases: {}", e),
            adbc_core::error::Status::Internal,
        ))?;

        // Filter by catalog if specified
        let databases: Vec<String> = databases.into_iter()
            .filter(|db| catalog.is_none() || catalog == Some(db.as_str()))
            .collect();

        if depth_val == catalogs_depth {
            // Return only catalog names
            let catalog_array: ArrayRef = Arc::new(StringArray::from(databases.clone()));
            
            let schema = Schema::new(vec![
                Field::new("catalog_name", DataType::Utf8, true),
            ]);
            
            let batch = RecordBatch::try_new(Arc::new(schema.clone()), vec![catalog_array])
                .map_err(|e| adbc_core::error::Error::with_message_and_status(
                    format!("Failed to create batch: {}", e),
                    adbc_core::error::Status::Internal,
                ))?;
            
            return Ok(Box::new(VecRecordBatchReader::new(vec![batch], schema)));
        }

        // For deeper levels, build full object tree
        let mut catalog_names: Vec<String> = Vec::new();
        let mut db_schema_names: Vec<Option<String>> = Vec::new();
        let mut table_names: Vec<Option<String>> = Vec::new();
        let mut table_types: Vec<Option<String>> = Vec::new();
        let mut column_names_list: Vec<Option<String>> = Vec::new();
        let mut column_types: Vec<Option<String>> = Vec::new();

        for db in &databases {
            if depth_val == schemas_depth {
                catalog_names.push(db.clone());
                db_schema_names.push(Some(db.clone()));
                table_names.push(None);
                table_types.push(None);
                column_names_list.push(None);
                column_types.push(None);
                continue;
            }

            // Get tables for this database
            let tables: Vec<(String, String)> = self.rt.block_on(async {
                self.conn.use_database(db).await?;
                let mut result = self.conn.query("SHOW TABLES").await?;
                use taos_client::sync::Fetchable;
                let mut tbls = Vec::new();
                for row in result.rows().flatten() {
                    let values = row.into_values();
                    // SHOW TABLES returns table_name and table_type (with table_type indicating "SUPER TABLE" for supertables)
                    if values.len() >= 2 {
                        if let (Some(taos_client::Value::VarChar(name)), Some(taos_client::Value::VarChar(ttype))) =
                            (values.first(), values.get(1))
                        {
                            let table_type = if ttype.as_str() == "SUPER TABLE" {
                                "SUPER TABLE"
                            } else {
                                "TABLE"
                            };
                            tbls.push((name.clone(), table_type.to_string()));
                        }
                    } else if let Some(taos_client::Value::VarChar(name)) = values.first() {
                        tbls.push((name.clone(), "TABLE".to_string()));
                    }
                }
                Ok::<_, taos_client::Error>(tbls)
            }).unwrap_or_default();

            // Filter tables
            let tables: Vec<(String, String)> = tables.into_iter()
                .filter(|(tbl, ttype)| {
                    let name_match = table_name.is_none() || table_name == Some(tbl.as_str());
                    let type_match = table_type.is_none() ||
                        table_type.as_ref().map(|t| t.contains(&ttype.as_str())).unwrap_or(true);
                    name_match && type_match
                })
                .collect();

            if depth_val == tables_depth {
                for (tbl, ttype) in tables {
                    catalog_names.push(db.clone());
                    db_schema_names.push(Some(db.clone()));
                    table_names.push(Some(tbl));
                    table_types.push(Some(ttype));
                    column_names_list.push(None);
                    column_types.push(None);
                }
                continue;
            }

            // Get columns for each table
            for (tbl, ttype) in tables {
                let columns: Vec<(String, String)> = self.rt.block_on(async {
                    self.conn.use_database(db).await?;
                    let desc = self.conn.describe(&tbl).await?;
                    use std::ops::Deref;
                    Ok::<_, taos_client::Error>(
                        desc.deref().iter()
                            .map(|f| (f.field().to_string(), format!("{:?}", f.ty())))
                            .collect()
                    )
                }).unwrap_or_default();

                // Filter columns
                let columns: Vec<(String, String)> = columns.into_iter()
                    .filter(|(col, _)| column_name.is_none() || column_name == Some(col.as_str()))
                    .collect();

                if columns.is_empty() {
                    catalog_names.push(db.clone());
                    db_schema_names.push(Some(db.clone()));
                    table_names.push(Some(tbl.clone()));
                    table_types.push(Some(ttype.clone()));
                    column_names_list.push(None);
                    column_types.push(None);
                } else {
                    for (col, ctype) in columns {
                        catalog_names.push(db.clone());
                        db_schema_names.push(Some(db.clone()));
                        table_names.push(Some(tbl.clone()));
                        table_types.push(Some(ttype.clone()));
                        column_names_list.push(Some(col));
                        column_types.push(Some(ctype));
                    }
                }
            }
        }

        // Build result schema and batch
        let schema = Schema::new(vec![
            Field::new("catalog_name", DataType::Utf8, true),
            Field::new("db_schema_name", DataType::Utf8, true),
            Field::new("table_name", DataType::Utf8, true),
            Field::new("table_type", DataType::Utf8, true),
            Field::new("column_name", DataType::Utf8, true),
            Field::new("column_type", DataType::Utf8, true),
        ]);

        let batch = RecordBatch::try_new(
            Arc::new(schema.clone()),
            vec![
                Arc::new(StringArray::from(catalog_names)) as ArrayRef,
                Arc::new(StringArray::from(db_schema_names)) as ArrayRef,
                Arc::new(StringArray::from(table_names)) as ArrayRef,
                Arc::new(StringArray::from(table_types)) as ArrayRef,
                Arc::new(StringArray::from(column_names_list)) as ArrayRef,
                Arc::new(StringArray::from(column_types)) as ArrayRef,
            ],
        ).map_err(|e| adbc_core::error::Error::with_message_and_status(
            format!("Failed to create objects batch: {}", e),
            adbc_core::error::Status::Internal,
        ))?;

        Ok(Box::new(VecRecordBatchReader::new(vec![batch], schema)))
    }

    fn get_table_schema(
        &self,
        _catalog: Option<&str>,
        db_schema: Option<&str>,
        table_name: &str,
    ) -> adbc_core::error::Result<Schema> {
        let db = db_schema.unwrap_or("log");

        // TDengine's describe doesn't support db.table format, need to USE database first
        let describe_result = self
            .rt
            .block_on(async {
                // Use database first
                self.conn.use_database(db).await?;
                // Then describe table (without database prefix)
                self.conn.describe(table_name).await
            })
            .map_err(|e| {
                adbc_core::error::Error::with_message_and_status(
                    format!("Failed to describe table '{}': {}", table_name, e),
                    adbc_core::error::Status::NotFound,
                )
            });

        let describe_result = describe_result?;

        // Convert TDengine ColumnMeta to Arrow fields
        use std::ops::Deref;
        let fields: Vec<arrow_schema::Field> = describe_result
            .deref()
            .iter()
            .map(|f| {
                arrow_schema::Field::new(
                    f.field(),
                    map_taos_ty_to_arrow(f.ty()),
                    true, // nullable
                )
            })
            .collect();

        Ok(Schema::new(fields))
    }

    fn get_table_types(&self) -> adbc_core::error::Result<impl arrow_array::RecordBatchReader + Send> {
        // TDengine supports "TABLE" and "BASE TABLE" (regular tables)
        let table_type_array = arrow_array::StringArray::from(vec![
            "TABLE",
            "BASE TABLE",
        ]);

        let schema = arrow_schema::Schema::new(vec![arrow_schema::Field::new(
            "table_type",
            arrow_schema::DataType::Utf8,
            false,
        )]);

        let batch =
            RecordBatch::try_new(Arc::new(schema), vec![Arc::new(table_type_array)]).map_err(
                |e| {
                    adbc_core::error::Error::with_message_and_status(
                        format!("Failed to create table types batch: {}", e),
                        adbc_core::error::Status::Internal,
                    )
                },
            )?;

        Ok(VecRecordBatchReader::new(vec![batch], arrow_schema::Schema::new(vec![
            arrow_schema::Field::new("table_type", arrow_schema::DataType::Utf8, false),
        ])))
    }

    fn get_statistic_names(&self) -> adbc_core::error::Result<impl arrow_array::RecordBatchReader + Send> {
        // Return empty statistics - TDengine doesn't have named statistics
        let name_array = arrow_array::StringArray::from(vec![] as Vec<String>);
        let description_array = arrow_array::StringArray::from(vec![] as Vec<String>);

        let schema = arrow_schema::Schema::new(vec![
            arrow_schema::Field::new("statistic_name", arrow_schema::DataType::Utf8, false),
            arrow_schema::Field::new("statistic_description", arrow_schema::DataType::Utf8, false),
        ]);

        let batch = RecordBatch::try_new(
            Arc::new(schema.clone()),
            vec![Arc::new(name_array), Arc::new(description_array)],
        )
        .map_err(|e| {
            adbc_core::error::Error::with_message_and_status(
                format!("Failed to create statistics batch: {}", e),
                adbc_core::error::Status::Internal,
            )
        })?;

        Ok(VecRecordBatchReader::new(vec![batch], schema))
    }

    fn get_statistics(
        &self,
        _catalog: Option<&str>,
        _db_schema: Option<&str>,
        _table_name: Option<&str>,
        _approximate: bool,
    ) -> adbc_core::error::Result<impl arrow_array::RecordBatchReader + Send> {
        // Return empty statistics
        let schema = arrow_schema::Schema::new(vec![] as Vec<arrow_schema::Field>);
        Ok(VecRecordBatchReader::new(vec![], schema))
    }

    fn commit(&mut self) -> adbc_core::error::Result<()> {
        // TDengine has limited transaction support
        Err(adbc_core::error::Error::with_message_and_status(
            "Transaction commit not supported",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn rollback(&mut self) -> adbc_core::error::Result<()> {
        // TDengine has limited transaction support
        Err(adbc_core::error::Error::with_message_and_status(
            "Transaction rollback not supported",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn read_partition(&self, _partition: impl AsRef<[u8]>) -> adbc_core::error::Result<Box<dyn arrow_array::RecordBatchReader + Send>> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Partitioned reads not supported",
            adbc_core::error::Status::NotImplemented,
        ))
    }
}

/// Maps TDengine Ty to Arrow data type.
fn map_taos_ty_to_arrow(ty: taos_client::Ty) -> arrow_schema::DataType {
    use taos_client::Ty;

    match ty {
        Ty::Bool => arrow_schema::DataType::Boolean,
        Ty::TinyInt => arrow_schema::DataType::Int8,
        Ty::SmallInt => arrow_schema::DataType::Int16,
        Ty::Int => arrow_schema::DataType::Int32,
        Ty::BigInt => arrow_schema::DataType::Int64,
        Ty::Float => arrow_schema::DataType::Float32,
        Ty::Double => arrow_schema::DataType::Float64,
        Ty::VarChar | Ty::VarBinary => arrow_schema::DataType::Binary,
        Ty::NChar | Ty::Json => arrow_schema::DataType::Utf8,
        Ty::Timestamp => arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, None),
        Ty::UTinyInt => arrow_schema::DataType::UInt8,
        Ty::USmallInt => arrow_schema::DataType::UInt16,
        Ty::UInt => arrow_schema::DataType::UInt32,
        Ty::UBigInt => arrow_schema::DataType::UInt64,
        Ty::Geometry => arrow_schema::DataType::Binary,
        Ty::Decimal => {
            // DECIMAL type with default precision/scale
            // TDengine DECIMAL(38, 0) is the maximum precision
            // Using 38 as precision and 0 as scale as default
            arrow_schema::DataType::Decimal128(38, 0)
        }
        _ => arrow_schema::DataType::Utf8,
    }
}

// Rust guideline compliant 2024-12-31
