//! RecordBatchReader implementation for streaming query results.
//!
//! `TaosRecordBatchReader` streams TDengine query results as Arrow RecordBatches.
//! Optimized to use block-based columnar access instead of row-by-row iteration.

use std::sync::Arc;

use arrow_array::RecordBatch;
use arrow_array::builder::{
    BooleanBuilder, StringBuilder, GenericBinaryBuilder,
    Int8Builder, Int16Builder, Int32Builder, Int64Builder,
    UInt8Builder, UInt16Builder, UInt32Builder, UInt64Builder,
    Float32Builder, Float64Builder,
    TimestampMillisecondBuilder, TimestampMicrosecondBuilder, TimestampNanosecondBuilder,
    NullBuilder, Decimal128Builder,
};
use arrow_schema::{Schema, SchemaRef};

use taos_client::sync::Fetchable;
use taos_client::{ColumnView, RawBlock, BorrowedValue};

/// Precomputed scale factors for decimal conversion (10^0 to 10^38).
/// Avoids repeated pow() calls in hot loop.
const SCALE_FACTORS: [i128; 39] = {
    let mut factors = [1i128; 39];
    let mut i = 1;
    while i < 39 {
        factors[i] = factors[i - 1] * 10;
        i += 1;
    }
    factors
};

/// Iterator-based RecordBatchReader for pre-loaded record batches.
pub struct VecRecordBatchReader {
    batches: std::vec::IntoIter<RecordBatch>,
    schema: SchemaRef,
}

impl VecRecordBatchReader {
    pub fn new(batches: Vec<RecordBatch>, schema: Schema) -> Self {
        Self {
            batches: batches.into_iter(),
            schema: Arc::new(schema),
        }
    }

    pub fn empty(schema: Schema) -> Self {
        Self {
            batches: vec![].into_iter(),
            schema: Arc::new(schema),
        }
    }
}

impl arrow_array::RecordBatchReader for VecRecordBatchReader {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
}

impl Iterator for VecRecordBatchReader {
    type Item = std::result::Result<arrow_array::RecordBatch, arrow_schema::ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        self.batches.next().map(Ok)
    }
}

/// TDengine result set reader that streams data in Arrow format.
///
/// Optimized implementation that uses `blocks()` API for direct columnar access,
/// eliminating the row-to-column conversion overhead of the original `rows()` approach.
pub struct TaosRecordBatchReader {
    result_set: Option<taos_client::ResultSet>,
    schema: SchemaRef,
    #[allow(dead_code)]
    batch_size: usize,
}

impl TaosRecordBatchReader {
    const DEFAULT_BATCH_SIZE: usize = 8192;

    pub fn new(result_set: taos_client::ResultSet, schema: Schema) -> Self {
        Self {
            result_set: Some(result_set),
            schema: Arc::new(schema),
            batch_size: Self::DEFAULT_BATCH_SIZE,
        }
    }

    pub fn with_batch_size(mut self, batch_size: usize) -> Self {
        self.batch_size = batch_size.max(1);
        self
    }

