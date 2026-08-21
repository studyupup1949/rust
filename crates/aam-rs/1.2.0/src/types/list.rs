//! `list<T>` — a homogeneous list type.
//!
//! ## Syntax in .aam files
//! ```text
//! tags = [rust, aam, config]
//! scores = [1.0, 2.5, 3.14]
//! flags = [true, false, true]
//! ```
//!
//! The value must be enclosed in square brackets. Items are comma-separated.
//! Each item is validated against the inner type `T`.
//!
//! ## Schema usage
//! ```text
//! @schema Post { tags: list<string>, scores: list<f64> }
//! ```

use crate::error::AamlError;
use crate::types::primitive_type::PrimitiveType;
use crate::types::{Type, resolve_builtin};

/// A list type that validates every element against an inner type.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ListType {
    /// Name of the inner element type (e.g. `"i32"`, `"math::vector3"`).
    pub(crate) inner_type: String,
}

impl ListType {
    /// Creates a `ListType` wrapping the given inner type name.
    pub fn new(inner_type: String) -> Self {
        Self { inner_type }
    }

    /// Parses a `list<T>` type string and returns the inner type name.
    ///
    /// Accepts both `list<T>` and `list<T>` with surrounding whitespace.
    pub fn parse_inner(type_str: &str) -> Option<String> {
        type_str
            .trim()
            .strip_prefix("list<")?
            .strip_suffix('>')?
            .trim()
            .pipe_non_empty()
    }

    /// Splits a `[…]` literal into top-level items, respecting nested `{}` and `[]`.
    ///
    /// `[{a=1, b=2}, {c=3}]` → `["{a=1, b=2}", "{c=3}"]`
    pub fn parse_items(value: &str) -> Option<Vec<String>> {
        let inner = value.trim().strip_prefix('[')?.strip_suffix(']')?;
        Some(split_top_level(inner))
    }
}

/// Splits `s` on commas that are not inside `{}` or `[]` nesting.
pub(crate) fn split_top_level(s: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut depth: i32 = 0;
    let mut cur = String::new();

    for ch in s.chars() {
        match ch {
            '{' | '[' => {
                depth += 1;
                cur.push(ch);
            }
            '}' | ']' => {
                depth -= 1;
                cur.push(ch);
            }
            ',' if depth == 0 => {
                let t = cur.trim().to_string();
                if !t.is_empty() {
                    items.push(t);
                }
                cur.clear();
            }
            _ => {
                cur.push(ch);
            }
        }
    }
    let t = cur.trim().to_string();
    if !t.is_empty() {
        items.push(t);
    }
    items
}

trait PipeStr {
    fn pipe_non_empty(self) -> Option<String>;
}
impl PipeStr for &str {
    fn pipe_non_empty(self) -> Option<String> {
        if self.is_empty() {
            None
        } else {
            Some(self.to_string())
        }
    }
}

impl Type for ListType {
    fn from_name(_name: &str) -> Result<Self, AamlError>
    where
        Self: Sized,
    {
        Err(AamlError::NotFound(
            "ListType::from_name — use ListType::new instead".to_string(),
        ))
    }

    fn base_type(&self) -> PrimitiveType {
        PrimitiveType::String
    }

    /// Validates the list literal `[item, item, ...]` where each item must
    /// satisfy the inner type.
    fn validate(&self, value: &str) -> Result<(), AamlError> {
        let items = ListType::parse_items(value).ok_or_else(|| {
            AamlError::InvalidValue(format!(
                "Expected a list literal in the form [item, item, ...], got '{}'",
                value
            ))
        })?;

        let inner = resolve_builtin(&self.inner_type).map_err(|_| {
            AamlError::NotFound(format!("Unknown list element type '{}'", self.inner_type))
        })?;

        for item in &items {
            inner.validate(item).map_err(|e| {
                AamlError::InvalidValue(format!(
                    "List item '{}' failed validation for type '{}': {}",
                    item, self.inner_type, e
                ))
            })?;
        }

        Ok(())
    }
}
