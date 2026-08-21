//! Type conversion between TDengine and Apache Arrow.
//!
//! This module provides conversion functions for mapping TDengine data types
//! to Arrow schema types and converting raw data to Arrow arrays.

use arrow_array::{Array, BooleanArray, Float32Array, Float64Array, StringArray};
use arrow_array::{BinaryArray, Int16Array, Int32Array, Int64Array, Int8Array};
use arrow_array::{TimestampMicrosecondArray, TimestampMillisecondArray, TimestampNanosecondArray};
use arrow_schema::{DataType, TimeUnit};
use std::sync::Arc;

use crate::error::TaosError;

/// TDengine field data type.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TaosDataType {
    Bool,
    TinyInt,
    SmallInt,
    Int,
    BigInt,
    Float,
    Double,
    Binary,
    Nchar,
    Timestamp,
    Json,
    Varbinary,
    Geometry,
    Blob,
    TinyIntUnsigned,
    SmallIntUnsigned,
    IntUnsigned,
    BigIntUnsigned,
}

/// Converts TDengine data type to Arrow data type.
///
/// # Arguments
/// * `taos_type` - TDengine data type
/// * `precision` - Timestamp precision (millisecond/microsecond/nanosecond)
pub fn taos_type_to_arrow(taos_type: TaosDataType, precision: Option<TimestampPrecision>) -> DataType {
    match taos_type {
        TaosDataType::Bool => DataType::Boolean,
        TaosDataType::TinyInt => DataType::Int8,
        TaosDataType::SmallInt => DataType::Int16,
        TaosDataType::Int => DataType::Int32,
        TaosDataType::BigInt => DataType::Int64,
        TaosDataType::Float => DataType::Float32,
        TaosDataType::Double => DataType::Float64,
        TaosDataType::Binary | TaosDataType::Varbinary | TaosDataType::Blob | TaosDataType::Geometry => DataType::Binary,
        TaosDataType::Nchar | TaosDataType::Json => DataType::Utf8,
        TaosDataType::Timestamp => {
            match precision.unwrap_or(TimestampPrecision::Millisecond) {
                TimestampPrecision::Millisecond => DataType::Timestamp(TimeUnit::Millisecond, None),
                TimestampPrecision::Microsecond => DataType::Timestamp(TimeUnit::Microsecond, None),
                TimestampPrecision::Nanosecond => DataType::Timestamp(TimeUnit::Nanosecond, None),
            }
        }
        TaosDataType::TinyIntUnsigned => DataType::UInt8,
        TaosDataType::SmallIntUnsigned => DataType::UInt16,
        TaosDataType::IntUnsigned => DataType::UInt32,
        TaosDataType::BigIntUnsigned => DataType::UInt64,
    }
}

/// Timestamp precision for TDengine TIMESTAMP columns.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimestampPrecision {
    Millisecond,
    Microsecond,
    Nanosecond,
}

impl TryFrom<i32> for TimestampPrecision {
    type Error = TaosError;

    fn try_from(value: i32) -> std::result::Result<Self, Self::Error> {
        match value {
            0 => Ok(TimestampPrecision::Millisecond),
            1 => Ok(TimestampPrecision::Microsecond),
            2 => Ok(TimestampPrecision::Nanosecond),
            _ => Err(TaosError::conversion(format!(
                "Invalid timestamp precision: {}",
                value
            ))),
        }
    }
}

/// Column data builder for constructing Arrow arrays from TDengine raw data.
pub enum ColumnData {
    Bool(Vec<bool>),
    Int8(Vec<i8>),
    Int16(Vec<i16>),
    Int32(Vec<i32>),
    Int64(Vec<i64>),
    UInt8(Vec<u8>),
    UInt16(Vec<u16>),
    UInt32(Vec<u32>),
    UInt64(Vec<u64>),
    Float32(Vec<f32>),
    Float64(Vec<f64>),
    Binary(Vec<Vec<u8>>),
    Utf8(Vec<String>),
    TimestampMillisecond(Vec<i64>),
    TimestampMicrosecond(Vec<i64>),
    TimestampNanosecond(Vec<i64>),
}

