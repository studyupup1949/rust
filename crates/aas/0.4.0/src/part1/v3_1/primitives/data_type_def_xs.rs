use crate::part1::v3_1::primitives::Iri;
use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};
use serde_with::DisplayFromStr;
use serde_with::serde_as;
use strum::{Display, EnumString};
use thiserror::Error;
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Type mapping of XSDef types.
#[serde_as]
#[derive(Clone, PartialEq, Debug, Display, Deserialize, Serialize)]
#[strum(prefix = "xs:", serialize_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(feature = "openapi", schema(as = String))]
pub enum DataTypeXSDef {
    // basic types
    #[serde(rename = "xs:int")]
    Int,
    #[serde(rename = "xs:long")]
    Long,
    #[serde(rename = "xs:integer")]
    Integer,

    #[serde(rename = "xs:negativeInteger")]
    NegativeInteger,

    #[serde(rename = "xs:nonNegativeInteger")]
    NonNegativeInteger,

    #[serde(rename = "xs:nonPositiveInteger")]
    NonPositiveInteger,

    #[serde(rename = "xs:positiveInteger")]
    PositiveInteger,

    #[serde(rename = "xs:short")]
    Short,

    #[serde(rename = "xs:string")]
    String,

    #[serde(rename = "xs:boolean")]
    Boolean,
    #[serde(rename = "xs:byte")]
    Byte,

    #[serde(rename = "xs:unsignedByte")]
    UnsignedByte,

    #[serde(rename = "xs:unsignedInt")]
    UnsignedInt,

    #[serde(rename = "xs:unsignedLong")]
    UnsignedLong,

    #[serde(rename = "xs:unsignedShort")]
    UnsignedShort,

    #[serde(rename = "xs:decimal")]
    Decimal,

    #[serde(rename = "xs:float")]
    Float,

    #[serde(rename = "xs:double")]
    Double,

    // Date Time related
    #[serde(rename = "xs:time")]
    Time,

    #[serde(rename = "xs:date")]
    Date,

    #[serde(rename = "xs:dateTime")]
    DateTime,

    #[serde(rename = "xs:duration")]
    Duration,

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gDay")]
    GDay,

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gMonth")]
    GMonth,

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gMonthDay")]
    GMonthDay,

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gYear")]
    GYear,

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gYearMonth")]
    GYearMonth,

    // binary
    #[serde(rename = "xs:base64Binary")]
    Base64Binary,

    #[serde(rename = "xs:hexBinary")]
    HexBinary,

    // Miscellaneous types
    /// URI and IRI possible
    #[serde(rename = "xs:anyURI")]
    AnyURI,
}

#[derive(Debug, Clone, Error)]
pub enum ConversionError {
    #[error("Value is not parseable")]
    ParseError,
}

impl TryFrom<(DataTypeXSDef, Option<String>)> for DataXsd {
    type Error = ConversionError;

