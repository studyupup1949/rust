//! Field type definitions and utilities

/// PDF form field types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldType {
    /// Text field (/Tx)
    Text,
    /// Button field (/Btn) - includes checkboxes, radio buttons, and push buttons
    Button,
    /// Choice field (/Ch) - includes list boxes and combo boxes
    Choice,
    /// Signature field (/Sig)
    Signature,
    /// Unknown or custom field type
    Unknown(String),
}

impl FieldType {
    /// Create a FieldType from a PDF field type name
    pub fn from_name(name: &str) -> Self {
        match name {
            "Tx" => FieldType::Text,
            "Btn" => FieldType::Button,
            "Ch" => FieldType::Choice,
            "Sig" => FieldType::Signature,
            other => FieldType::Unknown(other.to_string()),
        }
    }

    /// Get the PDF field type name
    pub fn to_name(&self) -> &str {
        match self {
            FieldType::Text => "Tx",
            FieldType::Button => "Btn",
            FieldType::Choice => "Ch",
            FieldType::Signature => "Sig",
            FieldType::Unknown(name) => name,
        }
    }
}

impl std::fmt::Display for FieldType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldType::Text => write!(f, "Text"),
            FieldType::Button => write!(f, "Button"),
            FieldType::Choice => write!(f, "Choice"),
            FieldType::Signature => write!(f, "Signature"),
            FieldType::Unknown(name) => write!(f, "Unknown({})", name),
        }
    }
}
