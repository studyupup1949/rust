//! Statement implementation for ADBC-Taos driver.
//!
//! The `TaosStatement` prepares and executes SQL queries with support for
//! parameterized queries using TDengine's Stmt API.

#![allow(refining_impl_trait)]

use std::sync::Arc;

use adbc_core::{Optionable, Statement, options::{OptionStatement, OptionValue}};
use arrow_array::RecordBatch;
use arrow_array::{Array, BooleanArray, Int8Array, Int16Array, Int32Array, Int64Array,
                  UInt8Array, UInt16Array, UInt32Array, UInt64Array,
                  Float32Array, Float64Array, StringArray, BinaryArray,
                  TimestampMillisecondArray, TimestampMicrosecondArray, TimestampNanosecondArray};
use arrow_schema::Schema;
use taos_client::AsyncFetchable;
use taos_client::AsyncQueryable;
use taos_client::Taos;
use taos_client::sync::Bindable;
use taos_client::Stmt;

use super::reader::TaosRecordBatchReader;
use super::utils::Runtime;

/// SQL statement for TDengine.
///
/// Handles query preparation, parameter binding, and execution.
/// Supports both direct SQL execution and prepared statements with parameter binding.
pub struct TaosStatement {
    rt: Arc<Runtime>,
    conn: Arc<Taos>,
    query: Option<String>,
    prepared_stmt: Option<Stmt>,
    bound_batch: Option<RecordBatch>,
}

impl TaosStatement {
    /// Creates a new statement.
    ///
    /// # Arguments
    /// * `rt` - Async runtime for bridging
    /// * `conn` - TDengine connection
    pub fn new(rt: Arc<Runtime>, conn: Arc<Taos>) -> Self {
        Self {
            rt,
            conn,
            query: None,
            prepared_stmt: None,
            bound_batch: None,
        }
    }

    /// Returns the current query string.
    pub fn query(&self) -> Option<&str> {
        self.query.as_deref()
    }

