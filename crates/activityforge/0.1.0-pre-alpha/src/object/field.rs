use activitystreams_vocabulary::{create_object, field_access};

use crate::HexColor;

mod field_type;
mod field_value;

pub use field_type::FieldType;
pub use field_value::{FieldValue, FieldValueItem};

create_object! {
    /// Represents a custom ticket field within a [Workflow](crate::Workflow).
    ///
    /// # Example (with flat `FieldValue`)
    ///
    /// ```rust
    /// use activityforge::{Field, FieldType, HexColor, context};
    /// use activitystreams_vocabulary::Name;
    ///
    /// # fn main() {
    /// let name = Name::try_from("workflow custom field").unwrap();
    /// let field_color = HexColor::new()
    ///     .with_red(0xaa)
    ///     .with_green(0xbb)
    ///     .with_blue(0xcc);
    /// let field_type = FieldType::Text;
    /// let value = "some field value";
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Field",
    ///   "name": "{name}",
    ///   "fieldColor": "{field_color}",
    ///   "fieldType": "{field_type}",
    ///   "fieldValue": "{value}"
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
    ///
    /// let field_value: serde_json::Value = value.to_string().into();
    ///
    /// let field = Field::new()
    ///     .with_context_property(context)
    ///     .with_name(name)
    ///     .with_field_color(field_color)
    ///     .with_field_type(field_type)
    ///     .with_field_value(field_value);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&field).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Field>(json_str.as_str()).unwrap(),
    ///     field
    /// );
    /// # }
    /// ```
    ///
    /// # Example (with object `FieldValue`)
    ///
    /// ```rust
    /// use activityforge::{Field, FieldType, FieldValue, HexColor, context};
    /// use activitystreams_vocabulary::Name;
    ///
    /// # fn main() {
    /// let name = Name::try_from("workflow custom field").unwrap();
    /// let field_color = HexColor::new()
    ///     .with_red(0xaa)
    ///     .with_green(0xbb)
    ///     .with_blue(0xcc);
    /// let field_type = FieldType::Text;
    /// let value = "some field value";
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Field",
    ///   "name": "{name}",
    ///   "fieldColor": "{field_color}",
    ///   "fieldType": "{field_type}",
    ///   "fieldValue": {{
    ///     "type": "FieldValue",
    ///     "fieldValue": "{value}"
    ///   }}
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
    ///
    /// let field_value = FieldValue::new_inner()
    ///         .with_field_value(value.to_string());
    ///
    /// let field = Field::new()
    ///     .with_context_property(context)
    ///     .with_name(name)
    ///     .with_field_color(field_color)
    ///     .with_field_type(field_type)
    ///     .with_field_value(field_value);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&field).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Field>(json_str.as_str()).unwrap(),
    ///     field
    /// );
    /// # }
    /// ```
    Field: crate::ObjectType::Field {
        field_color: Option<HexColor>,
        field_type: Option<FieldType>,
        field_value: Option<FieldValueItem>,
    }
}

field_access! {
    Field {
        field_color: option { HexColor },
        field_type: option { FieldType },
    }
}

field_access! {
    Field {
        field_value: option_ref { FieldValueItem },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::Name;

    #[test]
    fn test_field() {
        let name = Name::try_from("workflow custom field").unwrap();
        let field_color = HexColor::new()
            .with_red(0xaa)
            .with_green(0xbb)
            .with_blue(0xcc);
        let field_type = FieldType::Text;
        let value = "some field value";

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Field",
  "name": "{name}",
  "fieldColor": "{field_color}",
  "fieldType": "{field_type}",
  "fieldValue": "{value}"
}}"#
        );

        let context = context::forgefed_context();

        let field_value: serde_json::Value = value.to_string().into();

        let field = Field::new()
            .with_context_property(context)
            .with_name(name)
            .with_field_color(field_color)
            .with_field_type(field_type)
            .with_field_value(field_value);

        assert_eq!(serde_json::to_string_pretty(&field).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Field>(json_str.as_str()).unwrap(),
            field
        );
    }
}
