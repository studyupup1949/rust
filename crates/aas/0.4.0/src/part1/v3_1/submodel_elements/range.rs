use crate::part1::ToJsonMetamodel;
use bigdecimal::BigDecimal;
use iref::IriRefBuf;
use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};
#[cfg(feature = "openapi")]
use utoipa::ToSchema;

// TODO: If the min value is missing, the value is assumed to be negative infinite.
// TODO: If the max value is missing, the value is assumed to be positive infinite.
#[derive(Clone, PartialEq, Debug, Deserialize, Serialize, Default)]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
pub struct RangeInner<T> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min: Option<T>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max: Option<T>,
}

// TODO: update to big decimal
// TODO: Only allow xsd atomic types.
#[derive(Clone, PartialEq, Debug, Display, Deserialize, Serialize, EnumString)]
#[serde(tag = "valueType")]
#[strum(prefix = "xs:", serialize_all = "camelCase")]
#[cfg_attr(feature = "openapi", derive(ToSchema))]
#[cfg_attr(
    feature = "xml",
    serde(try_from = "xml::RangeXMLProxy", into = "xml::RangeXMLProxy")
)]
pub enum Range {
    // basic types
    #[serde(rename = "xs:int")]
    Int(RangeInner<i32>),

    #[serde(rename = "xs:integer")]
    Integer(RangeInner<i32>),

    #[serde(rename = "xs:long")]
    Long(RangeInner<i64>),

    #[serde(rename = "xs:negativeInteger")]
    NegativeInteger(RangeInner<i32>),

    #[serde(rename = "xs:nonNegativeInteger")]
    NonNegativeInteger(RangeInner<u32>),

    #[serde(rename = "xs:nonPositiveInteger")]
    NonPositiveInteger(RangeInner<i32>),

    #[serde(rename = "xs:positiveInteger")]
    PositiveInteger(RangeInner<u32>),

    #[serde(rename = "xs:short")]
    Short(RangeInner<u16>),

    #[serde(rename = "xs:string")]
    String(RangeInner<String>),

    #[serde(rename = "xs:boolean")]
    Boolean(RangeInner<bool>),

    #[serde(rename = "xs:byte")]
    Byte(RangeInner<i8>),

    #[serde(rename = "xs:unsignedByte")]
    UnsignedByte(RangeInner<u8>),

    #[serde(rename = "xs:unsignedInt")]
    UnsignedInt(RangeInner<u32>),

    #[serde(rename = "xs:unsignedLong")]
    UnsignedLong(RangeInner<u64>),

    #[serde(rename = "xs:unsignedShort")]
    UnsignedShort(RangeInner<u16>),

    #[serde(rename = "xs:decimal")]
    #[cfg_attr(feature = "openapi", schema(value_type = RangeInner<String>))]
    Decimal(RangeInner<BigDecimal>),

    #[serde(rename = "xs:float")]
    Float(RangeInner<f32>),

    #[serde(rename = "xs:double")]
    Double(RangeInner<f64>),

    // Date Time related
    #[serde(rename = "xs:time")]
    #[cfg_attr(feature = "openapi", schema(value_type = RangeInner<String>))]
    Time(RangeInner<iso8601::Time>),

    #[serde(rename = "xs:date")]
    #[cfg_attr(feature = "openapi", schema(value_type = RangeInner<String>))]
    Date(RangeInner<iso8601::Date>),

    #[serde(rename = "xs:dateTime")]
    #[cfg_attr(feature = "openapi", schema(value_type = RangeInner<String>))]
    DateTime(RangeInner<iso8601::DateTime>),

    /// TODO: using proper type
    #[serde(rename = "xs:duration")]
    Duration(RangeInner<String>),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gDay")]
    GDay(RangeInner<String>),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gMonth")]
    GMonth(RangeInner<String>),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gMonthDay")]
    GMonthDay(RangeInner<String>),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gYear")]
    GYear(RangeInner<String>),

    /// TODO: using proper type or parsing
    #[serde(rename = "xs:gYearMonth")]
    GYearMonth(RangeInner<String>),

    // binary
    #[serde(rename = "xs:base64Binary")]
    Base64Binary(RangeInner<Vec<u8>>),

    #[serde(rename = "xs:hexBinary")]
    HexBinary(RangeInner<Vec<u8>>),

    // string related
    // TODO: is this supported??
    #[serde(rename = "xs:anyURI")]
    #[cfg_attr(feature = "openapi", schema(value_type = RangeInner<String>))]
    AnyURI(RangeInner<IriRefBuf>),
}