    fn try_from(value: (DataTypeXSDef, Option<String>)) -> Result<Self, Self::Error> {
        match value.0 {
            DataTypeXSDef::Int => Ok(DataXsd::Int(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::Long => Ok(DataXsd::Long(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::Integer => Ok(DataXsd::Integer(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::NegativeInteger => Ok(DataXsd::NegativeInteger(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::NonNegativeInteger => Ok(DataXsd::NonNegativeInteger(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::NonPositiveInteger => Ok(DataXsd::NonPositiveInteger(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::PositiveInteger => Ok(DataXsd::PositiveInteger(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::Short => Ok(DataXsd::Short(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::String => Ok(DataXsd::String(value.1)),
            DataTypeXSDef::Boolean => Ok(DataXsd::Boolean(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::Byte => Ok(DataXsd::Byte(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::UnsignedByte => Ok(DataXsd::UnsignedByte(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::UnsignedInt => Ok(DataXsd::UnsignedInt(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::UnsignedLong => Ok(DataXsd::UnsignedLong(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::UnsignedShort => Ok(DataXsd::UnsignedShort(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::Decimal => Ok(DataXsd::Decimal(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::Float => Ok(DataXsd::Float(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::Double => Ok(DataXsd::Double(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::Time => Ok(DataXsd::Time(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::Date => Ok(DataXsd::Date(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::DateTime => Ok(DataXsd::DateTime(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::Duration => Ok(DataXsd::Duration(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::GDay => Ok(DataXsd::GDay(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::GMonth => Ok(DataXsd::GMonth(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::GMonthDay => Ok(DataXsd::GMonthDay(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::GYear => Ok(DataXsd::GYear(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::GYearMonth => Ok(DataXsd::GYearMonth(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
            DataTypeXSDef::Base64Binary => {
                Ok(DataXsd::Base64Binary(value.1.map(|v| v.into_bytes())))
            }
            DataTypeXSDef::HexBinary => Ok(DataXsd::HexBinary(value.1.map(|v| v.into_bytes()))),
            DataTypeXSDef::AnyURI => Ok(DataXsd::AnyURI(
                value
                    .1
                    .map(|v| v.parse().map_err(|_| ConversionError::ParseError))
                    .transpose()?,
            )),
        }
    }
}

impl From<DataXsd> for Option<String> {
    fn from(value: DataXsd) -> Self {
        match value {
            DataXsd::Int(v) => v.map(|v| v.to_string()),
            DataXsd::Long(v) => v.map(|v| v.to_string()),
            DataXsd::Integer(v) => v.map(|v| v.to_string()),
            DataXsd::NegativeInteger(v) => v.map(|v| v.to_string()),
            DataXsd::NonNegativeInteger(v) => v.map(|v| v.to_string()),
            DataXsd::NonPositiveInteger(v) => v.map(|v| v.to_string()),
            DataXsd::PositiveInteger(v) => v.map(|v| v.to_string()),
            DataXsd::Short(v) => v.map(|v| v.to_string()),
            DataXsd::String(v) => v,
            DataXsd::Boolean(v) => v.map(|v| v.to_string()),
            DataXsd::Byte(v) => v.map(|v| v.to_string()),
            DataXsd::UnsignedByte(v) => v.map(|v| v.to_string()),
            DataXsd::UnsignedInt(v) => v.map(|v| v.to_string()),
            DataXsd::UnsignedLong(v) => v.map(|v| v.to_string()),
            DataXsd::UnsignedShort(v) => v.map(|v| v.to_string()),
            DataXsd::Decimal(v) => v.map(|v| v.to_string()),
            DataXsd::Float(v) => v.map(|v| v.to_string()),
            DataXsd::Double(v) => v.map(|v| v.to_string()),
            DataXsd::Time(v) => v.map(|v| v.to_string()),
            DataXsd::Date(v) => v.map(|v| v.to_string()),
            DataXsd::DateTime(v) => v.map(|v| v.to_string()),
            DataXsd::Duration(v) => v.map(|v| v.to_string()),
            DataXsd::GDay(v) => v.map(|v| v.to_string()),
            DataXsd::GMonth(v) => v.map(|v| v.to_string()),
            DataXsd::GMonthDay(v) => v.map(|v| v.to_string()),
            DataXsd::GYear(v) => v.map(|v| v.to_string()),
            DataXsd::GYearMonth(v) => v.map(|v| v.to_string()),
            DataXsd::Base64Binary(v) => v.map(|v| String::from_utf8_lossy(&v).to_string()),
            DataXsd::HexBinary(v) => v.map(|v| String::from_utf8_lossy(&v).to_string()),
            DataXsd::AnyURI(v) => v.map(|v| v.to_string()),
        }
    }
}

impl From<DataXsd> for DataTypeXSDef {
    fn from(value: DataXsd) -> Self {
        match value {
            DataXsd::Int(_) => DataTypeXSDef::Int,
            DataXsd::Long(_) => DataTypeXSDef::Long,
            DataXsd::Integer(_) => DataTypeXSDef::Integer,
            DataXsd::NegativeInteger(_) => DataTypeXSDef::NegativeInteger,
            DataXsd::NonNegativeInteger(_) => DataTypeXSDef::NonNegativeInteger,
            DataXsd::NonPositiveInteger(_) => DataTypeXSDef::NonPositiveInteger,
            DataXsd::PositiveInteger(_) => DataTypeXSDef::PositiveInteger,
            DataXsd::Short(_) => DataTypeXSDef::Short,
            DataXsd::String(_) => DataTypeXSDef::String,
            DataXsd::Boolean(_) => DataTypeXSDef::Boolean,
            DataXsd::Byte(_) => DataTypeXSDef::Byte,
            DataXsd::UnsignedByte(_) => DataTypeXSDef::UnsignedByte,
            DataXsd::UnsignedInt(_) => DataTypeXSDef::UnsignedInt,
            DataXsd::UnsignedLong(_) => DataTypeXSDef::UnsignedLong,
            DataXsd::UnsignedShort(_) => DataTypeXSDef::UnsignedShort,
            DataXsd::Decimal(_) => DataTypeXSDef::Decimal,
            DataXsd::Float(_) => DataTypeXSDef::Float,
            DataXsd::Double(_) => DataTypeXSDef::Double,
            DataXsd::Time(_) => DataTypeXSDef::Time,
            DataXsd::Date(_) => DataTypeXSDef::Date,
            DataXsd::DateTime(_) => DataTypeXSDef::DateTime,
            DataXsd::Duration(_) => DataTypeXSDef::Duration,
            DataXsd::GDay(_) => DataTypeXSDef::GDay,
            DataXsd::GMonth(_) => DataTypeXSDef::GMonth,
            DataXsd::GMonthDay(_) => DataTypeXSDef::GMonthDay,
            DataXsd::GYear(_) => DataTypeXSDef::GYear,
            DataXsd::GYearMonth(_) => DataTypeXSDef::GYearMonth,
            DataXsd::Base64Binary(_) => DataTypeXSDef::Base64Binary,
            DataXsd::HexBinary(_) => DataTypeXSDef::HexBinary,
            DataXsd::AnyURI(_) => DataTypeXSDef::AnyURI,
        }
    }
}

/// represents the valueType/value pair typesafe. Used i.e. by Extension or Property.
/// ValueType has to be always present, value can be optional.
/// Default: String(None)
#[serde_as]
#[derive(Clone, PartialEq, Debug, Display, Deserialize, Serialize, EnumString)]
#[serde(tag = "valueType", content = "value")]
#[strum(prefix = "xs:", serialize_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum DataXsd {
    // basic types
    #[serde(rename = "xs:int")]
    Int(Option<i32>),

    #[serde(rename = "xs:long")]
    Long(Option<i64>),

    #[serde(rename = "xs:integer")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    Integer(Option<BigDecimal>),

    #[serde(rename = "xs:negativeInteger")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    NegativeInteger(Option<BigDecimal>),

    #[serde(rename = "xs:nonNegativeInteger")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    NonNegativeInteger(Option<BigDecimal>),

    #[serde(rename = "xs:nonPositiveInteger")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    NonPositiveInteger(Option<BigDecimal>),

    #[serde(rename = "xs:positiveInteger")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    PositiveInteger(Option<BigDecimal>),

    #[serde(rename = "xs:short")]
    Short(Option<u16>),

    #[serde(rename = "xs:string")]
    String(Option<String>),

    #[serde(rename = "xs:boolean")]
    Boolean(Option<bool>),
    #[serde(rename = "xs:byte")]
    Byte(Option<i8>),

    #[serde(rename = "xs:unsignedByte")]
    UnsignedByte(Option<u8>),

    #[serde(rename = "xs:unsignedInt")]
    UnsignedInt(Option<u32>),

    #[serde(rename = "xs:unsignedLong")]
    UnsignedLong(Option<u64>),

    #[serde(rename = "xs:unsignedShort")]
    UnsignedShort(Option<u16>),

    #[serde(rename = "xs:decimal")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    Decimal(Option<BigDecimal>),

    #[serde(rename = "xs:float")]
    Float(#[serde_as(as = "Option<DisplayFromStr>")] Option<f32>),

    #[serde(rename = "xs:double")]
    Double(#[serde_as(as = "Option<DisplayFromStr>")] Option<f64>),

    #[serde(rename = "xs:time")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    Time(Option<iso8601::Time>),

    #[serde(rename = "xs:date")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    Date(Option<iso8601::Date>),

    #[serde(rename = "xs:dateTime")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    DateTime(Option<iso8601::DateTime>),

    #[serde(rename = "xs:duration")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    Duration(Option<iso8601::Duration>),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gDay")]
    GDay(Option<String>),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gMonth")]
    GMonth(Option<String>),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gMonthDay")]
    GMonthDay(Option<String>),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gYear")]
    GYear(Option<String>),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gYearMonth")]
    GYearMonth(Option<String>),

    // binary
    #[serde(rename = "xs:base64Binary")]
    Base64Binary(Option<Vec<u8>>),

    #[serde(rename = "xs:hexBinary")]
    HexBinary(Option<Vec<u8>>),

    // Miscellaneous types
    /// URI and IRI possible
    #[serde(rename = "xs:anyURI")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    AnyURI(Option<Iri>),
}

impl Default for DataXsd {
    fn default() -> Self {
        DataXsd::String(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_xs_string() {
        let json = r#""xs:string""#;

        serde_json::from_str::<DataTypeXSDef>(&json).unwrap();
    }

    #[test]
    fn deserialize_double_from_string() {
        let json = r#"{
            "valueType": "xs:double",
            "value": "1.2"
        }"#;

        let expected = DataXsd::Double(Some(1.2));
        let actual: DataXsd = serde_json::from_str(json).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn deserialize_duration() {
        let json = r#"{
            "valueType": "xs:duration",
            "value": "P1Y"
        }"#;

        let expected = DataXsd::Duration(Some(iso8601::Duration::YMDHMS {
            year: 1,
            month: 0,
            day: 0,
            hour: 0,
            minute: 0,
            second: 0,
            millisecond: 0,
        }));
        let actual: DataXsd = serde_json::from_str(json).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn deserialize_naive_date_time() {
        let json = r#"{
            "valueType": "xs:dateTime",
            "value": "2001-10-26T21:32:52"
        }"#;

        let expected = DataXsd::DateTime(Some(iso8601::DateTime {
            date: iso8601::Date::YMD {
                year: 2001,
                month: 10,
                day: 26,
            },
            time: iso8601::Time {
                hour: 21,
                minute: 32,
                second: 52,
                millisecond: 0,
                tz_offset_hours: 0,
                tz_offset_minutes: 0,
            },
        }));
        let actual: DataXsd = serde_json::from_str(json).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn deserialize_naive_date_time_with_zone() {
        let json = r#"{
            "valueType": "xs:dateTime",
            "value": "2001-10-26T21:32:52Z"
        }"#;

        let expected = DataXsd::DateTime(Some(iso8601::DateTime {
            date: iso8601::Date::YMD {
                year: 2001,
                month: 10,
                day: 26,
            },
            time: iso8601::Time {
                hour: 21,
                minute: 32,
                second: 52,
                millisecond: 0,
                tz_offset_hours: 0,
                tz_offset_minutes: 0,
            },
        }));

        let actual: DataXsd = serde_json::from_str(json).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn deserialize_time_with_zone() {
        let json = r#"{
            "valueType": "xs:time",
            "value": "21:32:52Z"
        }"#;

        let expected = DataXsd::Time(Some(iso8601::Time {
            hour: 21,
            minute: 32,
            second: 52,
            millisecond: 0,
            tz_offset_hours: 0,
            tz_offset_minutes: 0,
        }));
        let actual: DataXsd = serde_json::from_str(json).unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn deserialize_date_with_zone() {
        let json = r#"{
            "valueType": "xs:date",
            "value": "2001-10-26Z"
        }"#;

        let expected = DataXsd::Date(Some(iso8601::Date::YMD {
            year: 2001,
            month: 10,
            day: 26,
        }));
        let actual: DataXsd = serde_json::from_str(json).unwrap();

        assert_eq!(expected, actual);
    }
}
