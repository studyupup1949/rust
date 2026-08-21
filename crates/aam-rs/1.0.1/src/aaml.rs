use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use crate::error::AamlError;
use crate::found_value::FoundValue;

#[derive(Debug, Clone)]
pub struct AAML {
    map: HashMap<String, String>,
}

impl AAML {
    pub fn parse(content: &str) -> Result<Self, AamlError> {
        let mut map = HashMap::with_capacity(content.lines().count());

        for (index, line) in content.lines().enumerate() {
            let clean_line = Self::strip_comment(line).trim();

            if clean_line.is_empty() {
                continue;
            }

            if let Some((key_part, value_part)) = clean_line.split_once('=') {
                let key = key_part.trim();
                let mut val = value_part.trim();

                if key.is_empty() {
                    return Err(AamlError::ParseError {
                        line: index + 1,
                        content: line.to_string(),
                        details: "Key cannot be empty".to_string(),
                    });
                }

                if (val.starts_with('"') && val.ends_with('"')) || (val.starts_with('\'') && val.ends_with('\'')) {
                    if val.len() >= 2 {
                        val = &val[1..val.len() - 1];
                    }
                }

                map.insert(key.to_string(), val.to_string());
            } else {
                return Err(AamlError::ParseError {
                    line: index + 1,
                    content: line.to_string(),
                    details: "Missing assignment operator '='".to_string(),
                });
            }
        }

        Ok(AAML { map })
    }

    pub fn load<P: AsRef<Path>>(file_path: P) -> Result<Self, AamlError> {
        let content = fs::read_to_string(file_path)?;
        Self::parse(&content)
    }

    pub fn find_obj(&self, key: &str) -> Option<FoundValue> {
        self.map.get(key)
            .map(|v| FoundValue::new(v))
            .or_else(|| self.find_key(key))
    }

    pub fn find_deep(&self, key: &str) -> Option<FoundValue> {
        let mut current_key = key;
        let mut last_found = None;
        let mut visited = HashSet::new();

        while let Some(next_val) = self.map.get(current_key) {
            let next_val_str = next_val.as_str();

            if !visited.insert(current_key) {
                break;
            }

            if visited.contains(next_val_str) {
                if last_found.is_none() {
                    last_found = Some(next_val_str);
                }
                break;
            }

            last_found = Some(next_val_str);
            current_key = next_val_str;
        }

        last_found.map(FoundValue::new)
    }

    pub fn find_key(&self, value: &str) -> Option<FoundValue> {
        self.map.iter()
            .find_map(|(k, v)| {
                if v == value {
                    Some(FoundValue::new(k))
                } else {
                    None
                }
            })
    }

    fn strip_comment(line: &str) -> &str {
        let mut in_quote = false;
        let mut quote_char = '\0';

        for (idx, c) in line.char_indices() {
            if c == '"' || c == '\'' {
                if !in_quote {
                    in_quote = true;
                    quote_char = c;
                } else if c == quote_char {
                    in_quote = false;
                }
            }

            if c == '#' && !in_quote {
                return &line[..idx];
            }
        }
        line
    }
}