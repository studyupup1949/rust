//! High-level API for PDF form manipulation

use crate::{FieldType, Result};
use lopdf::{Dictionary, Document, Object, ObjectId, StringFormat};
use std::collections::HashMap;
use std::path::Path;

/// A PDF document with form fields
pub struct AcroFormDocument {
    doc: Document,
}

impl AcroFormDocument {
    /// Load a PDF document from a file path
    pub fn from_pdf(path: impl AsRef<Path>) -> Result<Self> {
        let doc = Document::load(path)?;
        Ok(AcroFormDocument { doc })
    }

    /// Load a PDF document from bytes
    pub fn from_bytes(data: Vec<u8>) -> Result<Self> {
        use std::io::Cursor;
        let doc = Document::load_from(Cursor::new(data))?;
        Ok(AcroFormDocument { doc })
    }

    /// Get all form fields in the document
    pub fn fields(&self) -> Result<Vec<FormField>> {
        let mut results = Vec::new();

        // Get AcroForm dictionary
        let acroform = get_acroform_dict(&self.doc)?;

        // Traverse Fields array
        if let Ok(fields_array) = acroform.get(b"Fields").and_then(|obj| obj.as_array()) {
            for field_obj in fields_array {
                if let Ok(field_ref) = field_obj.as_reference() {
                    traverse_field_tree(&self.doc, field_ref, None, &mut results)?;
                }
            }
        }

        Ok(results)
    }

    /// Fill form fields with values and return the modified PDF as bytes
    pub fn fill(&mut self, values: HashMap<String, FieldValue>) -> Result<Vec<u8>> {
        // Get AcroForm reference
        let acroform_ref = self.doc.catalog()?.get(b"AcroForm")?.as_reference()?;

        // Update each field
        for (field_name, new_value) in &values {
            if let Some(field_ref) = self.find_field_by_name(field_name)? {
                // Update field value
                let field_dict = self.doc.get_dictionary_mut(field_ref)?;
                field_dict.set("V", new_value.to_object());
                field_dict.set("DV", new_value.to_object());

                // Update appearance state for buttons/choices
                if matches!(new_value, FieldValue::Choice(_) | FieldValue::Boolean(_)) {
                    field_dict.set("AS", new_value.to_object());
                }

                // Update widget annotations in the field's Kids array
                self.update_field_widget_kids(field_ref, new_value)?;
            }
        }

        // Update widget annotations on pages that reference the same fields
        self.update_page_widget_annotations(&values)?;

        // Set NeedAppearances flag
        let acroform = self.doc.get_dictionary_mut(acroform_ref)?;
        acroform.set("NeedAppearances", Object::Boolean(true));

        // Save to bytes
        let mut buffer = Vec::new();
        self.doc.save_to(&mut buffer)?;
        Ok(buffer)
    }

    /// Update widget annotations in a field's Kids array
    fn update_field_widget_kids(
        &mut self,
        field_ref: ObjectId,
        new_value: &FieldValue,
    ) -> Result<()> {
        // Get the field dictionary to check for Kids
        let field_dict = self.doc.get_dictionary(field_ref)?;

        // Get Kids array if it exists
        let kids_array = match field_dict.get(b"Kids").and_then(|obj| obj.as_array()) {
            Ok(arr) => arr.clone(),
            Err(_) => return Ok(()), // No Kids, nothing to update
        };

        // Update each kid that is a widget
        for kid_obj in kids_array {
            if let Ok(kid_ref) = kid_obj.as_reference() {
                let kid_dict = match self.doc.get_dictionary(kid_ref) {
                    Ok(dict) => dict,
                    Err(_) => continue,
                };

                // Check if this is a widget annotation
                let is_widget = kid_dict
                    .get(b"Subtype")
                    .and_then(|obj| obj.as_name())
                    .map(|name| name == b"Widget")
                    .unwrap_or(false);

                if is_widget {
                    // Update the widget annotation value
                    let kid_dict_mut = self.doc.get_dictionary_mut(kid_ref)?;
                    kid_dict_mut.set("V", new_value.to_object());
                    kid_dict_mut.set("DV", new_value.to_object());

                    // Update appearance state for buttons/choices
                    if matches!(new_value, FieldValue::Choice(_) | FieldValue::Boolean(_)) {
                        kid_dict_mut.set("AS", new_value.to_object());
                    }
                }
            }
        }

        Ok(())
    }

