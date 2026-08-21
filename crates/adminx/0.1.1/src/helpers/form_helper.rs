use convert_case::{Case, Casing};
use serde_json::{Value, json};

/// Converts the dynamic form_structure JSON into a flat list of fields with name/label
pub fn extract_fields_for_form(form_structure: &Value) -> Vec<serde_json::Value> {
    let mut fields = vec![];

    if let Some(groups) = form_structure.get("groups").and_then(|g| g.as_array()) {
        for group in groups {
            if let Some(group_fields) = group.get("fields").and_then(|f| f.as_array()) {
                for field in group_fields {
                    if let Some(name) = field.as_str() {
                        fields.push(json!({
                            "name": name,
                            "label": name.to_case(Case::Title), // Requires convert_case crate
                        }));
                    }
                }
            }
        }
    }

    fields
}
