//! Deserialization — converting generic [`Value`] instances into concrete Rust types.
//!
//! This module defines the [`Deserialize`] trait, which handles safe type conversion
//! and value-level range checks during deserialization from standard and nested formats.

use crate::error::{DeserializationError, Error};
use crate::value::Value;
use std::collections::BTreeMap;

/// Conversion of an unstructured or structured [`Value`] into a typed Rust representation.
///
/// Implementations of this trait are provided for all primitive types, standard options,
/// vectors, and tree structures. Users can derive this trait automatically for custom structs
/// using `#[derive(Schema)]`.
pub trait Deserialize: Sized {
    /// Deserializes `Self` from the provided dynamic [`Value`].
    ///
    /// # Errors
    ///
    /// Returns a [`DeserializationError`] if the data types mismatch, or if numeric constraints
    /// are violated (e.g. range overflow).
    fn deserialize(value: Value) -> Result<Self, Error>;
}

impl Deserialize for bool {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::Bool(b) => Ok(b),
            other => Err(DeserializationError::new(format!(
                "expected bool, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl Deserialize for String {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::String(s) => Ok(s),
            other => Err(DeserializationError::new(format!(
                "expected string, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl Deserialize for char {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::String(s) => {
                let mut chars = s.chars();
                if let (Some(c), None) = (chars.next(), chars.next()) {
                    Ok(c)
                } else {
                    Err(
                        DeserializationError::new("expected a single character string for char")
                            .into(),
                    )
                }
            }
            other => Err(DeserializationError::new(format!(
                "expected string for char, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl Deserialize for f64 {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::Float(f) => Ok(f),
            Value::Int(n) => Ok(n as f64),
            other => Err(DeserializationError::new(format!(
                "expected float, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl Deserialize for f32 {
    fn deserialize(value: Value) -> Result<Self, Error> {
        f64::deserialize(value).map(|f| f as f32)
    }
}

macro_rules! deserialize_int {
    ($($t:ty, $min:expr, $max:expr),*) => {
        $(impl Deserialize for $t {
            fn deserialize(value: Value) -> Result<Self, Error> {
                let n = match &value {
                    Value::Int(n) => *n,
                    Value::Float(f) if f.fract() == 0.0 => *f as i64,
                    other => return Err(DeserializationError::new(
                        format!("expected integer, got {}", other.type_name())
                    ).into()),
                };
                if n < ($min as i64) || n > ($max as i64) {
                    return Err(DeserializationError::new(
                        format!("value {n} out of range for {}", stringify!($t))
                    ).into());
                }
                Ok(n as $t)
            }
        })*
    };
}

deserialize_int!(
    i8,
    i8::MIN,
    i8::MAX,
    i16,
    i16::MIN,
    i16::MAX,
    i32,
    i32::MIN,
    i32::MAX,
    i64,
    i64::MIN,
    i64::MAX
);

macro_rules! deserialize_uint {
    ($($t:ty, $max:expr),*) => {
        $(impl Deserialize for $t {
            fn deserialize(value: Value) -> Result<Self, Error> {
                let n = match &value {
                    Value::Int(n) => *n,
                    Value::Float(f) if f.fract() == 0.0 => *f as i64,
                    other => return Err(DeserializationError::new(
                        format!("expected integer, got {}", other.type_name())
                    ).into()),
                };
                if n < 0 || n > ($max as i64) {
                    return Err(DeserializationError::new(
                        format!("value {n} out of range for {}", stringify!($t))
                    ).into());
                }
                Ok(n as $t)
            }
        })*
    };
}

deserialize_uint!(u8, u8::MAX, u16, u16::MAX, u32, u32::MAX, usize, usize::MAX);

impl Deserialize for u64 {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::Int(n) if n >= 0 => Ok(n as u64),
            Value::Int(n) => {
                Err(DeserializationError::new(format!("value {n} out of range for u64")).into())
            }
            Value::Float(f) if f.fract() == 0.0 && f >= 0.0 => Ok(f as u64),
            other => Err(DeserializationError::new(format!(
                "expected integer, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl Deserialize for i128 {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::Int(n) => Ok(n as i128),
            Value::Float(f) if f.fract() == 0.0 => Ok(f as i128),
            other => Err(DeserializationError::new(format!(
                "expected integer, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl Deserialize for u128 {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::Int(n) if n >= 0 => Ok(n as u128),
            Value::Int(n) => {
                Err(DeserializationError::new(format!("value {n} out of range for u128")).into())
            }
            Value::Float(f) if f.fract() == 0.0 && f >= 0.0 => Ok(f as u128),
            other => Err(DeserializationError::new(format!(
                "expected integer, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl Deserialize for std::net::Ipv4Addr {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::String(s) => s.parse::<std::net::Ipv4Addr>().map_err(|e| {
                DeserializationError::new(format!("invalid IPv4 address: {}", e)).into()
            }),
            other => Err(DeserializationError::new(format!(
                "expected string for IPv4 address, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl Deserialize for std::net::Ipv6Addr {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::String(s) => s.parse::<std::net::Ipv6Addr>().map_err(|e| {
                DeserializationError::new(format!("invalid IPv6 address: {}", e)).into()
            }),
            other => Err(DeserializationError::new(format!(
                "expected string for IPv6 address, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl Deserialize for std::net::IpAddr {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::String(s) => s.parse::<std::net::IpAddr>().map_err(|e| {
                DeserializationError::new(format!("invalid IP address: {}", e)).into()
            }),
            other => Err(DeserializationError::new(format!(
                "expected string for IP address, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl<T: Deserialize> Deserialize for Option<T> {
    fn deserialize(value: Value) -> Result<Self, Error> {
        if value.is_null() {
            Ok(None)
        } else {
            T::deserialize(value).map(Some)
        }
    }
}

impl<T: Deserialize> Deserialize for Vec<T> {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::Array(arr) => arr.into_iter().map(T::deserialize).collect(),
            other => Err(DeserializationError::new(format!(
                "expected array, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl<T: Deserialize> Deserialize for BTreeMap<String, T> {
    fn deserialize(value: Value) -> Result<Self, Error> {
        match value {
            Value::Object(map) => map
                .into_iter()
                .map(|(k, v)| T::deserialize(v).map(|t| (k, t)))
                .collect(),
            other => Err(DeserializationError::new(format!(
                "expected object, got {}",
                other.type_name()
            ))
            .into()),
        }
    }
}

impl Deserialize for Value {
    fn deserialize(value: Value) -> Result<Self, Error> {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool() {
        assert!(bool::deserialize(Value::Bool(true)).unwrap());
    }

    #[test]
    fn test_bool_wrong_type() {
        assert!(bool::deserialize(Value::Int(1)).is_err());
    }

    #[test]
    fn test_i64() {
        assert_eq!(i64::deserialize(Value::Int(42)).unwrap(), 42);
    }

    #[test]
    fn test_u8_in_range() {
        assert_eq!(u8::deserialize(Value::Int(255)).unwrap(), 255u8);
    }

    #[test]
    fn test_u8_out_of_range() {
        assert!(u8::deserialize(Value::Int(256)).is_err());
    }

    #[test]
    fn test_u8_negative_fails() {
        assert!(u8::deserialize(Value::Int(-1)).is_err());
    }

    #[test]
    fn test_i8_out_of_range() {
        assert!(i8::deserialize(Value::Int(200)).is_err());
    }

    #[test]
    fn test_f64() {
        assert_eq!(f64::deserialize(Value::Float(1.23)).unwrap(), 1.23);
    }

    #[test]
    fn test_f64_from_int() {
        assert_eq!(f64::deserialize(Value::Int(5)).unwrap(), 5.0);
    }

    #[test]
    fn test_string() {
        assert_eq!(
            String::deserialize(Value::String("hi".into())).unwrap(),
            "hi"
        );
    }

    #[test]
    fn test_option_none() {
        let v: Option<i32> = Option::deserialize(Value::Null).unwrap();
        assert_eq!(v, None);
    }

    #[test]
    fn test_option_some() {
        let v: Option<i32> = Option::deserialize(Value::Int(7)).unwrap();
        assert_eq!(v, Some(7));
    }

    #[test]
    fn test_vec() {
        let v = Vec::<i32>::deserialize(Value::Array(vec![Value::Int(1), Value::Int(2)])).unwrap();
        assert_eq!(v, vec![1, 2]);
    }

    #[test]
    fn test_btreemap() {
        let mut m = std::collections::BTreeMap::new();
        m.insert("n".to_string(), Value::Int(9));
        let result = BTreeMap::<String, i32>::deserialize(Value::Object(m)).unwrap();
        assert_eq!(result["n"], 9);
    }

    #[test]
    fn test_u64_negative_fails() {
        assert!(u64::deserialize(Value::Int(-1)).is_err());
    }

    #[test]
    fn test_new_types_deserialization() {
        // char tests
        assert_eq!(char::deserialize(Value::String("x".into())).unwrap(), 'x');
        assert!(char::deserialize(Value::String("xy".into())).is_err());
        assert!(char::deserialize(Value::Int(42)).is_err());

        // i128 & u128 tests
        assert_eq!(
            i128::deserialize(Value::Int(123456789)).unwrap(),
            123456789i128
        );
        assert_eq!(
            u128::deserialize(Value::Int(987654321)).unwrap(),
            987654321u128
        );
        assert!(u128::deserialize(Value::Int(-5)).is_err());

        // IpAddress tests
        let ip4 = std::net::Ipv4Addr::new(127, 0, 0, 1);
        let ip6 = std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);

        assert_eq!(
            std::net::Ipv4Addr::deserialize(Value::String("127.0.0.1".into())).unwrap(),
            ip4
        );
        assert!(std::net::Ipv4Addr::deserialize(Value::String("not-an-ip".into())).is_err());

        assert_eq!(
            std::net::Ipv6Addr::deserialize(Value::String("::1".into())).unwrap(),
            ip6
        );
        assert!(std::net::Ipv6Addr::deserialize(Value::String("not-an-ip".into())).is_err());

        assert_eq!(
            std::net::IpAddr::deserialize(Value::String("127.0.0.1".into())).unwrap(),
            std::net::IpAddr::V4(ip4)
        );
        assert_eq!(
            std::net::IpAddr::deserialize(Value::String("::1".into())).unwrap(),
            std::net::IpAddr::V6(ip6)
        );
    }
}
