//! Field traversal and name resolution utilities

use crate::types::FieldValue;
use acroform_pdf::error::PdfError;
use acroform_pdf::object::{FieldDictionary, FieldType, Resolve};
use acroform_pdf::primitive::{PdfString, Primitive};

/// High-level representation of a form field
#[derive(Debug, Clone)]
pub struct FormField {
    /// Full qualified name of the field (e.g., "parent.child")
    pub name: String,
    /// Type of the field
    pub field_type: Option<FieldType>,
    /// Current value of the field
    pub current_value: Option<FieldValue>,
    /// Field flags
    pub flags: u32,
}

/// Trait for traversing form fields
pub trait FieldTraversal {
    /// Get the full qualified name of a field by traversing parent chain
    fn get_full_name(&self, resolve: &impl Resolve) -> Result<String, PdfError>;

    /// Collect all terminal fields (fields with actual values, not just containers)
    fn collect_terminal_fields(
        &self,
        resolve: &impl Resolve,
        fields: &mut Vec<FormField>,
    ) -> Result<(), PdfError>;
}

impl FieldTraversal for FieldDictionary {
    fn get_full_name(&self, resolve: &impl Resolve) -> Result<String, PdfError> {
        let mut name_parts = Vec::new();

        // Add this field's name if present
        if let Some(ref name) = self.name {
            name_parts.push(name.to_string_lossy());
        }

        // Walk up the parent chain
        let mut current_parent = self.parent.clone();
        while let Some(parent_ref) = current_parent {
            let parent = resolve.get(parent_ref)?;
            if let Some(ref name) = parent.name {
                name_parts.push(name.to_string_lossy());
            }
            current_parent = parent.parent.clone();
        }

        // Reverse to get parent.child order
        name_parts.reverse();
        Ok(name_parts.join("."))
    }

    fn collect_terminal_fields(
        &self,
        resolve: &impl Resolve,
        fields: &mut Vec<FormField>,
    ) -> Result<(), PdfError> {
        // Check if this field has kids
        if !self.kids.is_empty() {
            // This is a container field, recurse into kids
            for kid_ref in &self.kids {
                let kid = resolve.get(*kid_ref)?;
                kid.collect_terminal_fields(resolve, fields)?;
            }
        } else {
            // This is a terminal field (leaf node), add it to the list
            let name = self.get_full_name(resolve)?;
            let current_value = primitive_to_field_value(&self.value);

            fields.push(FormField {
                name,
                field_type: self.typ,
                current_value,
                flags: self.flags,
            });
        }

        Ok(())
    }
}

/// Convert a PDF primitive to a FieldValue
fn primitive_to_field_value(primitive: &Primitive) -> Option<FieldValue> {
    match primitive {
        Primitive::String(s) => Some(FieldValue::Text(s.to_string_lossy())),
        Primitive::Integer(i) => Some(FieldValue::Integer(*i)),
        Primitive::Name(n) => Some(FieldValue::Choice(n.to_string())),
        Primitive::Boolean(b) => Some(FieldValue::Boolean(*b)),
        Primitive::Null => None,
        _ => None,
    }
}

/// Convert a FieldValue to a PDF primitive
pub fn field_value_to_primitive(value: &FieldValue) -> Primitive {
    match value {
        FieldValue::Text(s) => Primitive::String(PdfString::new(s.as_bytes().to_vec().into())),
        FieldValue::Integer(i) => Primitive::Integer(*i),
        FieldValue::Choice(s) => Primitive::Name(s.as_str().into()),
        FieldValue::Boolean(b) => Primitive::Boolean(*b),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primitive_conversion() {
        let text = FieldValue::Text("Hello".to_string());
        let primitive = field_value_to_primitive(&text);
        match primitive {
            Primitive::String(_) => (),
            _ => panic!("Expected String primitive"),
        }
    }
}
