//! Sequences, with emphasis on [`UltraNormed`](crate::math::normed::UltraNormed) sequences

mod bounds;
mod core;
pub mod factory;

pub use core::{derived, Sequence};