    /// Converts an Arrow array to TDengine ColumnView for parameter binding.
    ///
    /// # Arguments
    /// * `array` - Arrow array to convert
    ///
    /// # Returns
    /// ColumnView that can be bound to a prepared statement
    fn arrow_array_to_column_view(array: &Arc<dyn Array>) -> Option<taos_client::sync::ColumnView> {
        use taos_client::sync::ColumnView;

        let dtype = array.data_type();

        Some(match dtype {
            arrow_schema::DataType::Boolean => {
                let arr = array.as_any().downcast_ref::<BooleanArray>()?;
                ColumnView::from_bools(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::Int8 => {
                let arr = array.as_any().downcast_ref::<Int8Array>()?;
                ColumnView::from_tiny_ints(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::Int16 => {
                let arr = array.as_any().downcast_ref::<Int16Array>()?;
                ColumnView::from_small_ints(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::Int32 => {
                let arr = array.as_any().downcast_ref::<Int32Array>()?;
                ColumnView::from_ints(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::Int64 => {
                let arr = array.as_any().downcast_ref::<Int64Array>()?;
                ColumnView::from_big_ints(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::UInt8 => {
                let arr = array.as_any().downcast_ref::<UInt8Array>()?;
                ColumnView::from_unsigned_tiny_ints(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::UInt16 => {
                let arr = array.as_any().downcast_ref::<UInt16Array>()?;
                ColumnView::from_unsigned_small_ints(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::UInt32 => {
                let arr = array.as_any().downcast_ref::<UInt32Array>()?;
                ColumnView::from_unsigned_ints(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::UInt64 => {
                let arr = array.as_any().downcast_ref::<UInt64Array>()?;
                ColumnView::from_unsigned_big_ints(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::Float32 => {
                let arr = array.as_any().downcast_ref::<Float32Array>()?;
                ColumnView::from_floats(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::Float64 => {
                let arr = array.as_any().downcast_ref::<Float64Array>()?;
                ColumnView::from_doubles(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::Utf8 | arrow_schema::DataType::LargeUtf8 => {
                let arr = array.as_any().downcast_ref::<StringArray>()?;
                let values: Vec<Option<String>> = arr.iter()
                    .map(|s| s.map(|s| s.to_string()))
                    .collect();
                // Specify the turbofish syntax to help type inference
                taos_client::sync::ColumnView::from_nchar::<String, Option<String>, _, _>(values)
            }
            arrow_schema::DataType::Binary | arrow_schema::DataType::LargeBinary => {
                let arr = array.as_any().downcast_ref::<BinaryArray>()?;
                let values: Vec<Option<&[u8]>> = arr.iter().collect();
                taos_client::sync::ColumnView::from_bytes::<&[u8], Option<&[u8]>, _, _>(values)
            }
            arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, _) => {
                let arr = array.as_any().downcast_ref::<TimestampMillisecondArray>()?;
                ColumnView::from_millis_timestamp(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::Microsecond, _) => {
                let arr = array.as_any().downcast_ref::<TimestampMicrosecondArray>()?;
                ColumnView::from_micros_timestamp(arr.iter().collect::<Vec<_>>())
            }
            arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::Nanosecond, _) => {
                let arr = array.as_any().downcast_ref::<TimestampNanosecondArray>()?;
                ColumnView::from_nanos_timestamp(arr.iter().collect::<Vec<_>>())
            }
            _ => {
                // Unsupported type - return None
                return None;
            }
        })
    }
}

impl Optionable for TaosStatement {
    type Option = OptionStatement;

    fn set_option(
        &mut self,
        _key: Self::Option,
        _value: OptionValue,
    ) -> adbc_core::error::Result<()> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Statement options not implemented",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_option_string(&self, _key: Self::Option) -> adbc_core::error::Result<String> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Statement options not implemented",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_option_bytes(&self, _key: Self::Option) -> adbc_core::error::Result<Vec<u8>> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Statement options not implemented",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_option_double(&self, _key: Self::Option) -> adbc_core::error::Result<f64> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Statement options not implemented",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_option_int(&self, _key: Self::Option) -> adbc_core::error::Result<i64> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Statement options not implemented",
            adbc_core::error::Status::NotImplemented,
        ))
    }
}

impl Statement for TaosStatement {
    fn bind(&mut self, batch: RecordBatch) -> adbc_core::error::Result<()> {
        // Check if we have a prepared statement
        let stmt = self.prepared_stmt.as_mut().ok_or_else(|| {
            adbc_core::error::Error::with_message_and_status(
                "Cannot bind: statement not prepared. Call prepare() first.",
                adbc_core::error::Status::InvalidState,
            )
        })?;

        // Re-prepare the statement with the query (TDengine's API chaining)
        let query = self.query.as_ref().ok_or_else(|| {
            adbc_core::error::Error::with_message_and_status(
                "No query set",
                adbc_core::error::Status::InvalidState,
            )
        })?;
        let stmt = stmt.prepare(query.as_str()).map_err(|e| {
            adbc_core::error::Error::with_message_and_status(
                format!("Failed to prepare statement: {}", e),
                adbc_core::error::Status::Internal,
            )
        })?;

        // Convert Arrow columns to ColumnView array
        let mut column_views: Vec<taos_client::sync::ColumnView> = Vec::new();
        for col_idx in 0..batch.num_columns() {
            let array = batch.column(col_idx);
            match Self::arrow_array_to_column_view(array) {
                Some(cv) => column_views.push(cv),
                None => {
                    return Err(adbc_core::error::Error::with_message_and_status(
                        format!("Cannot bind column {}: unsupported Arrow type {:?}",
                               col_idx, array.data_type()),
                        adbc_core::error::Status::NotImplemented,
                    ));
                }
            }
        }

        // Bind parameters to the statement
        stmt.bind(&column_views).map_err(|e| {
            adbc_core::error::Error::with_message_and_status(
                format!("Failed to bind parameters: {}", e),
                adbc_core::error::Status::Internal,
            )
        })?;

        // Store bound batch for later execution
        self.bound_batch = Some(batch);
        Ok(())
    }

    fn bind_stream(
        &mut self,
        _reader: Box<dyn arrow_array::RecordBatchReader + Send>,
    ) -> adbc_core::error::Result<()> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Stream bind not implemented",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn execute(&mut self) -> adbc_core::error::Result<Box<dyn arrow_array::RecordBatchReader + Send>> {
        // Only use prepared statement if we have bound data (parameterized query)
        // TDengine Stmt API is only for parameterized INSERT, not for DDL or direct queries
        if self.prepared_stmt.is_some() && self.bound_batch.is_some() {
            if let Some(stmt) = &mut self.prepared_stmt {
                if let Some(query) = &self.query {
                    let stmt = stmt.prepare(query).map_err(|e| {
                        adbc_core::error::Error::with_message_and_status(
                            format!("Failed to prepare statement: {}", e),
                            adbc_core::error::Status::Internal,
                        )
                    })?;

                    let affected_rows = stmt.execute().map_err(|e| {
                        adbc_core::error::Error::with_message_and_status(
                            format!("Execute failed: {}", e),
                            adbc_core::error::Status::Internal,
                        )
                    })?;

                    let schema = arrow_schema::Schema::new(vec![
                        arrow_schema::Field::new("affected_rows", arrow_schema::DataType::Int64, false),
                    ]);
                    let batch = RecordBatch::try_new(
                        Arc::new(schema.clone()),
                        vec![Arc::new(arrow_array::Int64Array::from(vec![affected_rows as i64]))],
                    ).map_err(|e| adbc_core::error::Error::with_message_and_status(
                        format!("Failed to create result batch: {}", e),
                        adbc_core::error::Status::Internal,
                    ))?;
                    return Ok(Box::new(super::reader::VecRecordBatchReader::new(vec![batch], schema)));
                }
            }
        }

        // Direct query execution
        let query = self
            .query
            .as_ref()
            .ok_or_else(|| {
                adbc_core::error::Error::with_message_and_status(
                    "No query set",
                    adbc_core::error::Status::InvalidArguments,
                )
            })?
            .clone();

        // Execute query
        let result_set: taos_client::ResultSet = self
            .rt
            .block_on(async { self.conn.query(&query).await })
            .map_err(|e| {
                adbc_core::error::Error::with_message_and_status(
                    format!("Query failed: {}", e),
                    adbc_core::error::Status::Internal,
                )
            })?;

        // Build schema from result set with precision detection
        let fields = result_set.fields();
        let precision = result_set.precision();
        let schema = Schema::new(
            fields
                .iter()
                .map(|f: &taos_client::Field| {
                    arrow_schema::Field::new(
                        f.name(),
                        map_taos_ty_to_arrow_with_precision(f.ty(), Some(precision)),
                        true,
                    )
                })
                .collect::<Vec<_>>(),
        );

        // Create reader
        Ok(Box::new(TaosRecordBatchReader::new(result_set, schema)))
    }

    fn execute_update(&mut self) -> adbc_core::error::Result<Option<i64>> {
        // Only use prepared statement if we have bound data (parameterized query)
        // TDengine Stmt API is only for parameterized INSERT, not for DDL or direct queries
        if self.prepared_stmt.is_some() && self.bound_batch.is_some() {
            if let Some(stmt) = &mut self.prepared_stmt {
                if let Some(query) = &self.query {
                    let stmt = stmt.prepare(query).map_err(|e| {
                        adbc_core::error::Error::with_message_and_status(
                            format!("Failed to prepare statement: {}", e),
                            adbc_core::error::Status::Internal,
                        )
                    })?;

                    let affected_rows = stmt.execute().map_err(|e| {
                        adbc_core::error::Error::with_message_and_status(
                            format!("Execute failed: {}", e),
                            adbc_core::error::Status::Internal,
                        )
                    })?;
                    return Ok(Some(affected_rows as i64));
                }
            }
        }

        // Direct execution
        let query = self
            .query
            .as_ref()
            .ok_or_else(|| {
                adbc_core::error::Error::with_message_and_status(
                    "No query set",
                    adbc_core::error::Status::InvalidArguments,
                )
            })?
            .clone();

        // Execute statement
        let affected_rows = self
            .rt
            .block_on(async { self.conn.exec(&query).await })
            .map_err(|e| {
                adbc_core::error::Error::with_message_and_status(
                    format!("Execute failed: {}", e),
                    adbc_core::error::Status::Internal,
                )
            })?;

        Ok(Some(affected_rows as i64))
    }

    fn execute_schema(&mut self) -> adbc_core::error::Result<Schema> {
        let query = self
            .query
            .as_ref()
            .ok_or_else(|| {
                adbc_core::error::Error::with_message_and_status(
                    "No query set",
                    adbc_core::error::Status::InvalidArguments,
                )
            })?
            .clone();

        let result_set: taos_client::ResultSet = self
            .rt
            .block_on(async { self.conn.query(&query).await })
            .map_err(|e| {
                adbc_core::error::Error::with_message_and_status(
                    format!("Query failed: {}", e),
                    adbc_core::error::Status::Internal,
                )
            })?;

        let fields = result_set.fields();
        Ok(Schema::new(
            fields
                .iter()
                .map(|f: &taos_client::Field| {
                    arrow_schema::Field::new(
                        f.name(),
                        map_taos_ty_to_arrow(f.ty()),
                        true,
                    )
                })
                .collect::<Vec<_>>(),
        ))
    }

    fn execute_partitions(&mut self) -> adbc_core::error::Result<adbc_core::PartitionedResult> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Partitioned execution not supported",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn get_parameter_schema(&self) -> adbc_core::error::Result<Schema> {
        // For prepared statements, we could return the parameter schema
        // Currently returning empty schema for non-parameterized queries
        Ok(Schema::new(Vec::<arrow_schema::Field>::new()))
    }

    fn prepare(&mut self) -> adbc_core::error::Result<()> {
        let query = self
            .query
            .as_ref()
            .ok_or_else(|| {
                adbc_core::error::Error::with_message_and_status(
                    "No query set",
                    adbc_core::error::Status::InvalidArguments,
                )
            })?
            .clone();

        // Initialize prepared statement using TDengine's sync Stmt API
        let mut stmt = Stmt::init(&self.conn).map_err(|e| {
            adbc_core::error::Error::with_message_and_status(
                format!("Failed to init statement: {}", e),
                adbc_core::error::Status::Internal,
            )
        })?;

        // Prepare the SQL - this validates the SQL syntax and parameter placeholders
        stmt.prepare(&query).map_err(|e| {
            adbc_core::error::Error::with_message_and_status(
                format!("Failed to prepare statement: {}", e),
                adbc_core::error::Status::Internal,
            )
        })?;

        // Note: TDengine's Stmt API uses chaining, so we store the initialized statement
        // and re-prepare on bind/execute. The prepare() call above validates the SQL.
        self.prepared_stmt = Some(stmt);
        self.bound_batch = None;
        Ok(())
    }

    fn set_sql_query(&mut self, query: impl AsRef<str>) -> adbc_core::error::Result<()> {
        self.query = Some(query.as_ref().to_string());
        // Reset prepared state when new query is set
        self.prepared_stmt = None;
        self.bound_batch = None;
        Ok(())
    }

    fn set_substrait_plan(
        &mut self,
        _plan: impl AsRef<[u8]>,
    ) -> adbc_core::error::Result<()> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Substrait not supported",
            adbc_core::error::Status::NotImplemented,
        ))
    }

    fn cancel(&mut self) -> adbc_core::error::Result<()> {
        Err(adbc_core::error::Error::with_message_and_status(
            "Query cancellation not supported",
            adbc_core::error::Status::NotImplemented,
        ))
    }
}

/// Maps TDengine Ty to Arrow data type.
fn map_taos_ty_to_arrow(ty: taos_client::Ty) -> arrow_schema::DataType {
    map_taos_ty_to_arrow_with_precision(ty, None)
}

/// Maps TDengine Ty to Arrow data type with timestamp precision.
fn map_taos_ty_to_arrow_with_precision(
    ty: taos_client::Ty,
    precision: Option<taos_client::Precision>,
) -> arrow_schema::DataType {
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
        Ty::Timestamp => {
            let time_unit = match precision {
                Some(taos_client::Precision::Millisecond) => arrow_schema::TimeUnit::Millisecond,
                Some(taos_client::Precision::Microsecond) => arrow_schema::TimeUnit::Microsecond,
                Some(taos_client::Precision::Nanosecond) => arrow_schema::TimeUnit::Nanosecond,
                None => arrow_schema::TimeUnit::Millisecond, // Default to millisecond
            };
            arrow_schema::DataType::Timestamp(time_unit, None)
        }
        Ty::UTinyInt => arrow_schema::DataType::UInt8,
        Ty::USmallInt => arrow_schema::DataType::UInt16,
        Ty::UInt => arrow_schema::DataType::UInt32,
        Ty::UBigInt => arrow_schema::DataType::UInt64,
        Ty::Geometry => arrow_schema::DataType::Binary,
        Ty::Decimal => {
            // DECIMAL type with default precision/scale
            // TDengine DECIMAL(38, 0) is the maximum precision
            arrow_schema::DataType::Decimal128(38, 0)
        }
        _ => arrow_schema::DataType::Utf8,
    }
}
