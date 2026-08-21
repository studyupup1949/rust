use pdf::error::PdfError;
use pdf::file::{CachedFile, FileOptions};
use pdf::object::{FieldDictionary, FieldType, RcRef, Updater, Annot};
use pdf::primitive::{Primitive, PdfString, Dictionary};
use std::collections::HashMap;
use std::path::Path;

use crate::field::{FieldDictionaryExt, InteractiveFormDictionaryExt};

/// High-level representation of a form field
///
/// This struct contains all the information needed to understand and manipulate
/// a PDF form field, including its name, type, current value, and flags.
#[derive(Debug, Clone)]
pub struct FormField {
    /// The fully qualified name of the field (e.g., "parent.child.field")
    pub name: String,
    /// The type of the field (e.g., Text, Button, Choice)
    pub field_type: FieldType,
    /// The current value of the field, if any
    pub current_value: Option<FieldValue>,
    /// Field flags as defined in the PDF specification
    pub flags: u32,
}

/// Typed representation of field values
///
/// This enum represents the different types of values that can be stored in PDF form fields.
/// Each variant corresponds to a specific field type in the PDF specification.
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// Text field value (used for text input fields)
    Text(String),
    /// Boolean value (used for checkboxes and radio buttons)
    Boolean(bool),
    /// Choice value (used for dropdown menus and radio button selections)
    Choice(String),
    /// Integer value (used for numeric fields)
    Integer(i32),
}

impl FieldValue {
    /// Convert a PDF Primitive to a FieldValue
    ///
    /// This method attempts to convert a PDF primitive value (String, Integer, Name, Boolean)
    /// into a typed `FieldValue`. Returns `None` if the primitive type is not supported.
    ///
    /// This is primarily an internal method used when reading field values from PDFs.
    pub fn from_primitive(prim: &Primitive) -> Option<Self> {
        match prim {
            Primitive::String(s) => Some(FieldValue::Text(s.to_string_lossy().to_string())),
            Primitive::Integer(i) => Some(FieldValue::Integer(*i)),
            Primitive::Name(n) => Some(FieldValue::Choice(n.to_string())),
            Primitive::Boolean(b) => Some(FieldValue::Boolean(*b)),
            _ => None,
        }
    }
    
    /// Convert a FieldValue to a PDF Primitive
    ///
    /// This method converts a typed `FieldValue` into the corresponding PDF primitive
    /// that can be written to a PDF file.
    ///
    /// This is primarily an internal method used when writing field values to PDFs.
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
///
/// This struct provides the primary interface for loading PDF files,
/// reading form fields, and filling form values.
///
/// # Examples
///
/// ```no_run
/// use acroform::{AcroFormDocument, FieldValue};
/// use std::collections::HashMap;
///
/// let mut doc = AcroFormDocument::from_pdf("form.pdf").unwrap();
///
/// // List all fields
/// for field in doc.fields().unwrap() {
///     println!("{}: {:?}", field.name, field.current_value);
/// }
///
/// // Fill fields
/// let mut values = HashMap::new();
/// values.insert("name".to_string(), FieldValue::Text("John".to_string()));
/// doc.fill_and_save(values, "filled.pdf").unwrap();
/// ```
pub struct AcroFormDocument {
    file: CachedFile<Vec<u8>>,
}

impl AcroFormDocument {
    /// Load a PDF file from the given path
    ///
    /// Opens and parses a PDF file, preparing it for form field manipulation.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the PDF file to load
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if the file cannot be opened or parsed.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use acroform::AcroFormDocument;
    ///
    /// let doc = AcroFormDocument::from_pdf("form.pdf").unwrap();
    /// ```
    pub fn from_pdf(path: impl AsRef<Path>) -> Result<Self, PdfError> {
        let file = FileOptions::cached().open(path)?;
        Ok(AcroFormDocument { file })
    }
    
