//! Type definitions for AcroForm values

use std::fmt;

/// Represents a value that can be assigned to a form field
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// Text value (for text fields)
    Text(String),
    /// Boolean value (for checkboxes)
    Boolean(bool),
    /// Choice value (for radio buttons and dropdowns)
    Choice(String),
    /// Integer value (for numeric fields)
    Integer(i32),
}

impl fmt::Display for FieldValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FieldValue::Text(s) => write!(f, "{}", s),
            FieldValue::Boolean(b) => write!(f, "{}", b),
            FieldValue::Choice(s) => write!(f, "{}", s),
            FieldValue::Integer(i) => write!(f, "{}", i),
        }
    }
}
