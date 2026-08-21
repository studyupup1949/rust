//! Canonical JSON encoding for Accumulate protocol
//!
//! Provides deterministic JSON serialization with alphabetically ordered keys
//! to ensure byte-for-byte compatibility with TypeScript SDK

// Allow unwrap in this module - serialization of valid JSON values cannot fail
#![allow(clippy::unwrap_used)]

use serde_json::{Map, Value};
use std::collections::BTreeMap;

/// Canonical JSON encoder for Accumulate protocol
#[derive(Debug, Clone, Copy)]
pub struct CanonicalEncoder;

impl CanonicalEncoder {
    /// Encode a JSON value to canonical string format
    pub fn encode(value: &Value) -> String {
        Self::encode_value(value)
    }

    fn encode_value(value: &Value) -> String {
        match value {
            Value::Null => "null".to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Number(n) => n.to_string(),
            Value::String(s) => serde_json::to_string(s).unwrap(),
            Value::Array(arr) => Self::encode_array(arr),
            Value::Object(obj) => Self::encode_object(obj),
        }
    }

    fn encode_array(arr: &[Value]) -> String {
        let elements: Vec<String> = arr.iter().map(Self::encode_value).collect();
        format!("[{}]", elements.join(","))
    }

    fn encode_object(obj: &Map<String, Value>) -> String {
        // Sort keys alphabetically for deterministic output
        let mut sorted_keys: Vec<&String> = obj.keys().collect();
        sorted_keys.sort();

        let pairs: Vec<String> = sorted_keys
            .iter()
            .map(|key| {
                let key_json = serde_json::to_string(key).unwrap();
                let value_json = Self::encode_value(obj.get(*key).unwrap());
                format!("{}:{}", key_json, value_json)
            })
            .collect();

        format!("{{{}}}", pairs.join(","))
    }

    /// Pre-process a JSON value to ensure all objects have sorted keys
    pub fn canonicalize(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let mut btree: BTreeMap<String, Value> = BTreeMap::new();
                for (k, v) in map {
                    btree.insert(k.clone(), Self::canonicalize(v));
                }
                Value::Object(Map::from_iter(btree.into_iter()))
            }
            Value::Array(arr) => Value::Array(arr.iter().map(Self::canonicalize).collect()),
            _ => value.clone(),
        }
    }
}

/// Convenience function for canonical JSON encoding
pub fn to_canonical_string(value: &Value) -> String {
    CanonicalEncoder::encode(value)
}

/// Convenience function for canonical JSON preprocessing
pub fn canonicalize(value: &Value) -> Value {
    CanonicalEncoder::canonicalize(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_simple_object_ordering() {
        let value = json!({ "z": 3, "a": 1, "m": 2 });
        let canonical = to_canonical_string(&value);
        assert_eq!(canonical, r#"{"a":1,"m":2,"z":3}"#);
    }

    #[test]
    fn test_nested_object_ordering() {
        let value = json!({
            "outer": {
                "z": "last",
                "a": "first",
                "m": "middle"
            },
            "simple": 42
        });
        let canonical = to_canonical_string(&value);
        assert_eq!(
            canonical,
            r#"{"outer":{"a":"first","m":"middle","z":"last"},"simple":42}"#
        );
    }

    #[test]
    fn test_array_preservation() {
        let value = json!({
            "array": [
                { "b": 2, "a": 1 },
                { "d": 4, "c": 3 }
            ]
        });
        let canonical = to_canonical_string(&value);
        assert_eq!(canonical, r#"{"array":[{"a":1,"b":2},{"c":3,"d":4}]}"#);
    }

    #[test]
    fn test_all_json_types() {
        let value = json!({
            "string": "hello",
            "number": 42.5,
            "integer": 123,
            "boolean": true,
            "null_value": null,
            "array": [1, 2, 3],
            "object": { "nested": "value" }
        });

        let canonical = to_canonical_string(&value);
        let expected = r#"{"array":[1,2,3],"boolean":true,"integer":123,"null_value":null,"number":42.5,"object":{"nested":"value"},"string":"hello"}"#;
        assert_eq!(canonical, expected);
    }

    #[test]
    fn test_deep_nesting() {
        let value = json!({
            "level1": {
                "z_last": {
                    "z_deeply_nested": "value",
                    "a_deeply_nested": "another"
                },
                "a_first": "simple"
            }
        });

        let canonical = to_canonical_string(&value);
        let expected = r#"{"level1":{"a_first":"simple","z_last":{"a_deeply_nested":"another","z_deeply_nested":"value"}}}"#;
        assert_eq!(canonical, expected);
    }

    #[test]
    fn test_empty_structures() {
        let value = json!({
            "empty_object": {},
            "empty_array": [],
            "filled": { "key": "value" }
        });

        let canonical = to_canonical_string(&value);
        let expected = r#"{"empty_array":[],"empty_object":{},"filled":{"key":"value"}}"#;
        assert_eq!(canonical, expected);
    }

    #[test]
    fn test_unicode_strings() {
        let value = json!({
            "unicode": "Hello world",
            "multibyte": "cafe resume",
            "escape": "line1\nline2\ttab"
        });

        let canonical = to_canonical_string(&value);
        // Note: serde_json escapes unicode and control characters
        assert!(canonical.contains(r#""unicode":"Hello world""#));
        assert!(canonical.contains(r#""multibyte":"cafe resume""#));
        assert!(canonical.contains(r#""escape":"line1\nline2\ttab""#));
    }
}