impl ColumnData {
    /// Returns the number of rows in this column.
    pub fn len(&self) -> usize {
        match self {
            ColumnData::Bool(v) => v.len(),
            ColumnData::Int8(v) => v.len(),
            ColumnData::Int16(v) => v.len(),
            ColumnData::Int32(v) => v.len(),
            ColumnData::Int64(v) => v.len(),
            ColumnData::UInt8(v) => v.len(),
            ColumnData::UInt16(v) => v.len(),
            ColumnData::UInt32(v) => v.len(),
            ColumnData::UInt64(v) => v.len(),
            ColumnData::Float32(v) => v.len(),
            ColumnData::Float64(v) => v.len(),
            ColumnData::Binary(v) => v.len(),
            ColumnData::Utf8(v) => v.len(),
            ColumnData::TimestampMillisecond(v) => v.len(),
            ColumnData::TimestampMicrosecond(v) => v.len(),
            ColumnData::TimestampNanosecond(v) => v.len(),
        }
    }

    /// Returns true if the column has no rows.
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Converts this column data to an Arrow array.
    pub fn to_arrow_array(&self) -> Arc<dyn Array> {
        match self {
            ColumnData::Bool(v) => Arc::new(BooleanArray::from_iter(v.iter().copied())),
            ColumnData::Int8(v) => Arc::new(Int8Array::from_iter(v.iter().copied())),
            ColumnData::Int16(v) => Arc::new(Int16Array::from_iter(v.iter().copied())),
            ColumnData::Int32(v) => Arc::new(Int32Array::from_iter(v.iter().copied())),
            ColumnData::Int64(v) => Arc::new(Int64Array::from_iter(v.iter().copied())),
            ColumnData::UInt8(v) => {
                Arc::new(arrow_array::UInt8Array::from_iter(v.iter().copied()))
            }
            ColumnData::UInt16(v) => {
                Arc::new(arrow_array::UInt16Array::from_iter(v.iter().copied()))
            }
            ColumnData::UInt32(v) => {
                Arc::new(arrow_array::UInt32Array::from_iter(v.iter().copied()))
            }
            ColumnData::UInt64(v) => {
                Arc::new(arrow_array::UInt64Array::from_iter(v.iter().copied()))
            }
            ColumnData::Float32(v) => Arc::new(Float32Array::from_iter(v.iter().copied())),
            ColumnData::Float64(v) => Arc::new(Float64Array::from_iter(v.iter().copied())),
            ColumnData::Binary(v) => {
                Arc::new(BinaryArray::from_iter_values(v.iter().map(|x| x.as_slice())))
            }
            ColumnData::Utf8(v) => Arc::new(StringArray::from_iter_values(v.iter())),
            ColumnData::TimestampMillisecond(v) => {
                Arc::new(TimestampMillisecondArray::from_iter_values(v.iter().copied()))
            }
            ColumnData::TimestampMicrosecond(v) => {
                Arc::new(TimestampMicrosecondArray::from_iter_values(v.iter().copied()))
            }
            ColumnData::TimestampNanosecond(v) => {
                Arc::new(TimestampNanosecondArray::from_iter_values(v.iter().copied()))
            }
        }
    }

