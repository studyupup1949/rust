//! Canonical JSON encoding for Accumulate protocol
//!
//! Provides deterministic JSON serialization matching TypeScript SDK exactly.

// Allow unwrap in this module - serialization of valid JSON values cannot fail
#![allow(clippy::unwrap_used, clippy::expect_used)]

use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// Convert any serializable value to canonical JSON
/// Recursively uses BTreeMap for maps; produces compact JSON identical to TypeScript SDK
pub fn dumps_canonical<T: Serialize>(value: &T) -> String {
    let json_value = serde_json::to_value(value).expect("Serialization should not fail");
    canonicalize(&json_value)
}

/// Convert a JSON value to canonical JSON string with deterministic ordering
pub fn canonicalize(value: &Value) -> String {
    canonicalize_internal(value)
}

fn canonicalize_internal(value: &Value) -> String {
    match value {
        Value::Null => "null".to_string(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::String(s) => serde_json::to_string(s).unwrap(),
        Value::Array(arr) => {
            let elements: Vec<String> = arr.iter().map(canonicalize_internal).collect();
            format!("[{}]", elements.join(","))
        }
        Value::Object(obj) => {
            // Convert to BTreeMap to ensure sorted keys (canonical ordering)
            let mut sorted: BTreeMap<String, String> = BTreeMap::new();
            for (key, val) in obj {
                sorted.insert(key.clone(), canonicalize_internal(val));
            }

            let pairs: Vec<String> = sorted
                .iter()
                .map(|(k, v)| format!("{}:{}", serde_json::to_string(k).unwrap(), v))
                .collect();

            format!("{{{}}}", pairs.join(","))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_canonicalize_simple() {
        let value = json!({ "z": 3, "a": 1, "m": 2 });
        let canonical = canonicalize(&value);
        assert_eq!(canonical, r#"{"a":1,"m":2,"z":3}"#);
    }

    #[test]
    fn test_canonicalize_nested() {
        let value = json!({
            "z": { "y": 2, "x": 1 },
            "a": 1
        });
        let canonical = canonicalize(&value);
        assert_eq!(canonical, r#"{"a":1,"z":{"x":1,"y":2}}"#);
    }

    #[test]
    fn test_canonicalize_array() {
        let value = json!({
            "arr": [{ "b": 2, "a": 1 }, { "d": 4, "c": 3 }]
        });
        let canonical = canonicalize(&value);
        assert_eq!(canonical, r#"{"arr":[{"a":1,"b":2},{"c":3,"d":4}]}"#);
    }

    #[test]
    fn test_dumps_canonical() {
        use serde::Serialize;

        #[derive(Serialize)]
        struct TestStruct {
            z: i32,
            a: i32,
            m: i32,
        }

        let test_obj = TestStruct { z: 3, a: 1, m: 2 };
        let canonical = dumps_canonical(&test_obj);
        assert_eq!(canonical, r#"{"a":1,"m":2,"z":3}"#);
    }

    #[test]
    fn test_primitives() {
        assert_eq!(canonicalize(&json!(null)), "null");
        assert_eq!(canonicalize(&json!(true)), "true");
        assert_eq!(canonicalize(&json!(false)), "false");
        assert_eq!(canonicalize(&json!(42)), "42");
        assert_eq!(canonicalize(&json!(3.14)), "3.14");
        assert_eq!(canonicalize(&json!("hello")), r#""hello""#);
        assert_eq!(canonicalize(&json!([])), "[]");
        assert_eq!(canonicalize(&json!({})), "{}");
    }
}
