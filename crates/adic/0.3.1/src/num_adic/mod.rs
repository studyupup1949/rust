//! Adic number structs and traits
//!
//! ### Traits
//!
//! - [`AdicInteger`] - Represents an adic number without p-fractional digits
//!
//! ### Adic integer structs
//!
//! - [`UAdic`] - Finite set of digits to represent non-negative integers
//! - [`IAdic`] - Finite digits and trailing either zeros or p-1, to represent any integers
//! - [`RAdic`] - Infinite repeating digits to represent all integers and most rationals
//! - [`ZAdic`] - Infinite digits to represent all p-adic integers Z_p
//!
//! ### Adic number structs
//!
//! - [`QAdic`] - Generic that can hold any of the above `AdicInteger`s and represent and `AdicNumber`
//!
//! ### Other structs
//!
//! - [`AdicSign`] - The real number sign of a rational adic number
//! - [`ZAdicValuation`] - A simple enum to hold non-negative integers or infinity, used for sizing
//! - [`QAdicValuation`] - A simple enum to hold integers or infinity, used for sizing
//! - [`LazyIntDiv`] - An intermediate struct to hold an integer division until precision is selected
//! - [`LazyQDiv`] - An intermediate struct to hold a number division until precision is selected


mod adic_integer;
mod adic_number;
mod adic_sign;
mod lazy_div;
mod valuation;

pub use adic_integer::{
  AdicInteger, SignedAdicInteger,
  IAdic, RAdic, UAdic, ZAdic,
};
pub use adic_sign::AdicSign;
pub use lazy_div::{LazyIntDiv, LazyQDiv};
pub use adic_number::QAdic;
pub use valuation::{QAdicValuation, ZAdicValuation};

#[cfg(test)]
pub mod test_util;
