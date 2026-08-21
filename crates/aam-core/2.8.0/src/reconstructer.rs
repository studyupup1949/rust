use crate::aam::AAM;
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq)]
pub enum AamType {
    String,
    I32,
    F64,
    Bool,
    Color,
    List(Box<AamType>),
    Object(AamSchema),
    Custom(String),
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct AamSchema {
    pub fields: HashMap<String, SchemaField>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct SchemaField {
    pub name: String,
    pub ty: AamType,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Scalar(String),
    Object(HashMap<String, Value>),
    List(Vec<Value>),
}

fn clean_quotes(val: &str) -> &str {
    let trimmed = val.trim();
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        if trimmed.len() >= 2 {
            &trimmed[1..trimmed.len() - 1]
        } else {
            trimmed
        }
    } else {
        trimmed
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;
    for c in s.chars() {
        if c == '_' || c == '-' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }
    result
}

fn split_by_comma(input: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut brace_count = 0;
    let mut bracket_count = 0;
    let mut in_quote = None;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        if c == '\\' {
            current.push(c);
            if let Some(next_c) = chars.next() {
                current.push(next_c);
            }
            continue;
        }

        if let Some(q) = in_quote {
            if c == q {
                in_quote = None;
            }
            current.push(c);
        } else {
            match c {
                '\'' | '"' => {
                    in_quote = Some(c);
                    current.push(c);
                }
                '{' => {
                    brace_count += 1;
                    current.push(c);
                }
                '}' => {
                    brace_count -= 1;
                    current.push(c);
                }
                '[' => {
                    bracket_count += 1;
                    current.push(c);
                }
                ']' => {
                    bracket_count -= 1;
                    current.push(c);
                }
                ',' => {
                    if brace_count == 0 && bracket_count == 0 {
                        parts.push(current.trim().to_string());
                        current.clear();
                    } else {
                        current.push(c);
                    }
                }
                _ => {
                    current.push(c);
                }
            }
        }
    }
    if !current.trim().is_empty() {
        parts.push(current.trim().to_string());
    }
    parts
}

fn parse_value(input: &str) -> Value {
    let input = input.trim();
    if input.starts_with('{') && input.ends_with('}') {
        let content = &input[1..input.len() - 1];
        let mut kvs = HashMap::new();
        let parts = split_by_comma(content);
        for part in parts {
            if let Some(pos) = part.find('=') {
                let key = part[..pos].trim().to_string();
                let val_str = part[pos + 1..].trim();
                if !key.is_empty() {
                    insert_hierarchical(&mut kvs, &key, parse_value(val_str));
                }
            }
        }
        Value::Object(kvs)
    } else if input.starts_with('[') && input.ends_with(']') {
        let content = &input[1..input.len() - 1];
        let parts = split_by_comma(content);
        let mut list_vals = Vec::new();
        for part in parts {
            list_vals.push(parse_value(part.trim()));
        }
        Value::List(list_vals)
    } else {
        Value::Scalar(input.to_string())
    }
}

fn insert_hierarchical(fields: &mut HashMap<String, Value>, key_path: &str, value: Value) {
    let segments: Vec<&str> = key_path.split('.').collect();
    if segments.is_empty() {
        return;
    }
    insert_hierarchical_rec(fields, &segments, value);
}

fn insert_hierarchical_rec(fields: &mut HashMap<String, Value>, segments: &[&str], value: Value) {
    if segments.len() == 1 {
        fields.insert(segments[0].to_string(), value);
        return;
    }

    let current = segments[0].to_string();
    let entry = fields
        .entry(current)
        .or_insert_with(|| Value::Object(HashMap::new()));

    if let Value::Object(sub_map) = entry {
        insert_hierarchical_rec(sub_map, &segments[1..], value);
    } else {
        let mut sub_map = HashMap::new();
        insert_hierarchical_rec(&mut sub_map, &segments[1..], value);
        *entry = Value::Object(sub_map);
    }
}

fn infer_type_for_scalar(val: &str) -> AamType {
    let trimmed = val.trim();
    if trimmed.is_empty() {
        return AamType::Unknown;
    }
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        return AamType::String;
    }
    if trimmed.starts_with('#') {
        return AamType::Color;
    }
    if trimmed == "true" || trimmed == "false" || trimmed == "yes" || trimmed == "no" {
        return AamType::Bool;
    }
    if trimmed.parse::<i32>().is_ok() {
        return AamType::I32;
    }
    if trimmed.parse::<f64>().is_ok() {
        return AamType::F64;
    }
    AamType::String
}

