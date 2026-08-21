//! Command infrastructure for AAML directives.
//!
//! Each directive (`@import`, `@derive`, `@schema`, `@type`) is implemented as
//! a struct that implements the [`Command`] trait and is registered in
//! [`AAML::register_default_commands`](crate::aaml::AAML).

use crate::aaml::AAML;
use crate::error::AamlError;

pub mod import;
pub mod schema;
pub mod typecm;
pub mod derive;

/// Trait implemented by every AAML directive handler.
///
/// Commands are registered by name and invoked automatically when `@<name> <args>`
/// is encountered during parsing.
pub trait Command: Send + Sync {
    /// The directive name without the leading `@` (e.g. `"import"`, `"schema"`).
    fn name(&self) -> &str;

    /// Executes the directive with the given argument string.
    ///
    /// `args` contains everything after the directive name on the same line,
    /// with leading whitespace preserved.
    fn execute(&self, aaml: &mut AAML, args: &str) -> Result<(), AamlError>;
}