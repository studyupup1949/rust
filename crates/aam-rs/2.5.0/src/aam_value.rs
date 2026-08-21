use std::collections::HashMap;
use std::fmt::Display;
use std::ops::Deref;

/// Represents a value in the AAML map, which can be a string, a list of values, or an inline object.
#[derive(Debug, Clone, PartialEq)]
pub enum AamValue {
    String(String),
    List(Vec<AamValue>),
    Object(HashMap<String, AamValue>),
}

impl AamValue {
    /// Returns the inner string if this is a `String` variant, otherwise returns `None`.
    pub fn as_str(&self) -> Option<&str> {
        if let AamValue::String(s) = self {
            Some(s)
        } else {
            None
        }
    }

    /// Returns link for the list
    pub fn as_list(&self) -> Option<&Vec<AamValue>> {
        if let AamValue::List(l) = self {
            Some(l)
        } else {
            None
        }
    }

    /// Returns links to the fields of an inline object.
    pub fn as_object(&self) -> Option<&HashMap<String, AamValue>> {
        if let AamValue::Object(o) = self {
            Some(o)
        } else {
            None
        }
    }

    pub fn is_list(&self) -> bool {
        matches!(self, AamValue::List(_))
    }

    pub fn is_object(&self) -> bool {
        matches!(self, AamValue::Object(_))
    }
}

impl PartialEq<&str> for AamValue {
    fn eq(&self, other: &&str) -> bool {
        self.as_str().expect("Here must be AamlError") == *other
    }
}

impl Display for AamValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str().expect("Here must be AamlError")) // Will be... Maybe...
    }
}

impl Deref for AamValue {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        self.as_str().expect("Here must be AamlError")
    }
}