fn infer_type_of_value(val: &Value) -> AamType {
    match val {
        Value::Scalar(s) => infer_type_for_scalar(s),
        Value::List(items) => {
            if items.is_empty() {
                AamType::List(Box::new(AamType::Unknown))
            } else {
                let element_type = infer_type_of_value(&items[0]);
                AamType::List(Box::new(element_type))
            }
        }
        Value::Object(obj) => {
            let mut fields = HashMap::new();
            for (key, sub_val) in obj {
                let ty = infer_type_of_value(sub_val);
                fields.insert(
                    key.clone(),
                    SchemaField {
                        name: key.clone(),
                        ty,
                        optional: false,
                    },
                );
            }
            AamType::Object(AamSchema { fields })
        }
    }
}

fn reconstruct_schema_from_values(instances_fields: &[&HashMap<String, Value>]) -> AamSchema {
    let mut schema = AamSchema::default();
    let mut all_keys = HashSet::new();

    for fields in instances_fields {
        for key in fields.keys() {
            all_keys.insert(key.clone());
        }
    }

    for key in all_keys {
        let mut present_values = Vec::new();
        for fields in instances_fields {
            if let Some(val) = fields.get(&key) {
                present_values.push(val);
            }
        }

        let is_optional = present_values.len() < instances_fields.len();
        let is_object = present_values.iter().any(|v| matches!(v, Value::Object(_)));
        let is_list = present_values.iter().any(|v| matches!(v, Value::List(_)));

        let ty = if is_object {
            let mut sub_maps = Vec::new();
            for val in &present_values {
                if let Value::Object(map) = val {
                    sub_maps.push(map);
                }
            }
            let sub_schema = reconstruct_schema_from_values(&sub_maps);
            AamType::Object(sub_schema)
        } else if is_list {
            let mut element_types: Vec<AamType> = Vec::new();
            for val in &present_values {
                if let Value::List(items) = val {
                    if !items.is_empty() {
                        let elem_ty = infer_type_of_value(&items[0]);
                        if !element_types.contains(&elem_ty) {
                            element_types.push(elem_ty);
                        }
                    }
                }
            }
            let final_elem_type = if element_types.len() == 1 {
                element_types.into_iter().next().unwrap()
            } else {
                AamType::String
            };
            AamType::List(Box::new(final_elem_type))
        } else {
            let mut scalar_types: Vec<AamType> = Vec::new();
            for val in &present_values {
                if let Value::Scalar(s) = val {
                    let inferred = infer_type_for_scalar(s);
                    if !scalar_types.contains(&inferred) {
                        scalar_types.push(inferred);
                    }
                }
            }

            if scalar_types.len() == 1 {
                scalar_types.into_iter().next().unwrap()
            } else if scalar_types.contains(&AamType::F64) && scalar_types.contains(&AamType::I32) {
                AamType::F64
            } else {
                AamType::String
            }
        };

        schema.fields.insert(
            key.clone(),
            SchemaField {
                name: key,
                ty,
                optional: is_optional,
            },
        );
    }

    schema
}

pub fn reconstruct_from_aam_instances(instances: &[AAM]) -> AamSchema {
    let mut structured_instances = Vec::new();

    for inst in instances {
        let mut structured = HashMap::new();
        for (k, v) in inst.iter() {
            insert_hierarchical(&mut structured, k, parse_value(v));
        }
        structured_instances.push(structured);
    }

    let refs: Vec<&HashMap<String, Value>> = structured_instances.iter().collect();
    reconstruct_schema_from_values(&refs)
}

pub fn extract_sub_schemas(
    _parent_name: &str,
    schema: &mut AamSchema,
    extracted: &mut HashMap<String, AamSchema>,
) {
    for (field_name, field) in schema.fields.iter_mut() {
        let field_ty = &mut field.ty;
        match field_ty {
            AamType::Object(sub_schema) => {
                let sub_schema_name = to_pascal_case(field_name);

                extract_sub_schemas(&sub_schema_name, sub_schema, extracted);

                extracted.insert(sub_schema_name.clone(), sub_schema.clone());

                *field_ty = AamType::Custom(sub_schema_name);
            }
            AamType::List(inner_ty) => {
                if let AamType::Object(sub_schema) = &mut **inner_ty {
                    let sub_schema_name = to_pascal_case(field_name);
                    extract_sub_schemas(&sub_schema_name, sub_schema, extracted);
                    extracted.insert(sub_schema_name.clone(), sub_schema.clone());
                    **inner_ty = AamType::Custom(sub_schema_name);
                }
            }
            _ => {}
        }
    }
}

