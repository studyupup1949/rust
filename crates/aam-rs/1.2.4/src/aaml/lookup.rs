//! Key lookup methods for [`AAML`](AAML).

use std::collections::HashSet;
use crate::found_value::FoundValue;
use super::{AAML, Hasher};

impl AAML {
    /// Looks up `key` in the map. If not found as a key, performs a reverse
    /// lookup — searching for an entry whose *value* matches `key`.
    pub fn find_obj(&self, key: &str) -> Option<FoundValue> {
        self.map
            .get(key)
            .map(|v| FoundValue::new(v))
            .or_else(|| self.find_key(key))
    }

    /// Reverse lookup: finds the key whose value equals `value`.
    pub fn find_key(&self, value: &str) -> Option<FoundValue> {
        self.map
            .iter()
            .find_map(|(k, v)| (&**v == value).then(|| FoundValue::new(k)))
    }

    /// Follows a chain of key -> value -> key lookups until a terminal value
    /// is reached or a cycle is detected.
    pub fn find_deep(&self, key: &str) -> Option<FoundValue> {
        let mut current_key = key;
        let mut last_found = None;
        let mut visited: HashSet<&str, Hasher> = HashSet::with_hasher(Hasher::default());

        while let Some(next_val) = self.map.get(current_key) {
            if !visited.insert(current_key) {
                break;
            }
            if visited.contains(&**next_val) {
                if last_found.is_none() {
                    last_found = Some(next_val);
                }
                break;
            }
            last_found = Some(next_val);
            current_key = next_val;
        }

        last_found.map(|v| FoundValue::new(v))
    }
}