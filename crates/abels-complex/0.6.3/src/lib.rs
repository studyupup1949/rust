//! Complex numbers with rectangular and polar representations.

#![cfg_attr(not(feature = "std"), no_std)]
mod complex;
// pub mod dual;

pub mod traits;

pub use complex::*;
pub use traits::Number;
