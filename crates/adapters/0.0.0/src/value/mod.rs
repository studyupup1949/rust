//! Universal interchange `Value` type used across every module.
//!
//! This module defines the [`Value`] enum representing generic JSON-like structures,
//! along with type-safe extractors, type predicates, displaying, and conversion traits.

use std::collections::BTreeMap;
use std::fmt;

/// The dynamic structure representing any valid data payload.
///
/// Serves as the central intermediate representation between parsed JSON, custom structures,
/// and the validation/transformation layers of the library.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    /// JSON `null` value.
    Null,
    /// Boolean values (true/false).
    Bool(bool),
    /// Un-fractioned 64-bit integer values.
    Int(i64),
    /// Floating-point numeric representation.
    Float(f64),
    /// UTF-8 string values.
    String(String),
    /// Sequential arrays of values.
    Array(Vec<Value>),
    /// Key-value maps with ordered string keys.
    Object(BTreeMap<std::string::String, Value>),
}

impl Value {
    /// Returns `true` if the value is `Null`.
    pub fn is_null(&self) -> bool {
        matches!(self, Value::Null)
    }
    /// Returns `true` if the value is a boolean.
    pub fn is_bool(&self) -> bool {
        matches!(self, Value::Bool(_))
    }
    /// Returns `true` if the value is an integer.
    pub fn is_int(&self) -> bool {
        matches!(self, Value::Int(_))
    }
    /// Returns `true` if the value is a float.
    pub fn is_float(&self) -> bool {
        matches!(self, Value::Float(_))
    }
    /// Returns `true` if the value is numeric (integer or float).
    pub fn is_number(&self) -> bool {
        self.is_int() || self.is_float()
    }
    /// Returns `true` if the value is a string.
    pub fn is_string(&self) -> bool {
        matches!(self, Value::String(_))
    }
    /// Returns `true` if the value is an array.
    pub fn is_array(&self) -> bool {
        matches!(self, Value::Array(_))
    }
    /// Returns `true` if the value is an object.
    pub fn is_object(&self) -> bool {
        matches!(self, Value::Object(_))
    }

    /// Extracts the inner boolean value if possible.
    pub fn as_bool(&self) -> Option<bool> {
        if let Value::Bool(b) = self {
            Some(*b)
        } else {
            None
        }
    }

    /// Extracts the inner 64-bit integer value if possible.
    pub fn as_int(&self) -> Option<i64> {
        if let Value::Int(n) = self {
            Some(*n)
        } else {
            None
        }
    }

    /// Extracts the inner float value if possible.
    pub fn as_float(&self) -> Option<f64> {
        if let Value::Float(f) = self {
            Some(*f)
        } else {
            None
        }
    }

    /// Extracts a slice reference to the inner string if possible.
    pub fn as_str(&self) -> Option<&str> {
        if let Value::String(s) = self {
            Some(s.as_str())
        } else {
            None
        }
    }

    /// Extracts a reference to the inner array if possible.
    pub fn as_array(&self) -> Option<&Vec<Value>> {
        if let Value::Array(a) = self {
            Some(a)
        } else {
            None
        }
    }

    /// Extracts a reference to the inner object map if possible.
    pub fn as_object(&self) -> Option<&BTreeMap<std::string::String, Value>> {
        if let Value::Object(o) = self {
            Some(o)
        } else {
            None
        }
    }

    /// Extracts a mutable reference to the inner array if possible.
    pub fn as_array_mut(&mut self) -> Option<&mut Vec<Value>> {
        if let Value::Array(a) = self {
            Some(a)
        } else {
            None
        }
    }

    /// Extracts a mutable reference to the inner object map if possible.
    pub fn as_object_mut(&mut self) -> Option<&mut BTreeMap<std::string::String, Value>> {
        if let Value::Object(o) = self {
            Some(o)
        } else {
            None
        }
    }

