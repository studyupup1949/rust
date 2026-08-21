//! Various functions, implementing [`Mapping`](crate::mapping::Mapping)
//!  and [`IndexedMapping`](crate::mapping::IndexedMapping)
//!
//! - Power series: [`PowerSeries`]
//! - Polynomials: [`Polynomial`], [`Variety`]
//! - General function factory: [`factory`]
//! - Special function factory: [`special`]

mod polynomial;
mod series;
mod variety;

pub mod factory;
pub mod special;

pub use series::PowerSeries;
pub use polynomial::Polynomial;
pub use variety::Variety;

// Future public
#[allow(unused_imports)]
use polynomial::NewtonPolygon;

#[cfg(test)]
mod test;