    /// Gets the corresponding Arrow data type for this column.
    pub fn data_type(&self) -> DataType {
        match self {
            ColumnData::Bool(_) => DataType::Boolean,
            ColumnData::Int8(_) => DataType::Int8,
            ColumnData::Int16(_) => DataType::Int16,
            ColumnData::Int32(_) => DataType::Int32,
            ColumnData::Int64(_) => DataType::Int64,
            ColumnData::UInt8(_) => DataType::UInt8,
            ColumnData::UInt16(_) => DataType::UInt16,
            ColumnData::UInt32(_) => DataType::UInt32,
            ColumnData::UInt64(_) => DataType::UInt64,
            ColumnData::Float32(_) => DataType::Float32,
            ColumnData::Float64(_) => DataType::Float64,
            ColumnData::Binary(_) => DataType::Binary,
            ColumnData::Utf8(_) => DataType::Utf8,
            ColumnData::TimestampMillisecond(_) => {
                DataType::Timestamp(TimeUnit::Millisecond, None)
            }
            ColumnData::TimestampMicrosecond(_) => {
                DataType::Timestamp(TimeUnit::Microsecond, None)
            }
            ColumnData::TimestampNanosecond(_) => DataType::Timestamp(TimeUnit::Nanosecond, None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_taos_type_to_arrow_basic() {
        assert_eq!(taos_type_to_arrow(TaosDataType::Bool, None), DataType::Boolean);
        assert_eq!(taos_type_to_arrow(TaosDataType::TinyInt, None), DataType::Int8);
        assert_eq!(taos_type_to_arrow(TaosDataType::SmallInt, None), DataType::Int16);
        assert_eq!(taos_type_to_arrow(TaosDataType::Int, None), DataType::Int32);
        assert_eq!(taos_type_to_arrow(TaosDataType::BigInt, None), DataType::Int64);
        assert_eq!(taos_type_to_arrow(TaosDataType::Float, None), DataType::Float32);
        assert_eq!(taos_type_to_arrow(TaosDataType::Double, None), DataType::Float64);
    }

    #[test]
    fn test_taos_type_to_arrow_unsigned() {
        assert_eq!(taos_type_to_arrow(TaosDataType::TinyIntUnsigned, None), DataType::UInt8);
        assert_eq!(taos_type_to_arrow(TaosDataType::SmallIntUnsigned, None), DataType::UInt16);
        assert_eq!(taos_type_to_arrow(TaosDataType::IntUnsigned, None), DataType::UInt32);
        assert_eq!(taos_type_to_arrow(TaosDataType::BigIntUnsigned, None), DataType::UInt64);
    }

    #[test]
    fn test_taos_type_to_arrow_string_binary() {
        assert_eq!(taos_type_to_arrow(TaosDataType::Binary, None), DataType::Binary);
        assert_eq!(taos_type_to_arrow(TaosDataType::Varbinary, None), DataType::Binary);
        assert_eq!(taos_type_to_arrow(TaosDataType::Nchar, None), DataType::Utf8);
        assert_eq!(taos_type_to_arrow(TaosDataType::Json, None), DataType::Utf8);
    }

    #[test]
    fn test_taos_type_to_arrow_timestamp() {
        assert_eq!(
            taos_type_to_arrow(TaosDataType::Timestamp, Some(TimestampPrecision::Millisecond)),
            DataType::Timestamp(TimeUnit::Millisecond, None)
        );
        assert_eq!(
            taos_type_to_arrow(TaosDataType::Timestamp, Some(TimestampPrecision::Microsecond)),
            DataType::Timestamp(TimeUnit::Microsecond, None)
        );
        assert_eq!(
            taos_type_to_arrow(TaosDataType::Timestamp, Some(TimestampPrecision::Nanosecond)),
            DataType::Timestamp(TimeUnit::Nanosecond, None)
        );
        assert_eq!(
            taos_type_to_arrow(TaosDataType::Timestamp, None),
            DataType::Timestamp(TimeUnit::Millisecond, None)
        );
    }

    #[test]
    fn test_timestamp_precision_try_from() {
        assert_eq!(TimestampPrecision::try_from(0).unwrap(), TimestampPrecision::Millisecond);
        assert_eq!(TimestampPrecision::try_from(1).unwrap(), TimestampPrecision::Microsecond);
        assert_eq!(TimestampPrecision::try_from(2).unwrap(), TimestampPrecision::Nanosecond);
        assert!(TimestampPrecision::try_from(3).is_err());
    }

    #[test]
    fn test_column_data_len() {
        let col = ColumnData::Int32(vec![1, 2, 3]);
        assert_eq!(col.len(), 3);
        assert!(!col.is_empty());

        let empty_col = ColumnData::Int32(vec![]);
        assert_eq!(empty_col.len(), 0);
        assert!(empty_col.is_empty());
    }

    #[test]
    fn test_column_data_to_arrow_array() {
        let col = ColumnData::Int32(vec![1, 2, 3]);
        let arr = col.to_arrow_array();
        assert_eq!(arr.len(), 3);
        assert_eq!(col.data_type(), DataType::Int32);

        let str_col = ColumnData::Utf8(vec!["a".to_string(), "b".to_string()]);
        let str_arr = str_col.to_arrow_array();
        assert_eq!(str_arr.len(), 2);
        assert_eq!(str_col.data_type(), DataType::Utf8);
    }
}
