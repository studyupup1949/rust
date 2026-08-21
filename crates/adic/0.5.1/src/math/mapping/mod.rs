//! Mapping: A general kind of function
//!
//! - [`ComposedMapping`] - Two `Mapping`s composed in series
//! - [`Differentiable`] - Has a derivative
//! - [`Mapping`]/[`IndexedMapping`] - Objects that can be evaluated at some argument

mod composition;
mod differentiable;
mod mapping;

pub use composition::ComposedMapping;
pub use differentiable::Differentiable;
pub use mapping::{IndexedMapping, Mapping};