impl ToJsonMetamodel for Range {
    type Error = ();

    fn to_json_metamodel(&self) -> Result<String, Self::Error> {
        Ok(format!(r#"{{"valueType":"{}"}}"#, self.to_string()))
    }
}

pub(crate) mod xml {
    use crate::part1::v3_1::primitives::data_type_def_xs::DataTypeXSDef;
    use crate::part1::v3_1::submodel_elements::range::{Range, RangeInner};
    use serde::{Deserialize, Serialize};
    use thiserror::Error;

    #[derive(Serialize, Deserialize)]
    pub struct RangeXMLProxy {
        #[serde(skip_serializing_if = "Option::is_none")]
        pub min: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        pub max: Option<String>,

        #[serde(rename = "valueType")]
        value_type: DataTypeXSDef,
    }

    #[derive(Debug, Error)]
    pub enum ConversionError {
        #[error("Invalid integer")]
        ParseIntError(#[from] std::num::ParseIntError),
        #[error("Invalid boolean")]
        ParseBoolError(#[from] std::str::ParseBoolError),
        #[error("Invalid Decimal")]
        ParseDecimalError(#[from] bigdecimal::ParseBigDecimalError),
        #[error("Invalid Float")]
        ParseFloatError(#[from] std::num::ParseFloatError),
        #[error("Invalid Iri")]
        ParseUriError(#[from] iref::iri::InvalidIriRef<String>),

        #[error("Invalid: {0}")]
        Parse(String),
    }

    impl TryFrom<RangeXMLProxy> for Range {
        type Error = ConversionError;

        fn try_from(value: RangeXMLProxy) -> Result<Self, Self::Error> {
            let val = match value.value_type {
                DataTypeXSDef::Int => Range::Int(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse::<i32>().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse::<i32>().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::Long => Range::Long(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse::<i64>().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse::<i64>().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::Integer => Range::Integer(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse::<i32>().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse::<i32>().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::NegativeInteger => Range::NegativeInteger(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse::<i32>().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse::<i32>().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::NonNegativeInteger => Range::NonNegativeInteger(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::NonPositiveInteger => Range::NonPositiveInteger(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::PositiveInteger => Range::PositiveInteger(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::Short => Range::Short(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::String => Range::String(RangeInner {
                    min: value.min,
                    max: value.max,
                }),
                DataTypeXSDef::Boolean => Range::Boolean(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseBoolError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseBoolError))
                        .transpose()?,
                }),
                DataTypeXSDef::Byte => Range::Byte(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::UnsignedByte => Range::UnsignedByte(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::UnsignedInt => Range::UnsignedInt(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::UnsignedLong => Range::UnsignedLong(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::UnsignedShort => Range::UnsignedShort(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseIntError))
                        .transpose()?,
                }),
                DataTypeXSDef::Decimal => Range::Decimal(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseDecimalError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseDecimalError))
                        .transpose()?,
                }),
                DataTypeXSDef::Float => Range::Float(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseFloatError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseFloatError))
                        .transpose()?,
                }),
                DataTypeXSDef::Double => Range::Double(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseFloatError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseFloatError))
                        .transpose()?,
                }),
                DataTypeXSDef::Time => Range::Time(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::Parse))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::Parse))
                        .transpose()?,
                }),
                DataTypeXSDef::Date => Range::Date(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::Parse))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::Parse))
                        .transpose()?,
                }),
                DataTypeXSDef::DateTime => Range::DateTime(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::Parse))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::Parse))
                        .transpose()?,
                }),
                DataTypeXSDef::Duration => Range::Duration(RangeInner {
                    min: value.min,
                    max: value.max,
                }),
                DataTypeXSDef::GDay => Range::GDay(RangeInner {
                    min: value.min,
                    max: value.max,
                }),
                DataTypeXSDef::GMonth => Range::GMonth(RangeInner {
                    min: value.min,
                    max: value.max,
                }),
                DataTypeXSDef::GMonthDay => Range::GMonthDay(RangeInner {
                    min: value.min,
                    max: value.max,
                }),
                DataTypeXSDef::GYear => Range::GYear(RangeInner {
                    min: value.min,
                    max: value.max,
                }),
                DataTypeXSDef::GYearMonth => Range::GYearMonth(RangeInner {
                    min: value.min,
                    max: value.max,
                }),
                DataTypeXSDef::Base64Binary => Range::Base64Binary(RangeInner {
                    min: value.min.map(|v| v.into_bytes()),
                    max: value.max.map(|v| v.into_bytes()),
                }),
                DataTypeXSDef::HexBinary => Range::HexBinary(RangeInner {
                    min: value.min.map(|v| v.into_bytes()),
                    max: value.max.map(|v| v.into_bytes()),
                }),
                DataTypeXSDef::AnyURI => Range::AnyURI(RangeInner {
                    min: value
                        .min
                        .map(|v| v.parse().map_err(ConversionError::ParseUriError))
                        .transpose()?,
                    max: value
                        .max
                        .map(|v| v.parse().map_err(ConversionError::ParseUriError))
                        .transpose()?,
                }),
            };

            Ok(val)
        }
    }

    impl From<Range> for RangeXMLProxy {
        fn from(value: Range) -> Self {
            match value {
                Range::Int(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Int,
                },
                Range::Integer(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Integer,
                },
                Range::Long(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Long,
                },
                Range::NegativeInteger(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::NegativeInteger,
                },
                Range::NonNegativeInteger(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::NonNegativeInteger,
                },
                Range::NonPositiveInteger(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::NonPositiveInteger,
                },
                Range::PositiveInteger(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::PositiveInteger,
                },
                Range::Short(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Short,
                },
                Range::String(v) => Self {
                    min: v.min,
                    max: v.max,
                    value_type: DataTypeXSDef::String,
                },
                Range::Boolean(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Boolean,
                },
                Range::Byte(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Byte,
                },
                Range::UnsignedByte(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::UnsignedByte,
                },
                Range::UnsignedInt(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::UnsignedInt,
                },
                Range::UnsignedLong(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::UnsignedLong,
                },
                Range::UnsignedShort(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Boolean,
                },
                Range::Decimal(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::UnsignedShort,
                },
                Range::Float(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Float,
                },
                Range::Double(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Double,
                },
                Range::Time(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Time,
                },
                Range::Date(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Date,
                },
                Range::DateTime(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::DateTime,
                },
                Range::Duration(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::Duration,
                },
                Range::GDay(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::GDay,
                },
                Range::GMonth(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::GMonth,
                },
                Range::GMonthDay(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::GMonthDay,
                },
                Range::GYear(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::GYear,
                },
                Range::GYearMonth(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::GYearMonth,
                },
                Range::Base64Binary(v) => Self {
                    min: v.min.map(|v| unsafe { String::from_utf8_unchecked(v) }),
                    max: v.max.map(|v| unsafe { String::from_utf8_unchecked(v) }),
                    value_type: DataTypeXSDef::Base64Binary,
                },
                Range::HexBinary(v) => Self {
                    min: v.min.map(|v| unsafe { String::from_utf8_unchecked(v) }),
                    max: v.max.map(|v| unsafe { String::from_utf8_unchecked(v) }),
                    value_type: DataTypeXSDef::HexBinary,
                },
                Range::AnyURI(v) => Self {
                    min: v.min.map(|v| v.to_string()),
                    max: v.max.map(|v| v.to_string()),
                    value_type: DataTypeXSDef::AnyURI,
                },
            }
        }
    }

    mod tests {
        // TODO
    }
}

