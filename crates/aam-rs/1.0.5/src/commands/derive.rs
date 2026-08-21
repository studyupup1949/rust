//! `@derive` directive — inherits keys and schemas from another `.aam` file.
//!
//! # Syntax
//! ```text
//! @derive path/to/base.aam
//! @derive "path/to/base.aam"
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

use crate::aaml::AAML;
use crate::commands::Command;
use crate::error::AamlError;

/// Command handler for the `@derive` directive.
pub struct DeriveCommand;

impl Command for DeriveCommand {
    fn name(&self) -> &str { "derive" }

    /// Loads the base file, merges its schemas and key-value pairs into `aaml`
    /// (child entries win on conflict), then verifies that every field declared
    /// in any active schema is present in the final map.
    ///
    /// # Errors
    /// - [`AamlError::DirectiveError`] — path argument is missing.
    /// - [`AamlError::IoError`] — base file cannot be read.
    /// - Any parse error from the base file.
    /// - [`AamlError::SchemaValidationError`] — after the merge a required
    ///   schema field has no value assigned.
    fn execute(&self, aaml: &mut AAML, args: &str) -> Result<(), AamlError> {
        let raw_path = args.trim();
        if raw_path.is_empty() {
            return Err(AamlError::DirectiveError(
                "derive".into(),
                "Missing file path".into(),
            ));
        }

        let path = AAML::unwrap_quotes(raw_path);
        let mut base = AAML::load(path)?;

        // Merge schemas — child wins on name conflict
        for (schema_name, schema) in base.get_schemas_mut().drain() {
            aaml.get_schemas_mut().entry(schema_name).or_insert(schema);
        }

        // Merge key-value pairs — child wins on key conflict
        for (k, v) in base.get_map_mut().drain() {
            aaml.get_map_mut().entry(k).or_insert(v);
        }

        // After the merge every schema that is now in scope must be fully
        // satisfied: all declared fields must have a corresponding value.
        aaml.validate_schemas_completeness()?;

        Ok(())
    }
}
