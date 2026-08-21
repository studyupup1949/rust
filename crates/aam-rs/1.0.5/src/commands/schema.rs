//! `@schema` directive — defines a named struct-like schema with typed fields.
//!
//! # Syntax
//! ```text
//! @schema Name { field1: type1, field2: type2, ... }
//! ```
//!
//! # Semantics
//! After a schema is registered any `key = value` assignment whose key matches
//! a schema field is automatically validated against the declared type.
//! Use [`AAML::apply_schema`] to validate a complete data map programmatically.

use std::collections::HashMap;
use crate::aaml::AAML;
use crate::commands::Command;
use crate::error::AamlError;

/// A parsed schema definition: maps field names to their declared type strings.
///
/// Type strings can be primitives (`i32`, `f64`, `string`, `bool`, `color`),
/// built-in module paths (`math::vector3`, `physics::kilogram`, `time::datetime`),
/// or custom aliases registered via `@type`.
#[derive(Clone, Debug)]
pub struct SchemaDef {
    /// Map of `field_name → type_name`.
    pub fields: HashMap<String, String>,
}

/// Command handler for the `@schema` directive.
pub struct SchemaCommand;

impl SchemaCommand {
    /// Parses the raw argument string into a `(name, SchemaDef)` pair.
    ///
    /// Expected format: `Name { field: type, ... }`
    fn parse(args: &str) -> Result<(String, SchemaDef), AamlError> {
        let args = args.trim();
        let (name_part, body_part) = args.split_once('{')
            .ok_or_else(|| AamlError::DirectiveError("schema".into(), "Expected '{'".into()))?;

        let name = name_part.trim();
        if name.is_empty() {
            return Err(AamlError::DirectiveError("schema".into(), "Schema name is empty".into()));
        }

        let body = body_part.rsplit_once('}')
            .ok_or_else(|| AamlError::DirectiveError("schema".into(), "Expected '}'".into()))?
            .0;

        let mut fields = HashMap::new();
        // Normalize: commas and whitespace are both valid field separators.
        // Replace commas with spaces so we can use split_whitespace uniformly.
        let normalized = body.replace(',', " ");
        let mut tokens = normalized.split_whitespace();
        while let Some(token) = tokens.next() {
            let (field, ty) = if let Some((f, t)) = token.split_once(':') {
                // "field:type" or "field:" — type may follow as next token
                let ty = if t.is_empty() {
                    tokens.next().ok_or_else(|| {
                        AamlError::DirectiveError("schema".into(), format!("Bad field: '{f}:' has no type"))
                    })?
                } else {
                    t
                };
                (f, ty)
            } else {
                return Err(AamlError::DirectiveError("schema".into(), format!("Bad field: '{token}'")));
            };
            if field.is_empty() || ty.is_empty() {
                return Err(AamlError::DirectiveError("schema".into(), format!("Bad field: '{field}: {ty}'")));
            }
            fields.insert(field.to_string(), ty.to_string());
        }

        Ok((name.to_string(), SchemaDef { fields }))
    }
}

impl Command for SchemaCommand {
    fn name(&self) -> &str { "schema" }

    /// Parses the schema definition and registers it in the current [`AAML`] instance.
    ///
    /// If a schema with the same name already exists it is **replaced**.
    fn execute(&self, aaml: &mut AAML, args: &str) -> Result<(), AamlError> {
        let (name, schema) = Self::parse(args)?;
        aaml.get_schemas_mut().insert(name, schema);
        Ok(())
    }
}