    /// Retrieves an immutable field reference by key if the value is an object.
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.as_object()?.get(key)
    }

    /// Retrieves a mutable field reference by key if the value is an object.
    pub fn get_mut(&mut self, key: &str) -> Option<&mut Value> {
        self.as_object_mut()?.get_mut(key)
    }

    /// Inserts a new field entry if the value is an object.
    pub fn insert(&mut self, key: std::string::String, value: Value) {
        if let Value::Object(map) = self {
            map.insert(key, value);
        }
    }

    /// Accesses an array item by offset index if the value is an array.
    pub fn index(&self, i: usize) -> Option<&Value> {
        self.as_array()?.get(i)
    }

    /// Returns a static name descriptor string representing the structural type.
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Null => "null",
            Value::Bool(_) => "bool",
            Value::Int(_) => "int",
            Value::Float(_) => "float",
            Value::String(_) => "string",
            Value::Array(_) => "array",
            Value::Object(_) => "object",
        }
    }

    /// Coerces the current value to an integer if losslessly convertible.
    pub fn coerce_to_int(&self) -> Option<i64> {
        match self {
            Value::Int(n) => Some(*n),
            Value::Float(f) => {
                if f.fract() == 0.0 && *f >= i64::MIN as f64 && *f <= i64::MAX as f64 {
                    Some(*f as i64)
                } else {
                    None
                }
            }
            Value::String(s) => s.trim().parse::<i64>().ok(),
            Value::Bool(b) => Some(if *b { 1 } else { 0 }),
            _ => None,
        }
    }

    /// Coerces the current value to a floating point number.
    pub fn coerce_to_float(&self) -> Option<f64> {
        match self {
            Value::Float(f) => Some(*f),
            Value::Int(n) => Some(*n as f64),
            Value::String(s) => s.trim().parse::<f64>().ok(),
            Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
            _ => None,
        }
    }

    /// Coerces the current value to a string.
    pub fn coerce_to_string(&self) -> Option<std::string::String> {
        match self {
            Value::String(s) => Some(s.clone()),
            Value::Int(n) => Some(n.to_string()),
            Value::Float(f) => Some(f.to_string()),
            Value::Bool(b) => Some(b.to_string()),
            Value::Null => Some("null".to_string()),
            _ => None,
        }
    }

    /// Coerces the current value to a boolean.
    pub fn coerce_to_bool(&self) -> Option<bool> {
        match self {
            Value::Bool(b) => Some(*b),
            Value::Int(n) => Some(*n != 0),
            Value::String(s) => match s.trim().to_lowercase().as_str() {
                "true" | "1" | "yes" => Some(true),
                "false" | "0" | "no" => Some(false),
                _ => None,
            },
            _ => None,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Null => write!(f, "null"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Int(n) => write!(f, "{n}"),
            Value::Float(fv) => {
                if fv.is_finite() {
                    let s = format!("{fv}");
                    if s.contains('.') || s.contains('e') || s.contains('E') {
                        write!(f, "{s}")
                    } else {
                        write!(f, "{s}.0")
                    }
                } else {
                    write!(f, "null")
                }
            }
            Value::String(s) => {
                write!(f, "\"")?;
                for ch in s.chars() {
                    match ch {
                        '"' => write!(f, "\\\"")?,
                        '\\' => write!(f, "\\\\")?,
                        '\n' => write!(f, "\\n")?,
                        '\r' => write!(f, "\\r")?,
                        '\t' => write!(f, "\\t")?,
                        c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
                        c => write!(f, "{c}")?,
                    }
                }
                write!(f, "\"")
            }
            Value::Array(arr) => {
                write!(f, "[")?;
                for (i, v) in arr.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, "]")
            }
            Value::Object(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ",")?;
                    }
                    write!(f, "\"")?;
                    for ch in k.chars() {
                        match ch {
                            '"' => write!(f, "\\\"")?,
                            '\\' => write!(f, "\\\\")?,
                            '\n' => write!(f, "\\n")?,
                            '\r' => write!(f, "\\r")?,
                            '\t' => write!(f, "\\t")?,
                            c if (c as u32) < 0x20 => write!(f, "\\u{:04x}", c as u32)?,
                            c => write!(f, "{c}")?,
                        }
                    }
                    write!(f, "\":")?;
                    write!(f, "{v}")?;
                }
                write!(f, "}}")
            }
        }
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

macro_rules! from_int {
    ($($t:ty),*) => {
        $(impl From<$t> for Value {
            fn from(n: $t) -> Self { Value::Int(n as i64) }
        })*
    };
}
from_int!(i8, i16, i32, i64, u8, u16, u32, u64, usize);

impl From<f32> for Value {
    fn from(f: f32) -> Self {
        Value::Float(f as f64)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Value::Float(f)
    }
}

