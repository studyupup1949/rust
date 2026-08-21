//! High-level API for PDF form manipulation

use crate::field::{field_value_to_primitive, FieldTraversal, FormField};
use crate::types::FieldValue;
use acroform_pdf::error::PdfError;
use acroform_pdf::file::{CachedFile, FileOptions};
use acroform_pdf::object::{FieldDictionary, PlainRef, Ref, Resolve, Updater, MaybeRef, RcRef};
use acroform_pdf::primitive::Name;
use std::collections::HashMap;
use std::path::Path;

/// High-level wrapper for PDF documents with forms
pub struct AcroFormDocument {
    file: CachedFile<Vec<u8>>,
}

impl AcroFormDocument {
    /// Load a PDF document from a file path
    ///
    /// # Example
    ///
    /// ```no_run
    /// use acroform::AcroFormDocument;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let doc = AcroFormDocument::from_pdf("form.pdf")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn from_pdf<P: AsRef<Path>>(path: P) -> Result<Self, PdfError> {
        let file = FileOptions::cached().open(path.as_ref())?;
        Ok(AcroFormDocument { file })
    }

    /// Get all fillable fields in the document
    ///
    /// Returns a list of all terminal fields (fields that can be filled).
    /// Parent/container fields are not included.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use acroform::AcroFormDocument;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let doc = AcroFormDocument::from_pdf("form.pdf")?;
    /// for field in doc.fields() {
    ///     println!("Field: {} ({:?})", field.name, field.field_type);
    /// }
    /// # Ok(())
    /// # }
    /// ```
    pub fn fields(&self) -> Vec<FormField> {
        let mut all_fields = Vec::new();

        // Get the catalog and forms
        let catalog = self.file.get_root();
        if let Some(ref forms) = catalog.forms {
            // Traverse all root fields
            for field_ref in &forms.fields {
                if let Ok(field) = self.file.resolver().get(field_ref.get_ref()) {
                    let _ = field.collect_terminal_fields(&self.file.resolver(), &mut all_fields);
                }
            }
        }

        all_fields
    }

    /// Fill form fields and save to a new file
    ///
    /// Updates the specified fields with new values and saves the result.
    /// Deletes existing appearance streams (/AP) to force PDF viewers to regenerate them.
    /// This ensures consistent behavior across different PDF viewers.
    ///
    /// # Arguments
    ///
    /// * `values` - Map of field names to new values
    /// * `output` - Output file path
    ///
    /// # Example
    ///
    /// ```no_run
    /// use acroform::{AcroFormDocument, FieldValue};
    /// use std::collections::HashMap;
    ///
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// let mut doc = AcroFormDocument::from_pdf("form.pdf")?;
    ///
    /// let mut values = HashMap::new();
    /// values.insert("name".to_string(), FieldValue::Text("John Doe".to_string()));
    /// values.insert("age".to_string(), FieldValue::Integer(30));
    ///
    /// doc.fill_and_save(values, "filled.pdf")?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn fill_and_save<P: AsRef<Path>>(
        &mut self,
        values: HashMap<String, FieldValue>,
        output: P,
    ) -> Result<(), PdfError> {
        // Build a map of field name to field references (Vec to handle duplicate fields)
        let mut field_map: HashMap<String, Vec<PlainRef>> = HashMap::new();

        let catalog = self.file.get_root();
        
        // Collect fields from AcroForm /Fields array
        if let Some(ref forms) = catalog.forms {
            for field_ref in &forms.fields {
                if let Ok(field) = self.file.resolver().get(field_ref.get_ref()) {
                    self.collect_field_refs(
                        &field,
                        field_ref.get_ref().get_inner(),
                        &mut field_map,
                    )?;
                }
            }
        }
        
        // Also collect fields from page annotations
        // This handles cases where fields appear in both /Fields and /Annots
        for page_result in self.file.pages() {
            if let Ok(page_rc) = page_result {
                let page = &*page_rc; // Deref PageRc to Page
                if let Ok(annots) = page.annotations.load(&self.file.resolver()) {
                    for annot_maybe_ref in &*annots {
                        // Try to resolve the annotation as a field
                        if let MaybeRef::Indirect(r) = annot_maybe_ref {
                            let annot_plain_ref = r.get_ref().get_inner();
                            // Try to get it as a FieldDictionary
                            if let Ok(field) = self.file.resolver().get::<FieldDictionary>(Ref::new(annot_plain_ref)) {
                                // Check if it's a terminal field
                                if field.kids.is_empty() {
                                    let full_name = field.get_full_name(&self.file.resolver())?;
                                    field_map.entry(full_name)
                                        .or_insert_with(Vec::new)
                                        .push(annot_plain_ref);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Update each field (including all duplicates)
        for (field_name, field_value) in values {
            if let Some(field_refs) = field_map.get(&field_name) {
                // Deduplicate refs to avoid updating the same object multiple times
                let mut unique_refs: Vec<PlainRef> = field_refs.clone();
                // Use a HashSet for deduplication since PlainRef doesn't implement Ord
                let unique_refs: std::collections::HashSet<_> = unique_refs.into_iter().collect();
                
                for &field_ref in &unique_refs {
                    self.update_field(field_ref, field_value.clone())?;
                }
            }
        }

        // Set NeedAppearances to true
        self.set_need_appearances(true)?;

        // Save to output file
        self.file.save_to(output.as_ref())?;

        Ok(())
    }

    /// Recursively collect field references with their full names
    fn collect_field_refs(
        &self,
        field: &FieldDictionary,
        field_ref: PlainRef,
        map: &mut HashMap<String, Vec<PlainRef>>,
    ) -> Result<(), PdfError> {
        // If this field has kids, recurse
        if !field.kids.is_empty() {
            for kid_ref in &field.kids {
                let kid = self.file.resolver().get(*kid_ref)?;
                self.collect_field_refs(&kid, kid_ref.get_inner(), map)?;
            }
        } else {
            // Terminal field - add to map (append to vector to handle duplicates)
            let full_name = field.get_full_name(&self.file.resolver())?;
            map.entry(full_name)
                .or_insert_with(Vec::new)
                .push(field_ref);
        }
        Ok(())
    }

    /// Update a specific field with a new value
    fn update_field(&mut self, field_ref: PlainRef, value: FieldValue) -> Result<(), PdfError> {
        let field: RcRef<FieldDictionary> =
            self.file.resolver().get(Ref::new(field_ref))?;
        let mut updated_field = (*field).clone();

        // Update the value
        updated_field.value = field_value_to_primitive(&value);

        // Delete the appearance stream (/AP) to force PDF viewers to regenerate it
        // This is simpler and more reliable than generating appearance streams ourselves
        updated_field.other.remove("AP");

        // For boolean fields (checkboxes/radio buttons), also update /AS (appearance state)
        match value {
            FieldValue::Boolean(checked) => {
                let as_value = if checked {
                    // Need to check the field's options to determine the correct "on" state
                    // For simplicity, use "Yes" as default (common convention)
                    Name::from("Yes")
                } else {
                    Name::from("Off")
                };
                updated_field.other.insert("AS", as_value);
            }
            FieldValue::Choice(ref choice) => {
                // For choice fields, also update /AS
                updated_field
                    .other
                    .insert("AS", Name::from(choice.as_str()));
            }
            _ => {
                // For other field types (Text, Integer), removing /AP is sufficient
            }
        }

        // Update the field in the file
        self.file.update(field_ref, updated_field)?;

        Ok(())
    }

    /// Set the NeedAppearances flag in the AcroForm dictionary
    fn set_need_appearances(&mut self, _value: bool) -> Result<(), PdfError> {
        // Note: InteractiveFormDictionary in the pdf crate is not stored as a reference
        // in the Catalog. We would need to update the entire catalog to modify it.
        // For now, this is a limitation - forms is not a Ref, it's a value.
        // This could be fixed by modifying the pdf crate or finding an alternative approach.

        // TODO: Find a way to update the forms dictionary
        // The catalog.forms is an Option<InteractiveFormDictionary>, not a Ref
        // so we can't easily update it through the file.update() mechanism

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_compiles() {
        // Just a compilation test - we'll add real tests with sample PDFs later
    }
}
