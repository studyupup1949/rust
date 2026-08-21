use activitystreams_vocabulary::{create_object, field_access, impl_default};
use serde::{de, ser};

use crate::{Error, Result};

create_object! {
    /// Represents a set of named values.
    ///
    /// # Example
    ///
    /// ```rust
    /// use activityforge::{EnumValue, HexColor, context};
    /// use activitystreams_vocabulary::Name;
    ///
    /// # fn main() {
    /// let name = Name::try_from("workflow job #1").unwrap();
    /// let color = HexColor::new()
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
    ///   "type": "EnumValue",
    ///   "name": "{name}",
    ///   "enumValueColor": "{color}"
    /// }}"#
    ///         );
    ///
    /// let context = context::forgefed_context();
    ///
    /// let enum_value = EnumValue::new()
    ///     .with_context_property(context)
    ///     .with_name(name)
    ///     .with_enum_value_color(color);
    ///
    /// assert_eq!(serde_json::to_string_pretty(&enum_value).unwrap(), json_str);
    /// assert_eq!(
    ///     serde_json::from_str::<EnumValue>(json_str.as_str()).unwrap(),
    ///     enum_value
    /// );
    /// # }
    /// ```
    EnumValue: crate::ObjectType::EnumValue {
        #[serde(skip_serializing_if = "Option::is_none")]
        enum_value_color: Option<HexColor>,
    }
}

field_access! {
    EnumValue {
        /// For a given [EnumValue], identifies the color associated with it.
        enum_value_color: option { HexColor },
    }
}

/// Represents a RGB hex string.
///
/// - `#RRGGBB`: three pairs of hex digits representing a color's red, green, and blue components
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HexColor {
    red: u8,
    green: u8,
    blue: u8,
}

impl HexColor {
    /// Represents the string length of the [HexColor], e.g. "#RRGGBB".
    pub const LEN: usize = 7;

    /// Creates a new [HexColor].
    pub const fn new() -> Self {
        Self {
            red: 0,
            green: 0,
            blue: 0,
        }
    }

    /// Parses a hex-digit from the input string.
    ///
    /// Expects the input to be a 2-digit (minimum), hex-encoded string.
    #[inline]
    pub fn parse_hex_digit(val: &str) -> Result<u8> {
        val.get(..2)
            .ok_or(Error::object("hex_color: invalid hex digit length"))
            .and_then(|s| {
                u8::from_str_radix(s, 16)
                    .map_err(|err| Error::object(format!("hex_color: invalid hex digit: {err}")))
            })
    }
}

field_access! {
    HexColor {
        red: u8,
        green: u8,
        blue: u8,
    }
}

impl_default!(HexColor);

impl TryFrom<&str> for HexColor {
    type Error = Error;

    fn try_from(val: &str) -> Result<Self> {
        if val.len() != Self::LEN {
            Err(Error::object(format!(
                "hex_color: invalid length: {}",
                val.len()
            )))
        } else {
            val.strip_prefix('#')
                .ok_or(Error::object("hex_color: missing leading hash"))
                .and_then(|v| {
                    let red = v
                        .get(..2)
                        .ok_or(Error::object("hex_color: missing red component"))
                        .and_then(Self::parse_hex_digit)?;

                    let green = v
                        .get(2..)
                        .ok_or(Error::object("hex_color: missing green component"))
                        .and_then(Self::parse_hex_digit)?;

                    let blue = v
                        .get(4..)
                        .ok_or(Error::object("hex_color: missing blue component"))
                        .and_then(Self::parse_hex_digit)?;

                    Ok(Self { red, green, blue })
                })
        }
    }
}

impl TryFrom<String> for HexColor {
    type Error = Error;

    fn try_from(val: String) -> Result<Self> {
        val.as_str().try_into()
    }
}

impl core::fmt::Display for HexColor {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { red, green, blue } = self;
        write!(f, "#{red:02x}{green:02x}{blue:02x}")
    }
}

impl ser::Serialize for HexColor {
    fn serialize<S>(&self, serializer: S) -> core::result::Result<S::Ok, S::Error>
    where
        S: ser::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> de::Deserialize<'de> for HexColor {
    fn deserialize<D>(deserializer: D) -> core::result::Result<Self, D::Error>
    where
        D: de::Deserializer<'de>,
    {
        <&str>::deserialize(deserializer).and_then(|s| {
            Self::try_from(s).map_err(|err| de::Error::custom(format!("invalid IRI: {err}")))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context;

    use activitystreams_vocabulary::Name;

    #[test]
    fn test_hex_color() {
        (0..=u8::MAX)
            .zip(0..=u8::MAX)
            .zip(0..=u8::MAX)
            .for_each(|((r, g), b)| {
                let hex_color = HexColor::new().with_red(r).with_green(g).with_blue(b);
                let hex_str = format!("#{r:02x}{g:02x}{b:02x}");
                let json_str = format!(r#""{hex_str}""#);

                assert_eq!(hex_color.to_string(), hex_str);
                assert_eq!(HexColor::try_from(hex_str).unwrap(), hex_color);
                assert_eq!(serde_json::to_string(&hex_color).unwrap(), json_str);
                assert_eq!(
                    serde_json::from_str::<HexColor>(&json_str).unwrap(),
                    hex_color
                );
            });
    }

    #[test]
    fn test_hex_color_invalid() {
        // missing leading '#'
        assert!(HexColor::try_from("aabbcc").is_err());
        // invalid hex
        assert!(HexColor::try_from("#zzbbcc").is_err());
        // invalid length (too short)
        assert!(HexColor::try_from("#aabb").is_err());
        // invalid length (too long)
        assert!(HexColor::try_from("#aabbccdd").is_err());
    }

    #[test]
    fn test_enum_value() {
        let name = Name::try_from("workflow job #1").unwrap();
        let color = HexColor::new()
            .with_red(0xaa)
            .with_green(0xbb)
            .with_blue(0xcc);

        let json_str = format!(
            r#"{{
  "@context": [
    "https://www.w3.org/ns/activitystreams",
    "https://forgefed.org/ns"
  ],
  "type": "EnumValue",
  "name": "{name}",
  "enumValueColor": "{color}"
}}"#
        );

        let context = context::forgefed_context();

        let enum_value = EnumValue::new()
            .with_context_property(context)
            .with_name(name)
            .with_enum_value_color(color);

        assert_eq!(serde_json::to_string_pretty(&enum_value).unwrap(), json_str);
        assert_eq!(
            serde_json::from_str::<EnumValue>(json_str.as_str()).unwrap(),
            enum_value
        );
    }
}
