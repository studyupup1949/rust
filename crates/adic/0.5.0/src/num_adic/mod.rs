//! Adic number objects
//!
//! ### Adic integer structs
//!
//! - [`UAdic`] - Finite set of digits to represent arbitrary precision unsigned integers
//! - [`EAdic`] - Exact p-adic integer, either unsigned int, signed int, or rational
//! - [`ZAdic`] - Any of the above **or** approximately represent any p-adic integer in Z_p
//!
//! ### Adic number structs
//!
//! - [`QAdic`] - Fraction field, holding any of the above
//!   [`AdicIntegers`](crate::traits::AdicInteger) along with a [`Valuation`](crate::normed::Valuation)
//!
//! ### Composite Adic structs
//!
//! - [`PowAdic`] - p^n-adic integer
//! - [`MAdic`] - m-adic integer, a member of a mixed-prime ring with nonzero zero-divisors


mod composite;
mod fraction;
mod int_approx;
mod int_exact;
mod int_raw;
mod lazy_div;
mod maybe;

pub (crate) use int_raw::{IAdic, RAdic};
pub (crate) use int_exact::IntegerVariant;
pub (crate) use lazy_div::LazyDiv;

pub use composite::{MAdic, PowAdic};
pub use fraction::QAdic;
pub use int_approx::ZAdic;
pub use int_exact::EAdic;
pub use int_raw::UAdic;


#[cfg(test)]
mod test_adic;
#[cfg(test)]
pub mod test_util;