#[cfg(all(feature = "json", test))]
mod tests {
    use super::*;

    #[test]
    fn test_range_to_json() {
        let expected = r#"{"valueType":"xs:int","min":1,"max":10}"#;
        let actual = Range::Int(RangeInner {
            min: Some(1),
            max: Some(10),
        });
        let actual: String = serde_json::to_string(&actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_json_to_range() {
        let expected = Range::Int(RangeInner {
            min: Some(1),
            max: Some(10),
        });
        let actual = r#"{"valueType":"xs:int","min":1,"max":10}"#;
        let actual = serde_json::from_str(&actual).unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn test_range_to_metamodel() {
        let expected = r#"{"valueType":"xs:int"}"#;
        let actual = Range::Int(RangeInner {
            min: Some(1),
            max: Some(10),
        });
        let actual = actual.to_json_metamodel().unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn test_range_to_metamodel_camel_case() {
        let expected = r#"{"valueType":"xs:anyUri"}"#;
        let actual = Range::AnyURI(RangeInner {
            min: Some(IriRefBuf::new("https://example.com".into()).unwrap()),
            max: Some(IriRefBuf::new("https://example.com".into()).unwrap()),
        });
        let actual = actual.to_json_metamodel().unwrap();

        assert_eq!(expected, actual);
    }
}