    /// Load a PDF from a byte vector
    ///
    /// Parses a PDF from an in-memory byte vector, preparing it for form field manipulation.
    ///
    /// # Arguments
    ///
    /// * `data` - A byte vector containing the PDF data
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if the data cannot be parsed as a valid PDF.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use acroform::AcroFormDocument;
    /// use std::fs;
    ///
    /// let data = fs::read("form.pdf").unwrap();
    /// let doc = AcroFormDocument::from_bytes(data).unwrap();
    /// ```
    pub fn from_bytes(data: Vec<u8>) -> Result<Self, PdfError> {
        let file = FileOptions::cached().load(data)?;
        Ok(AcroFormDocument { file })
    }
    
    /// Get all form fields in the PDF
    ///
    /// Returns a vector of all fillable form fields in the document.
    /// Each field includes its name, type, current value, and flags.
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if field information cannot be retrieved from the PDF.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use acroform::AcroFormDocument;
    ///
    /// let doc = AcroFormDocument::from_pdf("form.pdf").unwrap();
    /// for field in doc.fields().unwrap() {
    ///     println!("Field: {} (type: {:?})", field.name, field.field_type);
    /// }
    /// ```
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
    
    /// Fill form fields with provided values and return the PDF as a byte vector
    ///
    /// Updates the specified form fields with new values and returns the modified
    /// PDF as an in-memory byte vector. Fields not specified in the `values` map remain unchanged.
    ///
    /// This method performs all operations in-memory without writing to disk,
    /// making it suitable for web services, stream processing, or other scenarios
    /// where disk I/O should be avoided.
    ///
    /// # Arguments
    ///
    /// * `values` - A map from field names to their new values
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if:
    /// - The PDF does not contain an AcroForm dictionary
    /// - Field updates cannot be applied
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use acroform::{AcroFormDocument, FieldValue};
    /// use std::collections::HashMap;
    ///
    /// let mut doc = AcroFormDocument::from_pdf("form.pdf").unwrap();
    /// let mut values = HashMap::new();
    /// values.insert("firstName".to_string(), FieldValue::Text("John".to_string()));
    /// values.insert("lastName".to_string(), FieldValue::Text("Doe".to_string()));
    /// let filled_pdf = doc.fill(values).unwrap();
    /// // Now you can send `filled_pdf` over HTTP, store it in a database, etc.
    /// ```
    pub fn fill(
        &mut self,
        values: HashMap<String, FieldValue>,
    ) -> Result<Vec<u8>, PdfError> {
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
        
        // Return the file as bytes instead of saving to disk
        Ok(self.file.save()?)
    }
    
    /// Fill form fields with provided values and save to a new file
    ///
    /// Updates the specified form fields with new values and writes the modified
    /// PDF to the output path. Fields not specified in the `values` map remain unchanged.
    ///
    /// This is a convenience method that combines `fill()` with writing to disk.
    /// For in-memory operations, use `fill()` directly.
    ///
    /// # Arguments
    ///
    /// * `values` - A map from field names to their new values
    /// * `output` - Path where the filled PDF should be saved
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if:
    /// - The PDF does not contain an AcroForm dictionary
    /// - Field updates cannot be applied
    /// - The file cannot be written to the output path
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use acroform::{AcroFormDocument, FieldValue};
    /// use std::collections::HashMap;
    ///
    /// let mut doc = AcroFormDocument::from_pdf("form.pdf").unwrap();
    /// let mut values = HashMap::new();
    /// values.insert("firstName".to_string(), FieldValue::Text("John".to_string()));
    /// values.insert("lastName".to_string(), FieldValue::Text("Doe".to_string()));
    /// doc.fill_and_save(values, "filled_form.pdf").unwrap();
    /// ```
    pub fn fill_and_save(
        &mut self,
        values: HashMap<String, FieldValue>,
        output: impl AsRef<Path>,
    ) -> Result<(), PdfError> {
        let bytes = self.fill(values)?;
        std::fs::write(output, bytes)?;
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
