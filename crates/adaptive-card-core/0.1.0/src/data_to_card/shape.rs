//! Detect the presentational shape best suited for a given input data value.

use crate::types::Presentation;
use serde_json::Value;

/// Detect the best presentation from a data shape. Falls back to [`Presentation::List`]
/// when nothing specific matches.
#[must_use]
pub fn detect(data: &Value) -> Presentation {
    match data {
        Value::Array(arr) if arr.is_empty() => Presentation::List,
        Value::Array(arr) => {
            if arr.iter().all(Value::is_object) {
                // Array of objects → Table if consistent keys, else List.
                let first_keys: Vec<String> = arr[0]
                    .as_object()
                    .map(|m| m.keys().cloned().collect())
                    .unwrap_or_default();
                let consistent = arr.iter().all(|el| {
                    el.as_object()
                        .is_some_and(|m| first_keys.iter().all(|k| m.contains_key(k)))
                });
                if consistent && first_keys.len() >= 2 {
                    Presentation::Table
                } else {
                    Presentation::List
                }
            } else {
                Presentation::List
            }
        }
        Value::Object(map) => {
            let all_primitive = map.values().all(|v| {
                matches!(
                    v,
                    Value::String(_) | Value::Number(_) | Value::Bool(_) | Value::Null
                )
            });
            if all_primitive {
                Presentation::FactSet
            } else {
                Presentation::List
            }
        }
        _ => Presentation::List,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn single_object_is_factset() {
        let data = json!({ "name": "Alice", "age": 30 });
        assert!(matches!(detect(&data), Presentation::FactSet));
    }

    #[test]
    fn array_of_objects_consistent_is_table() {
        let data = json!([
            { "region": "APAC", "revenue": 1000 },
            { "region": "EMEA", "revenue": 2000 }
        ]);
        assert!(matches!(detect(&data), Presentation::Table));
    }

    #[test]
    fn array_of_strings_is_list() {
        let data = json!(["apple", "banana", "cherry"]);
        assert!(matches!(detect(&data), Presentation::List));
    }

    #[test]
    fn empty_array_defaults_to_list() {
        let data = json!([]);
        assert!(matches!(detect(&data), Presentation::List));
    }
}
