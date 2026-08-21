use activitystreams_vocabulary::{impl_default, impl_display};
use serde::{Deserialize, Serialize};

/// Represents the type of a [Field](crate::Field).
#[derive(Clone, Copy, Debug, Eq, PartialEq, Deserialize, Serialize)]
pub enum FieldType {
    /// Represents a text field, e.g. "some text"
    #[serde(rename = "fieldTypeText")]
    Text,
    /// Represents an integer field, e.g. `42`
    #[serde(rename = "fieldTypeInteger")]
    Integer,
    /// Represents a rational field, e.g. `4.2`
    #[serde(rename = "fieldTypeRational")]
    Rational,
    /// Represents a boolean field, e.g. `false`
    #[serde(rename = "fieldTypeBoolean")]
    Boolean,
    /// No value.
    ///
    /// The presence or absence of the field itself is the information.
    ///
    /// Can be used for representing simple issue labels.
    #[serde(rename = "fieldTypeClass")]
    Class,
    /// A certain specific [Enum](crate::Enum).
    #[serde(rename = "fieldTypeEnum")]
    Enum,
}

impl FieldType {
    /// The string representation of the [Text](Self::Text) variant.
    pub const TEXT: &str = "fieldTypeText";
    /// The string representation of the [Integer](Self::Integer) variant.
    pub const INTEGER: &str = "fieldTypeInteger";
    /// The string representation of the [Rational](Self::Rational) variant.
    pub const RATIONAL: &str = "fieldTypeRational";
    /// The string representation of the [Boolean](Self::Boolean) variant.
    pub const BOOLEAN: &str = "fieldTypeBoolean";
    /// The string representation of the [Class](Self::Class) variant.
    pub const CLASS: &str = "fieldTypeClass";
    /// The string representation of the [Enum](Self::Enum) variant.
    pub const ENUM: &str = "fieldTypeEnum";

    /// Creates a new [FieldType].
    pub const fn new() -> Self {
        Self::Text
    }

    /// Gets the string representation of the [FieldType].
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Text => Self::TEXT,
            Self::Integer => Self::INTEGER,
            Self::Rational => Self::RATIONAL,
            Self::Boolean => Self::BOOLEAN,
            Self::Class => Self::CLASS,
            Self::Enum => Self::ENUM,
        }
    }
}

impl_default!(FieldType);
impl_display!(FieldType, str);

impl From<FieldType> for &'static str {
    fn from(val: FieldType) -> Self {
        val.as_str()
    }
}

impl From<&FieldType> for &'static str {
    fn from(val: &FieldType) -> Self {
        val.as_str()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_type() {
        [
            (FieldType::Text, FieldType::TEXT),
            (FieldType::Integer, FieldType::INTEGER),
            (FieldType::Rational, FieldType::RATIONAL),
            (FieldType::Boolean, FieldType::BOOLEAN),
            (FieldType::Class, FieldType::CLASS),
            (FieldType::Enum, FieldType::ENUM),
        ]
        .into_iter()
        .for_each(|(ty, ty_str)| {
            let json_str = format!(r#""{ty_str}""#);

            assert_eq!(ty.to_string(), ty_str);
            assert_eq!(serde_json::to_string(&ty).unwrap(), json_str);
            assert_eq!(serde_json::from_str::<FieldType>(&json_str).unwrap(), ty);
        });
    }
}
