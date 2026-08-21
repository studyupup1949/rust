//! Type system for AAML value validation.
//!
//! Types are used both directly (via [`AAML::validate_value`]) and indirectly
//! through `@schema` field declarations. The entry point for resolving a type
//! from a string path is [`resolve_builtin`].
//!
//! ## Built-in type paths
//! | Path | Description |
//! |------|-------------|
//! | `i32` / `f64` / `string` / `bool` / `color` | Primitive types |
//! | `math::vector2` … `math::matrix4x4` | N-component float vectors/matrices |
//! | `physics::kilogram` | Non-negative floating-point mass |
//! | `time::datetime` | ISO 8601 date or datetime string |

use crate::error::AamlError;
use crate::types::primitive_type::PrimitiveType;

pub(crate) mod physics;
pub(crate) mod primitive_type;
pub(crate) mod list;
mod math;
mod time;

/// Core trait that every AAML type must implement.
pub trait Type {
    /// Constructs the type from a name string.
    ///
    /// Used internally by [`resolve_builtin`] to create type instances from
    /// the sub-name after the `::` separator.
    fn from_name(name: &str) -> Result<Self, AamlError> where Self: Sized;

    /// Returns the primitive type that best represents this type.
    ///
    /// Used as a hint for serialization or schema introspection.
    fn base_type(&self) -> PrimitiveType;

    /// Validates `value` against this type's constraints.
    ///
    /// Returns `Ok(())` if the value is acceptable, or an
    /// [`AamlError`] with a human-readable message otherwise.
    fn validate(&self, value: &str) -> Result<(), AamlError>;
}

/// Resolves a type from a module-qualified path or a plain primitive name.
///
/// # Supported paths
/// - `math::<name>` — see [`math::MathTypes`]
/// - `time::<name>` — see [`time::TimeTypes`]
/// - `physics::<name>` — see [`physics::PhysicsTypes`]
/// - `list<T>` — a homogeneous list of elements with type `T`
/// - `<name>` (no `::`) — a [`PrimitiveType`] name
///
/// # Errors
/// [`AamlError::NotFound`] if the path is not recognised.
pub fn resolve_builtin(path: &str) -> Result<Box<dyn Type>, AamlError> {
    // list<T> — must be checked before splitn to avoid confusion
    if let Some(inner) = list::ListType::parse_inner(path) {
        return Ok(Box::new(list::ListType::new(inner)));
    }

    let parts: Vec<&str> = path.splitn(2, "::").collect();

    match parts.as_slice() {
        ["math", name] => Ok(Box::new(math::MathTypes::from_name(name)?)),
        ["time", name] => Ok(Box::new(time::TimeTypes::from_name(name)?)),
        ["physics", name] => Ok(Box::new(physics::PhysicsTypes::from_name(name)?)),
        [name] => Ok(Box::new(primitive_type::PrimitiveType::from_name(name)?)),
        _ => Err(AamlError::NotFound(path.to_string())),
    }
}