pub fn format_type(ty: &AamType, indent: usize) -> String {
    match ty {
        AamType::String => "string".to_string(),
        AamType::I32 => "i32".to_string(),
        AamType::F64 => "f64".to_string(),
        AamType::Bool => "bool".to_string(),
        AamType::Color => "color".to_string(),
        AamType::Custom(name) => name.clone(),
        AamType::Unknown => "string".to_string(),
        AamType::List(inner) => format!("list<{}>", format_type(inner, indent)),
        AamType::Object(schema) => {
            let mut fields_str = Vec::new();
            let spaces = " ".repeat(indent + 4);
            let closing_spaces = " ".repeat(indent);

            let mut keys: Vec<&String> = schema.fields.keys().collect();
            keys.sort();

            for key in keys {
                let field = &schema.fields[key];
                let opt_sign = if field.optional { "*" } else { "" };
                let field_ty = format_type(&field.ty, indent + 4);
                fields_str.push(format!(
                    "{}{}{}: {}",
                    spaces, field.name, opt_sign, field_ty
                ));
            }

            format!("{{\n{}\n{}}}", fields_str.join("\n"), closing_spaces)
        }
    }
}

pub fn format_schema(name: &str, schema: &AamSchema) -> String {
    let mut fields_str = Vec::new();
    let mut keys: Vec<&String> = schema.fields.keys().collect();
    keys.sort();

    for key in keys {
        let field = &schema.fields[key];
        let opt_sign = if field.optional { "*" } else { "" };
        let field_ty = format_type(&field.ty, 4);
        fields_str.push(format!("    {}{}: {}", field.name, opt_sign, field_ty));
    }

    format!("@schema {} {{\n{}\n}}", name, fields_str.join("\n"))
}

pub fn format_all_schemas(main_name: &str, main_schema: &AamSchema) -> String {
    let mut extracted = HashMap::new();
    let mut main_schema_clone = main_schema.clone();

    extract_sub_schemas(main_name, &mut main_schema_clone, &mut extracted);

    let mut output_parts = Vec::new();

    let mut schema_names: Vec<&String> = extracted.keys().collect();
    schema_names.sort();

    for name in schema_names {
        let sub_schema = &extracted[name];
        output_parts.push(format_schema(name, sub_schema));
    }

    output_parts.push(format_schema(main_name, &main_schema_clone));

    output_parts.join("\n\n")
}

pub fn reconstruct_schema(schema_name: &str, file_sources: &[&str]) -> Result<String, String> {
    let mut instances = Vec::new();

    for src in file_sources {
        let aam = AAM::parse(src).map_err(|e| format!("Failed to parse configuration: {:?}", e))?;
        instances.push(aam);
    }

    let reconstructed = reconstruct_from_aam_instances(&instances);
    Ok(format_all_schemas(schema_name, &reconstructed))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::write;

    #[test]
    fn test_reconstruct_schema_helper() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("base.aam");
        let child1 = dir.path().join("child1.aam");
        let child2 = dir.path().join("child2.aam");

        write(
            &base,
            r#"@schema Player {
    name: string
}
"#,
        )
        .unwrap();

        write(
            &child1,
            r#"@derive base.aam::Player
name = "Luna"
port = 8080
info = { level = 10 }
"#,
        )
        .unwrap();

        write(
            &child2,
            r#"@derive base.aam::Player
name = "Sol"
info = { level = 15, class = "Warrior" }
"#,
        )
        .unwrap();

        let aam1 = AAM::load(&child1).unwrap();
        let aam2 = AAM::load(&child2).unwrap();

        let reconstructed = reconstruct_from_aam_instances(&[aam1, aam2]);
        let result = format_all_schemas("Player", &reconstructed);

        assert!(result.contains("@schema Player {"));
        assert!(result.contains("name: string"));
        assert!(result.contains("port*: i32"));
        assert!(result.contains("info: Info"));
        assert!(result.contains("@schema Info {"));
        assert!(result.contains("level: i32"));
        assert!(result.contains("class*: string"));
    }
}
