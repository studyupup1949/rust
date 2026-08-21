use pdf::error::PdfError;
use pdf::file::{CachedFile, FileOptions};
use pdf::object::{FieldDictionary, FieldType, RcRef, Updater, Annot};
use pdf::primitive::{Primitive, PdfString, Dictionary};
use std::collections::HashMap;
use std::path::Path;

use crate::field::{FieldDictionaryExt, InteractiveFormDictionaryExt};

/// High-level representation of a form field
#[derive(Debug, Clone)]
pub struct FormField {
    pub name: String,
    pub field_type: FieldType,
    pub current_value: Option<FieldValue>,
    pub flags: u32,
}

/// Typed representation of field values
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    Text(String),
    Boolean(bool),
    Choice(String),
    Integer(i32),
}

impl FieldValue {
    /// Convert a Primitive to a FieldValue
    pub fn from_primitive(prim: &Primitive) -> Option<Self> {
        match prim {
            Primitive::String(s) => Some(FieldValue::Text(s.to_string_lossy().to_string())),
            Primitive::Integer(i) => Some(FieldValue::Integer(*i)),
            Primitive::Name(n) => Some(FieldValue::Choice(n.to_string())),
            Primitive::Boolean(b) => Some(FieldValue::Boolean(*b)),
            _ => None,
        }
    }
    
    /// Convert a FieldValue to a Primitive
    pub fn to_primitive(&self) -> Primitive {
        match self {
            FieldValue::Text(s) => Primitive::String(PdfString::new(s.as_bytes().into())),
            FieldValue::Integer(i) => Primitive::Integer(*i),
            FieldValue::Choice(s) => Primitive::Name(s.as_str().into()),
            FieldValue::Boolean(b) => Primitive::Boolean(*b),
        }
    }
}

/// Main API for working with PDF forms
pub struct AcroFormDocument {
    file: CachedFile<Vec<u8>>,
}

impl AcroFormDocument {
    /// Load a PDF file
    pub fn from_pdf(path: impl AsRef<Path>) -> Result<Self, PdfError> {
        let file = FileOptions::cached().open(path)?;
        Ok(AcroFormDocument { file })
    }
    
    /// Get all form fields
    pub fn fields(&self) -> Result<Vec<FormField>, PdfError> {
        let mut result = Vec::new();
        
        if let Some(ref forms) = self.file.get_root().forms {
            let resolver = self.file.resolver();
            let all_fields: Vec<RcRef<FieldDictionary>> = forms.all_fields(&resolver)?;
            
            for field in all_fields {
                if let Some(field_type) = field.typ {
                    let name = field.get_full_name(&resolver)?;
                    let current_value = FieldValue::from_primitive(&field.value);
                    
                    result.push(FormField {
                        name,
                        field_type,
                        current_value,
                        flags: field.flags,
                    });
                }
            }
        }
        
        Ok(result)
    }
    
    /// Fill fields and save to a new file
    pub fn fill_and_save(
        &mut self,
        values: HashMap<String, FieldValue>,
        output: impl AsRef<Path>,
    ) -> Result<(), PdfError> {
        // Collect field references and their values to update
        let mut field_updates: Vec<(pdf::object::PlainRef, FieldDictionary)> = Vec::new();
        let mut annotation_updates: Vec<(pdf::object::PlainRef, Annot)> = Vec::new();
        
        {
            // Get the forms dictionary
            let forms = self.file.get_root().forms.as_ref()
                .ok_or_else(|| PdfError::MissingEntry { 
                    typ: "Catalog",
                    field: "AcroForm".into() 
                })?;
            
            // Find fields to update
            let resolver = self.file.resolver();
            for (name, value) in &values {
                if let Some(field) = forms.find_field_by_name(&name, &resolver)? {
                    let field_ref = field.get_ref();
                    let mut updated_field = (*field).clone();
                    updated_field.value = value.to_primitive();
                    field_updates.push((field_ref.get_inner(), updated_field));
                }
            }
            
            // Also update page annotations that represent the same fields
            for page_rc in self.file.pages() {
                let page = page_rc?;
                let annots = page.annotations.load(&resolver)?;
                
                for annot_ref in annots.data().iter() {
                    let annot = annot_ref.data();
                    
                    // Check if this annotation has a field name (T key)
                    if let Some(Primitive::String(ref field_name)) = annot.other.get("T") {
                        let field_name_str = field_name.to_string_lossy().to_string();
                        
                        // Check if we're updating this field
                        if let Some(value) = values.get(&field_name_str) {
                            // Get the annotation reference if it's an indirect reference
                            if let Some(annot_ref_val) = annot_ref.as_ref() {
                                // Clone the annotation and update its value in the other dictionary
                                let mut updated_annot = (**annot).clone();
                                let mut new_other = Dictionary::new();
                                
                                // Copy all existing entries
                                for (key, val) in &annot.other {
                                    new_other.insert(key.clone(), val.clone());
                                }
                                
                                // Update the value
                                new_other.insert("V", value.to_primitive());
                                updated_annot.other = new_other;
                                
                                annotation_updates.push((annot_ref_val.get_inner(), updated_annot));
                            }
                        }
                    }
                }
            }
        } // resolver and forms are dropped here
        
        // Apply field updates
        for (field_ref, updated_field) in field_updates {
            self.file.update(field_ref, updated_field)?;
        }
        
        // Apply annotation updates
        for (annot_ref, updated_annot) in annotation_updates {
            self.file.update(annot_ref, updated_annot)?;
        }
        
        // Save the file
        self.file.save_to(output)?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_field_value_conversion() {
        let text = FieldValue::Text("hello".to_string());
        let prim = text.to_primitive();
        let back = FieldValue::from_primitive(&prim).unwrap();
        assert_eq!(text, back);
        
        let int = FieldValue::Integer(42);
        let prim = int.to_primitive();
        let back = FieldValue::from_primitive(&prim).unwrap();
        assert_eq!(int, back);
    }
}
