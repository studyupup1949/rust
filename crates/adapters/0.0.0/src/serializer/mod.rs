//! Serialization — converting Rust structures and primitive types into generic [`Value`] instances.
//!
//! This module defines the [`Serialize`] trait, which provides structural mapping
//! from typed domain models to dynamic unstructured formats.

use crate::value::Value;
use std::collections::BTreeMap;

/// Conversion of a concrete Rust type into a generic [`Value`] representation.
///
/// Standard implementations are provided for all numeric primitives, strings, options,
/// vectors, and maps. Users can derive this trait automatically using `#[derive(Schema)]`.
pub trait Serialize {
    /// Maps `self` into its corresponding dynamic [`Value`] tree structure.
    fn serialize(&self) -> Value;
}

impl Serialize for bool {
    fn serialize(&self) -> Value {
        Value::Bool(*self)
    }
}

impl Serialize for String {
    fn serialize(&self) -> Value {
        Value::String(self.clone())
    }
}

impl Serialize for &str {
    fn serialize(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl Serialize for char {
    fn serialize(&self) -> Value {
        Value::String(self.to_string())
    }
}

macro_rules! serialize_int {
    ($($t:ty),*) => {
        $(impl Serialize for $t {
            fn serialize(&self) -> Value { Value::Int(*self as i64) }
        })*
    };
}
serialize_int!(i8, i16, i32, i64, i128, u8, u16, u32, u64, u128, usize);

impl Serialize for std::net::Ipv4Addr {
    fn serialize(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl Serialize for std::net::Ipv6Addr {
    fn serialize(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl Serialize for std::net::IpAddr {
    fn serialize(&self) -> Value {
        Value::String(self.to_string())
    }
}

impl Serialize for f32 {
    fn serialize(&self) -> Value {
        Value::Float(*self as f64)
    }
}

impl Serialize for f64 {
    fn serialize(&self) -> Value {
        Value::Float(*self)
    }
}

impl<T: Serialize> Serialize for Option<T> {
    fn serialize(&self) -> Value {
        match self {
            Some(v) => v.serialize(),
            None => Value::Null,
        }
    }
}

impl<T: Serialize> Serialize for Vec<T> {
    fn serialize(&self) -> Value {
        Value::Array(self.iter().map(|v| v.serialize()).collect())
    }
}

impl<T: Serialize> Serialize for BTreeMap<String, T> {
    fn serialize(&self) -> Value {
        Value::Object(
            self.iter()
                .map(|(k, v)| (k.clone(), v.serialize()))
                .collect(),
        )
    }
}

impl Serialize for Value {
    fn serialize(&self) -> Value {
        self.clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bool() {
        assert_eq!(true.serialize(), Value::Bool(true));
        assert_eq!(false.serialize(), Value::Bool(false));
    }

    #[test]
    fn test_integers() {
        assert_eq!(42i32.serialize(), Value::Int(42));
        assert_eq!(255u8.serialize(), Value::Int(255));
        assert_eq!((-1i64).serialize(), Value::Int(-1));
    }

    #[test]
    fn test_float() {
        assert_eq!(1.23f64.serialize(), Value::Float(1.23));
    }

    #[test]
    fn test_string() {
        assert_eq!("hello".serialize(), Value::String("hello".into()));
        assert_eq!(
            "world".to_string().serialize(),
            Value::String("world".into())
        );
    }

    #[test]
    fn test_option_some() {
        let v: Option<i32> = Some(7);
        assert_eq!(v.serialize(), Value::Int(7));
    }

    #[test]
    fn test_option_none() {
        let v: Option<i32> = None;
        assert_eq!(v.serialize(), Value::Null);
    }

    #[test]
    fn test_vec() {
        let v = vec![1i32, 2, 3];
        assert_eq!(
            v.serialize(),
            Value::Array(vec![Value::Int(1), Value::Int(2), Value::Int(3),])
        );
    }

    #[test]
    fn test_btreemap() {
        let mut m = BTreeMap::new();
        m.insert("x".to_string(), 10i32);
        let v = m.serialize();
        assert_eq!(v.get("x"), Some(&Value::Int(10)));
    }

    #[test]
    fn test_new_types_serialization() {
        assert_eq!('a'.serialize(), Value::String("a".into()));
        assert_eq!(12345i128.serialize(), Value::Int(12345));
        assert_eq!(67890u128.serialize(), Value::Int(67890));

        let ip4 = std::net::Ipv4Addr::new(127, 0, 0, 1);
        assert_eq!(ip4.serialize(), Value::String("127.0.0.1".into()));

        let ip6 = std::net::Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
        assert_eq!(ip6.serialize(), Value::String("::1".into()));
    }
}
