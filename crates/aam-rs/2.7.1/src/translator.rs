use crate::builder::{AAMBuilder, SchemaField};
use std::error::Error;

pub struct TOMLTranslator;

impl TOMLTranslator {
    /// Translate TOML source string into a vector of `AAMBuilder` modules.
    /// Each module is a new .aam-file
    ///
    /// # Errors
    ///
    /// Returns an error if the TOML source is invalid.
    ///
    /// # Panics
    ///
    /// This function should not panic, but it was previously using `expect`.
    pub fn toml_to_aam(toml_source: &str) -> Result<Vec<AAMBuilder>, Box<dyn Error>> {
        let root: toml::Value = toml::from_str(toml_source.trim())?;

        let mut modules: Vec<AAMBuilder> = Vec::new();

        let mut root_builder = AAMBuilder::new();
        root_builder.comment("Generated from TOML");

        if let toml::Value::Table(table) = &root {
            let mut keys: Vec<&String> = table.keys().collect();
            keys.sort();

            for key in keys {
                let value = &table[key];

                if let toml::Value::Table(inner) = value {
                    let module_name = format!("{key}.aam");
                    root_builder.import(&module_name);

                    let mut mod_builder = AAMBuilder::new();
                    mod_builder.comment(&format!("Module generated from TOML table [{key}]"));

                    let mut fields = Vec::new();
                    for (field_key, field_value) in inner {
                        let type_name = Self::toml_type_to_aam_type(field_value);
                        fields.push(SchemaField::required(field_key, type_name));
                    }
                    if !fields.is_empty() {
                        mod_builder.schema(key, fields);
                    }

                    for (field_key, field_value) in inner {
                        let serialized = Self::toml_value_to_aam_string(field_value);
                        mod_builder.add_line(field_key, &serialized);
                    }

                    modules.push(mod_builder);
                } else {
                    let serialized = Self::toml_value_to_aam_string(value);
                    root_builder.add_line(key, &serialized);
                }
            }
        }

        modules.insert(0, root_builder);

        Ok(modules)
    }

    fn toml_type_to_aam_type(value: &toml::Value) -> &str {
        match value {
            toml::Value::String(_) | toml::Value::Datetime(_) => "string",
            toml::Value::Integer(_) => "i32",
            toml::Value::Float(_) => "f64",
            toml::Value::Boolean(_) => "bool",
            toml::Value::Array(arr) => {
                if arr.is_empty() {
                    "array"
                } else {
                    Self::toml_type_to_aam_type(&arr[0])
                }
            }
            toml::Value::Table(_) => "object",
        }
    }

    /// Translate TOML value to AAM String
    fn toml_value_to_aam_string(value: &toml::Value) -> String {
        match value {
            toml::Value::String(s) => s.clone(),
            other => other.to_string(),
        }
    }
}
