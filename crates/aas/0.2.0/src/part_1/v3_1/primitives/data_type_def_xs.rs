use crate::part_1::v3_1::primitives::Iri;
use bigdecimal::BigDecimal;
use chrono::{DateTime, NaiveDate, NaiveTime, Utc};
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

/// Type mapping of XSDef types.
#[derive(Clone, PartialEq, Debug, Display, Deserialize, Serialize)]
#[strum(prefix = "xs:", serialize_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub enum DataTypeXSDef {
    // basic types
    #[serde(rename = "xs:int")]
    Int(i32),

    #[serde(rename = "xs:long")]
    Long(i64),

    #[serde(rename = "xs:integer")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    Integer(BigDecimal),

    #[serde(rename = "xs:negativeInteger")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    NegativeInteger(BigDecimal),

    #[serde(rename = "xs:nonNegativeInteger")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    NonNegativeInteger(BigDecimal),

    #[serde(rename = "xs:nonPositiveInteger")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    NonPositiveInteger(BigDecimal),

    #[serde(rename = "xs:positiveInteger")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    PositiveInteger(BigDecimal),

    #[serde(rename = "xs:short")]
    Short(u16),

    #[serde(rename = "xs:string")]
    String(String),

    #[serde(rename = "xs:boolean")]
    Boolean(bool),
    #[serde(rename = "xs:byte")]
    Byte(i8),

    #[serde(rename = "xs:unsignedByte")]
    UnsignedByte(u8),

    #[serde(rename = "xs:unsignedInt")]
    UnsignedInt(u32),

    #[serde(rename = "xs:unsignedLong")]
    UnsignedLong(u64),

    #[serde(rename = "xs:unsignedShort")]
    UnsignedShort(u16),

    #[serde(rename = "xs:decimal")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    Decimal(BigDecimal),

    #[serde(rename = "xs:float")]
    Float(f32),

    #[serde(rename = "xs:double")]
    Double(f64),

    // Date Time related
    // TODO: TIMEZONES?
    #[serde(rename = "xs:time")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    Time(NaiveTime),

    // TODO: TIMEZONES?
    #[serde(rename = "xs:date")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    Date(NaiveDate),

    #[serde(rename = "xs:dateTime")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    DateTime(DateTime<Utc>),

    /// TODO: using proper type
    #[serde(rename = "xs:duration")]
    Duration(String),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gDay")]
    GDay(String),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gMonth")]
    GMonth(String),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gMonthDay")]
    GMonthDay(String),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gYear")]
    GYear(String),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gYearMonth")]
    GYearMonth(String),

    // binary
    #[serde(rename = "xs:base64Binary")]
    Base64Binary(Vec<u8>),

    #[serde(rename = "xs:hexBinary")]
    HexBinary(Vec<u8>),

    // Miscellaneous types
    /// URI and IRI possible
    #[serde(rename = "xs:anyURI")]
    #[cfg_attr(feature = "openapi", schema(value_type = String))]
    AnyURI(Iri),
}

/// represents the valueType/value pair typesafe. Used i.e. by Extension or Property.
/// ValueType has to be always present, value can be optional.
/// Default: String(None)
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
    Float(Option<f32>),

    #[serde(rename = "xs:double")]
    Double(Option<f64>),

    // Date Time related
    // TODO: TIMEZONES?
    #[serde(rename = "xs:time")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    Time(Option<NaiveTime>),

    // TODO: TIMEZONES?
    #[serde(rename = "xs:date")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    Date(Option<NaiveDate>),

    /// TODO: using proper type
    #[serde(rename = "xs:dateTime")]
    #[cfg_attr(feature = "openapi", schema(value_type = Option<String>))]
    DateTime(Option<DateTime<Utc>>),

    /// TODO: using proper type
    #[serde(rename = "xs:duration")]
    Duration(Option<String>),

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
}
