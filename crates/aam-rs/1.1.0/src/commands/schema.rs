//! `@schema` directive — defines a named struct-like schema with typed fields.
//!
//! # Syntax
//! ```text
//! @schema Name { field1: type1, field2*: type2, ... }
//! ```
//!
//! A field name ending with `*` is **optional** — it is not required to be present
//! in the data map, but if it *is* present the value must satisfy the declared type.
//!
//! # Semantics
//! After a schema is registered any `key = value` assignment whose key matches
//! a schema field is automatically validated against the declared type.
//! Use [`AAML::apply_schema`] to validate a complete data map programmatically.

use std::collections::HashSet;
use std::collections::HashMap;
use crate::aaml::AAML;
use crate::commands::Command;
use crate::error::AamlError;

/// A parsed schema definition: maps field names to their declared type strings.
///
/// Type strings can be primitives (`i32`, `f64`, `string`, `bool`, `color`),
/// built-in module paths (`math::vector3`, `physics::kilogram`, `time::datetime`),
/// or custom aliases registered via `@type`.
///
/// Fields listed in `optional_fields` do not have to be present in the data map,
/// but if they *are* present their values are still validated.
#[derive(Clone, Debug)]
pub struct SchemaDef {
    /// Map of `field_name → type_name`.
    pub fields: HashMap<String, String>,
    /// Set of field names that are optional (declared with `*` suffix).
    pub optional_fields: HashSet<String>,
}

impl SchemaDef {
    /// Returns `true` when `field` was declared with `*` (optional).
    pub fn is_optional(&self, field: &str) -> bool {
        self.optional_fields.contains(field)
    }
}

/// Command handler for the `@schema` directive.
pub struct SchemaCommand;

impl SchemaCommand {
    /// Splits `args` into the schema name and the raw body between `{` and `}`.
    fn parse_header(args: &str) -> Result<(&str, &str), AamlError> {
        let (name_part, body_part) = args
            .split_once('{')
            .ok_or_else(|| AamlError::DirectiveError("schema".into(), "Expected '{'".into()))?;

        let name = name_part.trim();
        if name.is_empty() {
            return Err(AamlError::DirectiveError("schema".into(), "Schema name is empty".into()));
        }

        let body = body_part
            .rsplit_once('}')
            .ok_or_else(|| AamlError::DirectiveError("schema".into(), "Expected '}'".into()))?
            .0;

        Ok((name, body))
    }

    /// Parses a single `field:type` or `field*:type` token pair.
    ///
    /// Returns `(field_name, type_name, is_optional)`.
    /// A field name ending with `*` is optional — the `*` is stripped from
    /// the stored name and `is_optional` is set to `true`.
    fn parse_field<'a>(
        token: &'a str,
        tokens: &mut impl Iterator<Item = &'a str>,
    ) -> Result<(String, String, bool), AamlError> {
        let (field_raw, ty) = token
            .split_once(':')
            .ok_or_else(|| AamlError::DirectiveError("schema".into(), format!("Bad field: '{token}'")))?;

        // "field:type" or "field:" — type may follow as the next token.
        let ty = if ty.is_empty() {
            tokens.next().ok_or_else(|| {
                AamlError::DirectiveError("schema".into(), format!("Bad field: '{field_raw}:' has no type"))
            })?
        } else {
            ty
        };

        let is_optional = field_raw.ends_with('*');
        let field = if is_optional {
            field_raw.trim_end_matches('*')
        } else {
            field_raw
        };

        if field.is_empty() || ty.is_empty() {
            return Err(AamlError::DirectiveError(
                "schema".into(),
                format!("Bad field: '{field}: {ty}'"),
            ));
        }

        Ok((field.to_string(), ty.to_string(), is_optional))
    }

    /// Parses the raw argument string into a `(name, SchemaDef)` pair.
    ///
    /// Expected format: `Name { field: type, field*: type, ... }`
    fn parse(args: &str) -> Result<(String, SchemaDef), AamlError> {
        let (name, body) = Self::parse_header(args.trim())?;

        // Normalize: commas and whitespace are both valid field separators.
        // Replace commas with spaces so we can use split_whitespace uniformly.
        let normalized = body.replace(',', " ");
        let mut tokens = normalized.split_whitespace();
        let mut fields = HashMap::new();
        let mut optional_fields = HashSet::new();

        while let Some(token) = tokens.next() {
            let (field, ty, is_optional) = Self::parse_field(token, &mut tokens)?;
            if is_optional {
                optional_fields.insert(field.clone());
            }
            fields.insert(field, ty);
        }

        Ok((name.to_string(), SchemaDef { fields, optional_fields }))
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
