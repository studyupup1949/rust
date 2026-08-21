//! `@import` directive — merges another `.aam` file into the current document.
//!
//! # Syntax
//! ```text
//! @import path/to/file.aam
//! @import "path/to/file.aam"
//! ```
//!
//! # Semantics
//! Unlike `@derive`, `@import` uses `merge_content` which means **later** values
//! overwrite earlier ones. If the same key appears in both the current document
//! and the imported file, the imported value **wins** (last-write semantics).

use crate::aaml::AAML;
use crate::commands::Command;
use crate::error::AamlError;

/// Command handler for the `@import` directive.
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportCommand;

impl Command for ImportCommand {
    fn name(&self) -> &str {
        "import"
    }

    /// Reads the file at the given path and merges its content into `aaml`.
    ///
    /// # Errors
    /// - [`AamlError::ParseError`] — path argument is empty.
    /// - [`AamlError::IoError`] — file cannot be read.
    /// - Any parse error from the imported file.
    fn execute(&self, aaml: &mut AAML, args: &str) -> Result<(), AamlError> {
        let raw_path = args.trim();
        if raw_path.is_empty() {
            return Err(AamlError::ParseError {
                line: 0,
                content: args.to_string(),
                details: "Import path cannot be empty".to_string(),
            });
        }

        let path = AAML::unwrap_quotes(raw_path);
        aaml.merge_file(path)
    }
}