    /// Update widget annotations on pages to match field values
    fn update_page_widget_annotations(
        &mut self,
        values: &HashMap<String, FieldValue>,
    ) -> Result<()> {
        // Get all pages
        let pages = self.doc.get_pages();
        let page_ids: Vec<ObjectId> = pages.values().copied().collect();

        for page_id in page_ids {
            // Get page dictionary
            let page_dict = match self.doc.get_dictionary(page_id) {
                Ok(dict) => dict,
                Err(_) => continue,
            };

            // Get annotations array if it exists
            let annots_array = match page_dict.get(b"Annots") {
                Ok(annots_obj) => {
                    // Handle both direct arrays and references to arrays
                    match annots_obj {
                        Object::Array(arr) => arr.clone(),
                        Object::Reference(annots_ref) => {
                            match self
                                .doc
                                .get_object(*annots_ref)
                                .and_then(|obj| obj.as_array())
                            {
                                Ok(arr) => arr.clone(),
                                Err(_) => continue,
                            }
                        }
                        _ => continue,
                    }
                }
                Err(_) => continue,
            };

            // Check each annotation
            for annot_obj in annots_array {
                if let Ok(annot_ref) = annot_obj.as_reference() {
                    // Get annotation dictionary
                    let annot_dict = match self.doc.get_dictionary(annot_ref) {
                        Ok(dict) => dict,
                        Err(_) => continue,
                    };

                    // Check if this is a widget annotation
                    let is_widget = annot_dict
                        .get(b"Subtype")
                        .and_then(|obj| obj.as_name())
                        .map(|name| name == b"Widget")
                        .unwrap_or(false);

                    if !is_widget {
                        continue;
                    }

                    // Get the field name from the annotation
                    let field_name = match annot_dict
                        .get(b"T")
                        .and_then(|obj| obj.as_str())
                        .map(decode_text_string)
                    {
                        Ok(name) => name,
                        Err(_) => continue,
                    };

                    // Check if we're updating this field
                    if let Some(new_value) = values.get(&field_name) {
                        // Update the annotation value
                        let annot_dict_mut = self.doc.get_dictionary_mut(annot_ref)?;
                        annot_dict_mut.set("V", new_value.to_object());
                        annot_dict_mut.set("DV", new_value.to_object());

                        // Update appearance state for buttons/choices
                        if matches!(new_value, FieldValue::Choice(_) | FieldValue::Boolean(_)) {
                            annot_dict_mut.set("AS", new_value.to_object());
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Fill form fields with values and save to a file
    pub fn fill_and_save(
        &mut self,
        values: HashMap<String, FieldValue>,
        output: impl AsRef<Path>,
    ) -> Result<()> {
        let bytes = self.fill(values)?;
        std::fs::write(output, bytes)?;
        Ok(())
    }

    /// Find a field by name
    fn find_field_by_name(&self, name: &str) -> Result<Option<ObjectId>> {
        let acroform = get_acroform_dict(&self.doc)?;
        if let Ok(fields_array) = acroform.get(b"Fields").and_then(|obj| obj.as_array()) {
            for field_obj in fields_array {
                if let Ok(field_ref) = field_obj.as_reference() {
                    if let Some(found) = self.search_field_tree(field_ref, name, None)? {
                        return Ok(Some(found));
                    }
                }
            }
        }
        Ok(None)
    }

    /// Search for a field in the field tree
    fn search_field_tree(
        &self,
        field_ref: ObjectId,
        target_name: &str,
        parent_name: Option<String>,
    ) -> Result<Option<ObjectId>> {
        let field_dict = self.doc.get_dictionary(field_ref)?;

        let field_name = get_field_name(field_dict);
        let full_name = match (parent_name, field_name) {
            (Some(p), Some(n)) => format!("{}.{}", p, n),
            (None, Some(n)) => n,
            (Some(p), None) => p,
            (None, None) => String::new(),
        };

        if full_name == target_name {
            return Ok(Some(field_ref));
        }

        // Search children
        if let Ok(kids) = field_dict.get(b"Kids").and_then(|obj| obj.as_array()) {
            for kid_obj in kids {
                if let Ok(kid_ref) = kid_obj.as_reference() {
                    if let Some(found) =
                        self.search_field_tree(kid_ref, target_name, Some(full_name.clone()))?
                    {
                        return Ok(Some(found));
                    }
                }
            }
        }

        Ok(None)
    }
}

/// A form field in a PDF document
#[derive(Debug, Clone)]
pub struct FormField {
    /// The fully qualified name of the field (e.g., "parent.child.field")
    pub name: String,
    /// The type of the field
    pub field_type: FieldType,
    /// The current value of the field
    pub current_value: Option<FieldValue>,
    /// The default value of the field
    pub default_value: Option<FieldValue>,
    /// Field flags (bit field)
    pub flags: u32,
    /// Tooltip text (alternate description)
    pub tooltip: Option<String>,
}

/// A typed value for a form field
#[derive(Debug, Clone, PartialEq)]
pub enum FieldValue {
    /// Text string
    Text(String),
    /// Boolean value (for checkboxes)
    Boolean(bool),
    /// Choice value (for radio buttons, list boxes, combo boxes)
    Choice(String),
    /// Integer value
    Integer(i32),
}

impl FieldValue {
    /// Create a FieldValue from a PDF object
    pub fn from_object(obj: &Object) -> Option<Self> {
        match obj {
            Object::String(bytes, _) => Some(FieldValue::Text(decode_text_string(bytes))),
            Object::Integer(i) => Some(FieldValue::Integer(*i as i32)),
            Object::Name(n) => Some(FieldValue::Choice(String::from_utf8_lossy(n).to_string())),
            Object::Boolean(b) => Some(FieldValue::Boolean(*b)),
            _ => None,
        }
    }

    /// Convert to a PDF object
    pub fn to_object(&self) -> Object {
        match self {
            FieldValue::Text(s) => {
                // Encode as UTF-16BE with BOM
                let bytes = encode_text_utf16be(s);
                Object::String(bytes, StringFormat::Literal)
            }
            FieldValue::Integer(i) => Object::Integer(*i as i64),
            FieldValue::Choice(s) => Object::Name(s.as_bytes().to_vec()),
            FieldValue::Boolean(b) => Object::Boolean(*b),
        }
    }
}

// Helper functions

/// Get the AcroForm dictionary from the document catalog
fn get_acroform_dict(doc: &Document) -> Result<&Dictionary> {
    let catalog = doc.catalog()?;
    let acroform_ref = catalog.get(b"AcroForm")?.as_reference()?;
    Ok(doc.get_dictionary(acroform_ref)?)
}

/// Get the field type from a field dictionary
fn get_field_type(field_dict: &Dictionary) -> Option<String> {
    field_dict
        .get(b"FT")
        .ok()
        .and_then(|obj| obj.as_name().ok())
        .map(|bytes| String::from_utf8_lossy(bytes).to_string())
}

/// Get the field name from a field dictionary
fn get_field_name(field_dict: &Dictionary) -> Option<String> {
    field_dict
        .get(b"T")
        .ok()
        .and_then(|obj| obj.as_str().ok())
        .map(decode_text_string)
}

/// Get the field value from a field dictionary
fn get_field_value(field_dict: &Dictionary) -> Option<Object> {
    field_dict.get(b"V").ok().cloned()
}

/// Traverse the field tree and collect all fields
fn traverse_field_tree(
    doc: &Document,
    field_ref: ObjectId,
    parent_name: Option<String>,
    results: &mut Vec<FormField>,
) -> Result<()> {
    let field_dict = doc.get_dictionary(field_ref)?;

    // Get field name
    let field_name = get_field_name(field_dict);
    let full_name = match (parent_name.clone(), field_name) {
        (Some(p), Some(n)) => format!("{}.{}", p, n),
        (None, Some(n)) => n,
        (Some(p), None) => p,
        (None, None) => String::new(),
    };

    // Check if this is a terminal field (has FT key)
    if let Some(field_type_name) = get_field_type(field_dict) {
        // Extract field information
        let current_value =
            get_field_value(field_dict).and_then(|obj| FieldValue::from_object(&obj));

        let default_value = field_dict.get(b"DV").ok().and_then(FieldValue::from_object);

        let flags = field_dict
            .get(b"Ff")
            .ok()
            .and_then(|obj| obj.as_i64().ok())
            .unwrap_or(0) as u32;

        let tooltip = field_dict
            .get(b"TU")
            .ok()
            .and_then(|obj| obj.as_str().ok())
            .map(decode_text_string);

        results.push(FormField {
            name: full_name.clone(),
            field_type: FieldType::from_name(&field_type_name),
            current_value,
            default_value,
            flags,
            tooltip,
        });
    }

    // Recursively process children (Kids array)
    if let Ok(kids) = field_dict.get(b"Kids").and_then(|obj| obj.as_array()) {
        for kid_obj in kids {
            if let Ok(kid_ref) = kid_obj.as_reference() {
                traverse_field_tree(doc, kid_ref, Some(full_name.clone()), results)?;
            }
        }
    }

    Ok(())
}

/// Encode text as UTF-16BE with BOM
fn encode_text_utf16be(text: &str) -> Vec<u8> {
    let mut bytes = vec![0xFE, 0xFF]; // BOM
    for c in text.encode_utf16() {
        bytes.push((c >> 8) as u8);
        bytes.push((c & 0xFF) as u8);
    }
    bytes
}

/// Decode a text string from PDF (handles UTF-16BE with BOM and Latin-1)
fn decode_text_string(bytes: &[u8]) -> String {
    // Check for UTF-16BE BOM
    if bytes.len() >= 2 && bytes[0] == 0xFE && bytes[1] == 0xFF {
        // UTF-16BE with BOM
        let u16_chars: Vec<u16> = bytes[2..]
            .chunks_exact(2)
            .map(|chunk| u16::from_be_bytes([chunk[0], chunk[1]]))
            .collect();
        String::from_utf16(&u16_chars).unwrap_or_else(|_| String::new())
    } else {
        // Latin-1/PDFDocEncoding
        String::from_utf8_lossy(bytes).to_string()
    }
}
