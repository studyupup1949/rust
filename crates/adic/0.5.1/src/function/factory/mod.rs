//! Factory to create [`Mappings`](crate::mapping::Mapping),
//!  [`IndexedMapping`](crate::mapping::IndexedMapping) and related
//!
//! Note: These functions may not work well with adic numbers because of costly division operation.
//! This will be improved in the future with careful, lazy precision analysis.

mod series_factory;

pub use series_factory::{exp, geometric, ln, ln_from_atanh};
