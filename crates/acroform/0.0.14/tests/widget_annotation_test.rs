use acroform::{AcroFormDocument, FieldValue};
use pdf::file::FileOptions;
use pdf::object::Resolve;
use std::collections::HashMap;

#[test]
fn test_widget_annotations_updated() {
    // This test ensures that when we update a field value,
    // both the field widget in AcroForm AND the annotation widgets
    // on pages are updated.
    
    let mut doc = AcroFormDocument::from_pdf("../acroform_files/af8.pdf")
        .expect("Failed to load PDF");
    
    // Update the field
    let mut values = HashMap::new();
    values.insert(
        "topmostSubform[0].Page1[0].P[0].MbrName[1]".to_string(),
        FieldValue::Text("NEW_VALUE".to_string()),
    );
    
    doc.fill_and_save(values, "/tmp/test_widget_update.pdf")
        .expect("Failed to save PDF");
    
    // Now inspect the saved PDF to verify both field and annotation are updated
    let file = FileOptions::cached().open("/tmp/test_widget_update.pdf")
        .expect("Failed to reopen PDF");
    let resolver = file.resolver();
    
    // Check the field value in AcroForm
    if let Some(ref forms) = file.get_root().forms {
        for field_ref in &forms.fields {
            let field = resolver.get(field_ref.get_ref())
                .expect("Failed to get field");
            if let Some(ref name) = field.name {
                if name.to_string_lossy() == "topmostSubform[0].Page1[0].P[0].MbrName[1]" {
                    // Check the field value
                    if let pdf::primitive::Primitive::String(ref s) = field.value {
                        assert_eq!(s.to_string_lossy(), "NEW_VALUE", 
                                   "Field value should be updated");
                    } else {
                        panic!("Field value should be a string");
                    }
                }
            }
        }
    }
    
    // Check the annotation value on the page
    for page_rc in file.pages() {
        let page = page_rc.expect("Failed to get page");
        let annots = page.annotations.load(&resolver)
            .expect("Failed to load annotations");
        
        for annot_ref in annots.data().iter() {
            let annot = annot_ref.data();
            
            // Check if this is the MbrName annotation
            if let Some(ref field_name) = annot.other.get("T") {
                if let pdf::primitive::Primitive::String(ref name_str) = field_name {
                    if name_str.to_string_lossy() == "topmostSubform[0].Page1[0].P[0].MbrName[1]" {
                        // Check the annotation value
                        if let Some(ref value) = annot.other.get("V") {
                            if let pdf::primitive::Primitive::String(ref v) = value {
                                assert_eq!(v.to_string_lossy(), "NEW_VALUE",
                                           "Annotation widget value should also be updated");
                            } else {
                                panic!("Annotation value should be a string");
                            }
                        } else {
                            panic!("Annotation should have a value");
                        }
                    }
                }
            }
        }
    }
}
