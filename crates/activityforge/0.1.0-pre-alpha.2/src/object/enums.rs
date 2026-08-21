use activitystreams_vocabulary::{OrderedCollection, create_object, field_access};

mod value;

pub use value::{EnumValue, HexColor};

create_object! {
    /// Represents a set of named values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{Enum, EnumValue, HexColor, context};
    /// use activitystreams_vocabulary::{Name, OrderedCollection};
    ///
    /// # fn main() {
    /// let value_name = Name::try_from("workflow job #1").unwrap();
    /// let value_color = HexColor::new()
    ///     .with_red(0xaa)
    ///     .with_green(0xbb)
    ///     .with_blue(0xcc);
    ///
    /// let json_str = format!(
    /// r#"{{
    ///   "@context": [
    ///     "https://www.w3.org/ns/activitystreams",
    ///     "https://forgefed.org/ns"
    ///   ],
    ///   "type": "Enum",
    ///   "enumIsOrdered": true,
    ///   "enumValues": {{
    ///     "type": "OrderedCollection",
    ///     "totalItems": 1,
    ///     "orderedItems": [
    ///       {{
    ///         "type": "EnumValue",
    ///         "name": "{value_name}",
    ///         "enumValueColor": "{value_color}"
    ///       }}
    ///     ]
    ///   }}
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
    ///
    /// let enum_value = EnumValue::new_inner()
    ///     .with_name(value_name)
    ///     .with_enum_value_color(value_color);
    ///
    /// let enum_values = OrderedCollection::new_inner()
    ///     .with_total_items(1u64)
    ///     .with_ordered_items([enum_value]);
    ///
    /// let enums = Enum::new()
    ///     .with_context_property(context)
    ///     .with_enum_is_ordered(true)
    ///     .with_enum_values(enum_values);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&enums).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<Enum>(json_str.as_str()).unwrap(),
    ///     enums
    /// );
    /// # }
    /// ```
    Enum: crate::ObjectType::Enum {
        #[serde(skip_serializing_if = "Option::is_none")]
        enum_is_ordered: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        enum_values: Option<OrderedCollection>,
    }
}

field_access! {
    Enum {
        /// For a given [Enum], indicates whether its values have an ordering relation, or they’re an unordered set.
        ///
        /// **NOTE**: this should always be true, or assumed true if absent.
        enum_is_ordered: option { bool },
    }
}

field_access! {
    Enum {
        /// For a given [Enum], identifies the list of possible values it has.
        enum_values: option_ref { OrderedCollection },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::Name;

    #[test]
    fn test_enum() {
        let value_name = Name::try_from("workflow job #1").unwrap();
        let value_color = HexColor::new()
            .with_red(0xaa)
            .with_green(0xbb)
            .with_blue(0xcc);

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "Enum",
  "enumIsOrdered": true,
  "enumValues": {{
    "type": "OrderedCollection",
    "totalItems": 1,
    "orderedItems": [
      {{
        "type": "EnumValue",
        "name": "{value_name}",
        "enumValueColor": "{value_color}"
      }}
    ]
  }}
}}"#
        );

        let context = context::forgefed_context();

        let enum_value = EnumValue::new_inner()
            .with_name(value_name)
            .with_enum_value_color(value_color);

        let enum_values = OrderedCollection::new_inner()
            .with_total_items(1u64)
            .with_ordered_items([enum_value]);

        let enums = Enum::new()
            .with_context_property(context)
            .with_enum_is_ordered(true)
            .with_enum_values(enum_values);

        assert_eq!(serde_json::to_string_pretty(&enums).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<Enum>(json_str.as_str()).unwrap(),
            enums
        );
    }
}