impl From<std::string::String> for Value {
    fn from(s: std::string::String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}

impl From<BTreeMap<std::string::String, Value>> for Value {
    fn from(m: BTreeMap<std::string::String, Value>) -> Self {
        Value::Object(m)
    }
}

impl From<Option<Value>> for Value {
    fn from(opt: Option<Value>) -> Self {
        opt.unwrap_or(Value::Null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_predicates() {
        assert!(Value::Null.is_null());
        assert!(Value::Bool(true).is_bool());
        assert!(Value::Int(1).is_int());
        assert!(Value::Float(1.5).is_float());
        assert!(Value::Int(1).is_number());
        assert!(Value::Float(1.0).is_number());
        assert!(Value::String("x".into()).is_string());
        assert!(Value::Array(vec![]).is_array());
        assert!(Value::Object(BTreeMap::new()).is_object());
    }

    #[test]
    fn test_extractors() {
        assert_eq!(Value::Bool(true).as_bool(), Some(true));
        assert_eq!(Value::Int(42).as_int(), Some(42));
        assert_eq!(Value::Float(1.23).as_float(), Some(1.23));
        assert_eq!(Value::String("hi".into()).as_str(), Some("hi"));
        assert!(Value::Array(vec![]).as_array().is_some());
        assert!(Value::Object(BTreeMap::new()).as_object().is_some());
    }

    #[test]
    fn test_object_get_insert() {
        let mut obj = Value::Object(BTreeMap::new());
        obj.insert("key".to_string(), Value::Int(99));
        assert_eq!(obj.get("key"), Some(&Value::Int(99)));
        assert_eq!(obj.get("missing"), None);
    }

    #[test]
    fn test_array_index() {
        let arr = Value::Array(vec![Value::Int(1), Value::Int(2)]);
        assert_eq!(arr.index(0), Some(&Value::Int(1)));
        assert_eq!(arr.index(5), None);
    }

    #[test]
    fn test_type_name() {
        assert_eq!(Value::Null.type_name(), "null");
        assert_eq!(Value::Bool(false).type_name(), "bool");
        assert_eq!(Value::Int(0).type_name(), "int");
        assert_eq!(Value::Float(0.0).type_name(), "float");
        assert_eq!(Value::String(String::new()).type_name(), "string");
        assert_eq!(Value::Array(vec![]).type_name(), "array");
        assert_eq!(Value::Object(BTreeMap::new()).type_name(), "object");
    }

    #[test]
    fn test_coerce_to_int() {
        assert_eq!(Value::Int(7).coerce_to_int(), Some(7));
        assert_eq!(Value::Float(3.0).coerce_to_int(), Some(3));
        assert_eq!(Value::Float(3.5).coerce_to_int(), None);
        assert_eq!(Value::String("42".into()).coerce_to_int(), Some(42));
        assert_eq!(Value::Bool(true).coerce_to_int(), Some(1));
    }

    #[test]
    fn test_coerce_to_float() {
        assert_eq!(Value::Float(1.5).coerce_to_float(), Some(1.5));
        assert_eq!(Value::Int(3).coerce_to_float(), Some(3.0));
        assert_eq!(Value::String("2.5".into()).coerce_to_float(), Some(2.5));
    }

    #[test]
    fn test_coerce_to_string() {
        assert_eq!(
            Value::String("hi".into()).coerce_to_string(),
            Some("hi".into())
        );
        assert_eq!(Value::Int(7).coerce_to_string(), Some("7".into()));
        assert_eq!(Value::Bool(false).coerce_to_string(), Some("false".into()));
    }

    #[test]
    fn test_coerce_to_bool() {
        assert_eq!(Value::Bool(true).coerce_to_bool(), Some(true));
        assert_eq!(Value::Int(0).coerce_to_bool(), Some(false));
        assert_eq!(Value::String("true".into()).coerce_to_bool(), Some(true));
        assert_eq!(Value::String("false".into()).coerce_to_bool(), Some(false));
        assert_eq!(Value::String("garbage".into()).coerce_to_bool(), None);
    }

    #[test]
    fn test_display_null() {
        assert_eq!(Value::Null.to_string(), "null");
    }

    #[test]
    fn test_display_bool() {
        assert_eq!(Value::Bool(true).to_string(), "true");
        assert_eq!(Value::Bool(false).to_string(), "false");
    }

    #[test]
    fn test_display_int() {
        assert_eq!(Value::Int(-42).to_string(), "-42");
    }

    #[test]
    fn test_display_string_escaping() {
        let v = Value::String("say \"hello\"\n".into());
        let s = v.to_string();
        assert!(s.contains("\\\""));
        assert!(s.contains("\\n"));
    }

    #[test]
    fn test_display_array() {
        let v = Value::Array(vec![Value::Int(1), Value::Bool(true)]);
        assert_eq!(v.to_string(), "[1,true]");
    }

    #[test]
    fn test_display_object() {
        let mut m = BTreeMap::new();
        m.insert("a".to_string(), Value::Int(1));
        let v = Value::Object(m);
        assert_eq!(v.to_string(), r#"{"a":1}"#);
    }

    #[test]
    fn test_from_impls() {
        let _: Value = true.into();
        let _: Value = 42i64.into();
        let _: Value = 1.23f64.into();
        let _: Value = "hello".into();
        let _: Value = "world".to_string().into();
        let _: Value = vec![Value::Null].into();
        let v: Value = (None as Option<Value>).into();
        assert!(v.is_null());
        let v: Value = Some(Value::Int(1)).into();
        assert_eq!(v, Value::Int(1));
    }

    #[test]
    fn test_get_nested() {
        let mut inner = BTreeMap::new();
        inner.insert("city".to_string(), Value::String("Berlin".into()));
        let mut outer = BTreeMap::new();
        outer.insert("address".to_string(), Value::Object(inner));
        let root = Value::Object(outer);
        let city = root.get("address").and_then(|a| a.get("city"));
        assert_eq!(city, Some(&Value::String("Berlin".into())));
    }
}
