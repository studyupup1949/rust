//! `@derive` directive — inherits keys and schemas from another `.aam` file.
//!
//! # Syntax
//! ```text
//! @derive path/to/base.aam
//! @derive "path/to/base.aam"
//! @derive path/to/base.aam::Schema1
//! @derive path/to/base.aam::Schema1::Schema2
//! ```
//!
//! # Semantics
//! - All key-value pairs from the base file are imported into the current document.
//! - Child values take precedence: existing keys are **never** overwritten.
//! - Schema definitions follow the same rule: a child schema beats a base schema
//!   with the same name.
//! - After the merge, all schemas that are now in scope are checked for
//!   completeness — every declared field must have a value assigned somewhere
//!   in the resulting document. Missing fields produce a
//!   [`AamlError::SchemaValidationError`].
//!   Optional fields (declared with `*`) are ignored during completeness check.

use crate::aaml::AAML;
use crate::commands::Command;
use crate::error::AamlError;

/// Command handler for the `@derive` directive.
pub struct DeriveCommand;

/// Splits a raw `@derive` argument into `(file_path, schema_selectors)`.
///
/// Supported forms:
/// - `base.aam` → `("base.aam", [])`
/// - `base.aam::Foo::Bar` → `("base.aam", ["Foo", "Bar"])`
/// - `"base.aam"::Foo` → `("base.aam", ["Foo"])`
fn parse_derive_arg(raw: &str) -> (&str, Vec<&str>) {
    let (path_raw, rest) = if raw.starts_with('"') || raw.starts_with('\'') {
        let q = raw.chars().next().unwrap();
        match raw[1..].find(q) {
            Some(end) => {
                let path = &raw[1..end + 1];
                let after = raw[end + 2..].trim_start_matches(':').trim();
                (path, after)
            }
            None => (raw, ""),
        }
    } else {
        match raw.find("::") {
            Some(pos) => (&raw[..pos], &raw[pos + 2..]),
            None => (raw, ""),
        }
    };

    let selectors = rest
        .split("::")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .collect();

    (path_raw.trim(), selectors)
}

impl Command for DeriveCommand {
    fn name(&self) -> &str { "derive" }

    /// Loads the base file, merges its schemas and key-value pairs into `aaml`
    /// (child entries win on conflict), then verifies that every required field
    /// declared in any active schema is present in the final map.
    ///
    /// If schema selectors (`::SchemaName`) are provided, only the named
    /// schemas are imported from the base file.
    ///
    /// # Errors
    /// - [`AamlError::DirectiveError`] — path argument is missing or a
    ///   requested schema does not exist in the base file.
    /// - [`AamlError::IoError`] — base file cannot be read.
    /// - Any parse error from the base file.
    /// - [`AamlError::SchemaValidationError`] — after the merge a required
    ///   schema field has no value assigned.
    fn execute(&self, aaml: &mut AAML, args: &str) -> Result<(), AamlError> {
        let raw = args.trim();
        if raw.is_empty() {
            return Err(AamlError::DirectiveError("derive".into(), "Missing file path".into()));
        }

        // Snapshot child-owned schema names BEFORE merging base schemas.
        let child_schema_names: Vec<String> =
            aaml.get_schemas_mut().keys().cloned().collect();

        let (path, selectors) = parse_derive_arg(raw);
        let mut base = AAML::load(path)?;

        if selectors.is_empty() {
            for (name, schema) in base.get_schemas_mut().drain() {
                aaml.get_schemas_mut().entry(name).or_insert(schema);
            }
        } else {
            for selector in &selectors {
                let schema = base.get_schemas_mut().remove(*selector).ok_or_else(|| {
                    AamlError::DirectiveError(
                        "derive".into(),
                        format!("Schema '{selector}' not found in '{path}'"),
                    )
                })?;
                aaml.get_schemas_mut().entry(selector.to_string()).or_insert(schema);
            }
        }

        // Merge key-value pairs — child wins on conflict.
        for (k, v) in base.get_map_mut().drain() {
            aaml.get_map_mut().entry(k).or_insert(v);
        }

        // Validate completeness only for child-owned schemas.
        let names: Vec<&str> = child_schema_names.iter().map(|s| s.as_str()).collect();
        aaml.validate_schemas_completeness_for(&names)?;

        Ok(())
    }
}
