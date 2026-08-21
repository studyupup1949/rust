//! `@type` directive — registers a named type alias or built-in type reference.
//!
//! # Syntax
//! ```text
//! @type alias_name = primitive_type
//! @type alias_name = module::type_name
//! ```
//!
//! # Examples
//! ```text
//! @type age     = i32
//! @type ratio   = f64
//! @type pos     = math::vector3
//! @type mass    = physics::kilogram
//! @type created = time::datetime
//! ```
//!
//! After registration the alias can be used as a field type in `@schema`
//! definitions and validated via [`AAML::validate_value`].
//!

use crate::commands::Command;
use crate::error::AamlError;
use crate::types::{resolve_builtin, Type};
use crate::types::primitive_type::PrimitiveType;

/// A resolved type definition stored in the [`AAML`](crate::aaml::AAML) type registry.
///
/// Variants correspond to the three ways a type can be declared:
/// - [`TypeDefinition::Primitive`] — a primitive name such as `i32` or `bool`.
/// - [`TypeDefinition::Builtin`] — a module-qualified path such as `math::vector3`.
/// - [`TypeDefinition::Alias`] — an opaque alias (currently always passes validation).
pub enum TypeDefinition {
    /// A primitive type identified by name (e.g. `"i32"`, `"f64"`).
    Primitive(String),
    /// An alias that doesn't map to a concrete type — always valid.
    Alias(String),
    /// A built-in module path (e.g. `"math::vector3"`).
    Builtin(String),
}

impl Type for TypeDefinition {
    fn from_name(_name: &str) -> Result<Self, AamlError>
    where
        Self: Sized,
    {
        Err(AamlError::NotFound("TypeDefinition::from_name not supported".to_string()))
    }

    /// Returns the underlying [`PrimitiveType`] that best represents this type.
    fn base_type(&self) -> PrimitiveType {
        match self {
            TypeDefinition::Builtin(path) => {
                resolve_builtin(path).map(|t| t.base_type()).unwrap_or(PrimitiveType::String)
            }
            TypeDefinition::Primitive(name) => {
                PrimitiveType::from_name(name).unwrap_or(PrimitiveType::String).base_type()
            }
            TypeDefinition::Alias(_) => PrimitiveType::String,
        }
    }

    /// Validates `value` according to the underlying type definition.
    ///
    /// - `Builtin` — delegates to the corresponding module type.
    /// - `Primitive` — delegates to [`PrimitiveType`].
    /// - `Alias` — always returns `Ok(())`.
    fn validate(&self, value: &str) -> Result<(), AamlError> {
        match self {
            TypeDefinition::Builtin(path) => resolve_builtin(path)?.validate(value),
            TypeDefinition::Primitive(name) => PrimitiveType::from_name(name)?.validate(value),
            TypeDefinition::Alias(_) => Ok(()),
        }
    }
}

/// Command handler for the `@type` directive.
pub struct TypeCommand;

impl Command for TypeCommand {
    fn name(&self) -> &str { "type" }

    /// Parses `name = definition` and registers the resulting [`TypeDefinition`].
    ///
    /// Built-in paths (containing `::`) become [`TypeDefinition::Builtin`];
    /// all other definitions become [`TypeDefinition::Primitive`].
    ///
    /// # Errors
    /// [`AamlError::ParseError`] if the format is invalid or name/definition is empty.
    fn execute(&self, aaml: &mut crate::aaml::AAML, args: &str) -> Result<(), AamlError> {
        let (name, definition) = args.split_once('=').ok_or_else(|| AamlError::ParseError {
            line: 0,
            content: args.to_string(),
            details: "Type definition must be in the format 'name = definition'".to_string(),
        })?;

        let name = name.trim();
        let definition = definition.trim();

        if name.is_empty() {
            return Err(AamlError::ParseError {
                line: 0,
                content: args.to_string(),
                details: "Type name cannot be empty".to_string(),
            });
        }
        if definition.is_empty() {
            return Err(AamlError::ParseError {
                line: 0,
                content: args.to_string(),
                details: "Type definition cannot be empty".to_string(),
            });
        }

        let type_def = if definition.contains("::") {
            TypeDefinition::Builtin(definition.to_string())
        } else {
            TypeDefinition::Primitive(definition.to_string())
        };

        aaml.register_type(name.to_string(), type_def);

        Ok(())
    }
}