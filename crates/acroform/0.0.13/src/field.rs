use pdf::error::PdfError;
use pdf::object::{FieldDictionary, InteractiveFormDictionary, Resolve, RcRef};

/// Extension trait to add traversal functionality to FieldDictionary
///
/// This trait provides additional methods for working with PDF form field dictionaries,
/// including traversing field hierarchies and obtaining fully qualified field names.
pub trait FieldDictionaryExt {
    /// Get the full name of this field by concatenating parent /T values
    ///
    /// PDF forms can have hierarchical field structures. This method walks up the
    /// parent chain and concatenates all field names with dots (e.g., "parent.child.field").
    ///
    /// # Arguments
    ///
    /// * `resolver` - A resolver for looking up indirect PDF objects
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if parent references cannot be resolved.
    fn get_full_name(&self, resolver: &impl Resolve) -> Result<String, PdfError>;
    
    /// Recursively traverse all child fields and return their references
    ///
    /// This method recursively walks through all children of this field dictionary,
    /// collecting references to all terminal (leaf) fields that have a type.
    ///
    /// # Arguments
    ///
    /// * `resolver` - A resolver for looking up indirect PDF objects
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if field references cannot be resolved.
    fn traverse_field_refs(&self, resolver: &impl Resolve) -> Result<Vec<RcRef<FieldDictionary>>, PdfError>;
}

impl FieldDictionaryExt for FieldDictionary {
    fn get_full_name(&self, resolver: &impl Resolve) -> Result<String, PdfError> {
        let mut parts = Vec::new();
        
        // Add current field name if it exists
        if let Some(ref name) = self.name {
            parts.push(name.to_string_lossy().to_string());
        }
        
        // Walk up the parent chain by collecting all parent refs first
        let mut parent_refs = Vec::new();
        let mut current_parent = self.parent;
        while let Some(parent_ref) = current_parent {
            parent_refs.push(parent_ref);
            let parent: RcRef<FieldDictionary> = resolver.get(parent_ref)?;
            current_parent = parent.parent;
        }
        
        // Now walk the parent refs in reverse to build the name
        for parent_ref in parent_refs.iter().rev() {
            let parent: RcRef<FieldDictionary> = resolver.get(*parent_ref)?;
            if let Some(ref name) = parent.name {
                parts.insert(0, name.to_string_lossy().to_string());
            }
        }
        
        Ok(parts.join("."))
    }
    
    fn traverse_field_refs(&self, resolver: &impl Resolve) -> Result<Vec<RcRef<FieldDictionary>>, PdfError> {
        let mut result = Vec::new();
        
        // Recursively traverse children
        for kid_ref in &self.kids {
            let kid: RcRef<FieldDictionary> = resolver.get(*kid_ref)?;
            
            // If this kid has a type, it's a terminal field
            if kid.typ.is_some() {
                result.push(kid.clone());
            }
            
            // Recursively process grandchildren
            result.extend(kid.traverse_field_refs(resolver)?);
        }
        
        Ok(result)
    }
}

/// Extension trait to add traversal functionality to InteractiveFormDictionary
///
/// This trait provides methods for working with the PDF's AcroForm (interactive form)
/// dictionary, including listing all fields and finding fields by name.
pub trait InteractiveFormDictionaryExt {
    /// Get all terminal fields in the form (flattened)
    ///
    /// Returns a flat list of all terminal (leaf) fields in the form, regardless of
    /// their position in the field hierarchy. Terminal fields are those that have a
    /// field type and can be filled.
    ///
    /// # Arguments
    ///
    /// * `resolver` - A resolver for looking up indirect PDF objects
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if field references cannot be resolved.
    fn all_fields(&self, resolver: &impl Resolve) -> Result<Vec<RcRef<FieldDictionary>>, PdfError>;
    
    /// Find a field by its full name
    ///
    /// Searches through all fields in the form and returns the field with the
    /// matching fully qualified name (e.g., "parent.child.field").
    ///
    /// # Arguments
    ///
    /// * `name` - The fully qualified name of the field to find
    /// * `resolver` - A resolver for looking up indirect PDF objects
    ///
    /// # Returns
    ///
    /// Returns `Ok(Some(field))` if found, `Ok(None)` if not found.
    ///
    /// # Errors
    ///
    /// Returns `PdfError` if field references cannot be resolved.
    fn find_field_by_name(&self, name: &str, resolver: &impl Resolve) -> Result<Option<RcRef<FieldDictionary>>, PdfError>;
}

impl InteractiveFormDictionaryExt for InteractiveFormDictionary {
    fn all_fields(&self, resolver: &impl Resolve) -> Result<Vec<RcRef<FieldDictionary>>, PdfError> {
        let mut result = Vec::new();
        
        for field_ref in &self.fields {
            let field: RcRef<FieldDictionary> = resolver.get(field_ref.get_ref())?;
            
            // If this field has a type, it's a terminal field itself
            if field.typ.is_some() {
                result.push(field.clone());
            }
            
            // Also check its children
            result.extend(field.traverse_field_refs(resolver)?);
        }
        
        Ok(result)
    }
    
    fn find_field_by_name(&self, name: &str, resolver: &impl Resolve) -> Result<Option<RcRef<FieldDictionary>>, PdfError> {
        let all = self.all_fields(resolver)?;
        
        for field in all {
            let field_name = field.get_full_name(resolver)?;
            if field_name == name {
                return Ok(Some(field));
            }
        }
        
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_field_name() {
        // Basic test - we'll add more comprehensive tests with actual PDFs
    }
}
