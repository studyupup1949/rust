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
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
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
    fn name(&self) -> &str {
        "derive"
    }

    fn execute(&self, aaml: &mut AAML, args: &str) -> Result<(), AamlError> {
        let (path, selectors) = parse_derive_arg(args.trim());
        let mut base = AAML::load(path)?;

        let original_schemas: Vec<String> = aaml.get_schemas().keys().cloned().collect();

        if selectors.is_empty() {
            let names: Vec<String> = base.get_schemas().keys().cloned().collect();
            for name in names {
                aaml.import_schema(&name, &mut base)?;
            }
        } else {
            for name in selectors {
                aaml.import_schema(name, &mut base)?;
            }
        }

        aaml.merge_map_weak(base.get_map_mut());

        let refs: Vec<&str> = original_schemas.iter().map(|s| s.as_str()).collect();
        aaml.validate_schemas_completeness_for(&refs)?;

        Ok(())
    }
}