    /// Converts a RawBlock directly to Arrow arrays using columnar access.
    fn convert_block_to_arrays(
        &self,
        block: &RawBlock,
    ) -> Result<Vec<Arc<dyn arrow_array::Array>>, arrow_schema::ArrowError> {
        let num_rows = block.nrows();
        let column_views = block.column_views();
        let num_columns = self.schema.fields().len();

        let mut arrays: Vec<Arc<dyn arrow_array::Array>> = Vec::with_capacity(num_columns);

        for col_idx in 0..num_columns {
            let data_type = self.schema.field(col_idx).data_type();
            let column_view = column_views.get(col_idx);

            let array: Arc<dyn arrow_array::Array> = match (data_type, column_view) {
                (arrow_schema::DataType::Boolean, Some(cv)) => {
                    Self::convert_bool_column(cv, num_rows)
                }
                (arrow_schema::DataType::Int8, Some(cv)) => {
                    Self::convert_tinyint_column(cv, num_rows)
                }
                (arrow_schema::DataType::Int16, Some(cv)) => {
                    Self::convert_smallint_column(cv, num_rows)
                }
                (arrow_schema::DataType::Int32, Some(cv)) => {
                    Self::convert_int_column(cv, num_rows)
                }
                (arrow_schema::DataType::Int64, Some(cv)) => {
                    Self::convert_bigint_column(cv, num_rows)
                }
                (arrow_schema::DataType::UInt8, Some(cv)) => {
                    Self::convert_utinyint_column(cv, num_rows)
                }
                (arrow_schema::DataType::UInt16, Some(cv)) => {
                    Self::convert_usmallint_column(cv, num_rows)
                }
                (arrow_schema::DataType::UInt32, Some(cv)) => {
                    Self::convert_uint_column(cv, num_rows)
                }
                (arrow_schema::DataType::UInt64, Some(cv)) => {
                    Self::convert_ubigint_column(cv, num_rows)
                }
                (arrow_schema::DataType::Float32, Some(cv)) => {
                    Self::convert_float_column(cv, num_rows)
                }
                (arrow_schema::DataType::Float64, Some(cv)) => {
                    Self::convert_double_column(cv, num_rows)
                }
                (arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::Millisecond, _), Some(cv)) => {
                    Self::convert_timestamp_ms_column(cv, num_rows)
                }
                (arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::Microsecond, _), Some(cv)) => {
                    Self::convert_timestamp_us_column(cv, num_rows)
                }
                (arrow_schema::DataType::Timestamp(arrow_schema::TimeUnit::Nanosecond, _), Some(cv)) => {
                    Self::convert_timestamp_ns_column(cv, num_rows)
                }
                (arrow_schema::DataType::Utf8, Some(cv)) => {
                    Self::convert_string_column(cv, num_rows)
                }
                (arrow_schema::DataType::Binary, Some(cv)) => {
                    Self::convert_binary_column(cv, num_rows)
                }
                (arrow_schema::DataType::Decimal128(precision, scale), Some(cv)) => {
                    Self::convert_decimal_column(cv, num_rows, *precision, *scale)
                }
                _ => {
                    let mut builder = NullBuilder::new();
                    builder.append_nulls(num_rows);
                    Arc::new(builder.finish())
                }
            };

