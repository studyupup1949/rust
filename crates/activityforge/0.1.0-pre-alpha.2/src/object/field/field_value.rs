use activitystreams_vocabulary::{create_object, field_access, impl_default, impl_display};
use serde::{Deserialize, Serialize};

use crate::{Error, Result};

create_object! {
    /// Specifies a value of a [Field](crate::Field).
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{FieldValue, context};
    ///
    /// # fn main() {
    /// let value = "some field value";
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "FieldValue",
    ///   "fieldValue": "{value}"
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
    ///
    /// let field_value = FieldValue::new()
    ///     .with_context_property(context)
    ///     .with_field_value(value.to_string());
    ///
    /// assert_eq!(serde_json::to_string_pretty(&field_value).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<FieldValue>(json_str.as_str()).unwrap(),
    ///     field_value
    /// );
    /// # }
    /// ```
    FieldValue: crate::ObjectType::FieldValue {
        field_value: Option<serde_json::Value>,
    }
}

field_access! {
    FieldValue {
        /// Specifies the value of a custom field.
        field_value: option_ref { serde_json::Value },
    }
}

/// Represents the different variants for `fieldValue` properties.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum FieldValueItem {
    Object(Box<FieldValue>),
    Flat(Box<serde_json::Value>),
}

impl FieldValueItem {
    /// Creates a new [FieldValueItem].
    pub fn new() -> Self {
        Self::Object(Box::default())
    }

    /// Creates a new [Object](Self::Object) variant.
    pub fn object<I: Into<FieldValue>>(val: I) -> Self {
        Self::Object(Box::new(val.into()))
    }

    /// Gets whether the [FieldValueItem] is a [Object](Self::Object) variant.
    pub const fn is_object(&self) -> bool {
        matches!(self, Self::Object(_))
    }

    /// Gets reference to the [Object](Self::Object) variant.
    pub fn as_object(&self) -> Result<&FieldValue> {
        match self {
            Self::Object(val) => Ok(val),
            _ => Err(Error::object("field: invalid field value variant")),
        }
    }

    /// Converts into an [Object](Self::Object) variant.
    pub fn to_object(self) -> Result<FieldValue> {
        match self {
            Self::Object(val) => Ok(*val),
            _ => Err(Error::object("field: invalid field value variant")),
        }
    }

    /// Creates a new [Flat](Self::Flat) variant.
    pub fn flat<I: Into<serde_json::Value>>(val: I) -> Self {
        Self::Flat(Box::new(val.into()))
    }

    /// Gets whether the [FieldValueItem] is a [Flat](Self::Flat) variant.
    pub const fn is_flat(&self) -> bool {
        matches!(self, Self::Flat(_))
    }

    /// Gets reference to the [Flat](Self::Flat) variant.
    pub fn as_flat(&self) -> Result<&serde_json::Value> {
        match self {
            Self::Flat(val) => Ok(val),
            _ => Err(Error::object("field: invalid field value variant")),
        }
    }

    /// Converts into an [Flat](Self::Flat) variant.
    pub fn to_flat(self) -> Result<serde_json::Value> {
        match self {
            Self::Flat(val) => Ok(*val),
            _ => Err(Error::object("field: invalid field value variant")),
        }
    }
}

impl From<FieldValue> for FieldValueItem {
    fn from(val: FieldValue) -> Self {
        Self::object(val)
    }
}

impl From<serde_json::Value> for FieldValueItem {
    fn from(val: serde_json::Value) -> Self {
        Self::flat(val)
    }
}

impl TryFrom<FieldValueItem> for FieldValue {
    type Error = Error;

    fn try_from(val: FieldValueItem) -> Result<Self> {
        val.to_object()
    }
}

impl<'a> TryFrom<&'a FieldValueItem> for &'a FieldValue {
    type Error = Error;

    fn try_from(val: &'a FieldValueItem) -> Result<Self> {
        val.as_object()
    }
}

impl TryFrom<FieldValueItem> for serde_json::Value {
    type Error = Error;

    fn try_from(val: FieldValueItem) -> Result<Self> {
        val.to_flat()
    }
}

impl<'a> TryFrom<&'a FieldValueItem> for &'a serde_json::Value {
    type Error = Error;

    fn try_from(val: &'a FieldValueItem) -> Result<Self> {
        val.as_flat()
    }
}

impl_default!(FieldValueItem);
impl_display!(FieldValueItem, json);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    #[test]
    fn test_field_value() {
        let value = "some field value";

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "FieldValue",
  "fieldValue": "{value}"
}}"#
        );

        let context = context::forgefed_context();

        let field_value = FieldValue::new()
            .with_context_property(context)
            .with_field_value(value.to_string());

        assert_eq!(
            serde_json::to_string_pretty(&field_value).unwrap(),
            json_str
        );
        assert_eq!(
            serde_json::from_str::<FieldValue>(json_str.as_str()).unwrap(),
            field_value
        );
    }
}
