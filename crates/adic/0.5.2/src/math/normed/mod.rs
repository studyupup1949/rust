//! Structures with archimedian and ultrametric norms
//!
//! - [`Normed`] - A number with a norm/size and unit component
//! - [`UltraNormed`] - A `Normed` number with an ultrametric norm and valuation
//! - [`Valuation`] - Ultrametric valuation, a sort of discrete exponential size
//! - [`ValuationRing`] - Ring used for finite ultrametric valuations

mod normed;
mod trait_impl;
mod ultra_normed;
mod valuation;
mod valuation_ring;

pub use normed::Normed;
pub use ultra_normed::UltraNormed;
pub use valuation::Valuation;
pub use valuation_ring::ValuationRing;