            arrays.push(array);
        }

        Ok(arrays)
    }

    fn convert_bool_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = BooleanBuilder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::Bool(v) => builder.append_value(v),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_tinyint_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = Int8Builder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::TinyInt(v) => builder.append_value(v),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_smallint_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = Int16Builder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::SmallInt(v) => builder.append_value(v),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_int_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = Int32Builder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::Int(v) => builder.append_value(v),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_bigint_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = Int64Builder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::BigInt(v) => builder.append_value(v),
                BorrowedValue::Timestamp(ts) => builder.append_value(ts.as_raw_i64()),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_utinyint_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = UInt8Builder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::UTinyInt(v) => builder.append_value(v),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_usmallint_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = UInt16Builder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::USmallInt(v) => builder.append_value(v),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_uint_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = UInt32Builder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::UInt(v) => builder.append_value(v),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_ubigint_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = UInt64Builder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::UBigInt(v) => builder.append_value(v),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_float_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = Float32Builder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::Float(v) => builder.append_value(v),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_double_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = Float64Builder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::Double(v) => builder.append_value(v),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_timestamp_ms_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = TimestampMillisecondBuilder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::Timestamp(ts) => builder.append_value(ts.as_raw_i64()),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_timestamp_us_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = TimestampMicrosecondBuilder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::Timestamp(ts) => builder.append_value(ts.as_raw_i64()),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_timestamp_ns_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = TimestampNanosecondBuilder::with_capacity(num_rows);
        for value in view.iter() {
            match value {
                BorrowedValue::Timestamp(ts) => builder.append_value(ts.as_raw_i64()),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_string_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = StringBuilder::with_capacity(num_rows, num_rows * 32);
        for value in view.iter() {
            match value {
                BorrowedValue::VarChar(s) => builder.append_value(s),
                BorrowedValue::NChar(s) => builder.append_value(s),
                BorrowedValue::Json(j) => {
                    match std::str::from_utf8(&j) {
                        Ok(s) => builder.append_value(s),
                        Err(_) => builder.append_null(),
                    }
                }
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    fn convert_binary_column(view: &ColumnView, num_rows: usize) -> Arc<dyn arrow_array::Array> {
        let mut builder = GenericBinaryBuilder::<i32>::with_capacity(num_rows, num_rows * 64);
        for value in view.iter() {
            match value {
                BorrowedValue::VarBinary(b) => builder.append_value(b),
                BorrowedValue::Geometry(b) => builder.append_value(b),
                BorrowedValue::Null(_) => builder.append_null(),
                _ => builder.append_null(),
            }
        }
        Arc::new(builder.finish())
    }

    /// Converts decimal column with precomputed scale factors to avoid pow() in hot loop.
    fn convert_decimal_column(
        view: &ColumnView,
        num_rows: usize,
        precision: u8,
        target_scale: i8,
    ) -> Arc<dyn arrow_array::Array> {
        let mut builder = Decimal128Builder::with_capacity(num_rows);
        let max_val = SCALE_FACTORS.get(precision as usize).copied().unwrap_or(i128::MAX);

        for value in view.iter() {
            match value {
                BorrowedValue::Null(_) => builder.append_null(),
                BorrowedValue::Decimal(dec) => {
                    let mantissa = dec.mantissa();
                    let dec_scale = dec.scale() as i8;
                    let scale_diff = (target_scale - dec_scale) as i32;

                    let scaled_value = if scale_diff == 0 {
                        mantissa
                    } else if scale_diff > 0 && (scale_diff as usize) < SCALE_FACTORS.len() {
                        mantissa * SCALE_FACTORS[scale_diff as usize]
                    } else if scale_diff < 0 && ((-scale_diff) as usize) < SCALE_FACTORS.len() {
                        mantissa / SCALE_FACTORS[(-scale_diff) as usize]
                    } else {
                        mantissa
                    };

                    if scaled_value.abs() < max_val {
                        builder.append_value(scaled_value);
                    } else {
                        builder.append_null();
                    }
                }
                _ => builder.append_null(),
            }
        }

        let array = builder.finish();
        Arc::new(array.with_data_type(arrow_schema::DataType::Decimal128(precision, target_scale)))
    }
}

impl arrow_array::RecordBatchReader for TaosRecordBatchReader {
    fn schema(&self) -> SchemaRef {
        Arc::clone(&self.schema)
    }
}

impl Iterator for TaosRecordBatchReader {
    type Item = std::result::Result<arrow_array::RecordBatch, arrow_schema::ArrowError>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut result_set = self.result_set.take()?;

        // Use blocks() for direct columnar access instead of rows()
        let block = match result_set.blocks().next() {
            Some(Ok(block)) => block,
            Some(Err(e)) => {
                self.result_set = None;
                return Some(Err(arrow_schema::ArrowError::from_external_error(
                    format!("Failed to fetch block: {}", e).into(),
                )));
            }
            None => {
                self.result_set = None;
                return None;
            }
        };

        self.result_set = Some(result_set);

        if block.nrows() == 0 {
            self.result_set = None;
            return None;
        }

        match self.convert_block_to_arrays(&block) {
            Ok(arrays) => {
                match RecordBatch::try_new(Arc::clone(&self.schema), arrays) {
                    Ok(batch) => Some(Ok(batch)),
                    Err(e) => Some(Err(e)),
                }
            }
            Err(e) => Some(Err(e)),
        }
    }
}

// Rust guideline compliant 2026-01-03
