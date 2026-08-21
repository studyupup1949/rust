//! Adic number structs and traits
//!
//! ### Traits
//!
//! - [`AdicNumber`] - An adic number
//! - [`SignedAdicNumber`] - An adic number that includes signed integers
//! - [`AdicInteger`] - An adic number without p-fractional digits
//! - [`AdicFraction`] - An adic number with p-fractional digits
//! - [`AdicSized`] - An adic number that's size can be measured
//! - [`AdicApproximate`] - An adic number whose certainty can be measured
//!
//! ### Adic integer structs
//!
//! - [`UAdic`] - Finite set of digits to represent non-negative integers
//! - [`IAdic`] - Finite digits and trailing either zeros or p-1, to represent any integers
//! - [`RAdic`] - Infinite repeating digits to represent all integers and most rationals
//! - [`ZAdic`] - Any of the above OR approximately represent all p-adic integers Z_p
//!
//! ### Adic number structs
//!
//! - [`QAdic`] - Generic that can hold any of the above `AdicInteger`s and represent and `AdicNumber`
//!
//! ### Composite Adic structs
//!
//! - [`AdicPower`] - Generic that representing an `AdicInteger` for a power of a prime
//! - [`AdicComposite`] - An non-primitive adic ring element with a composite base
//!
//! ### Other structs
//!
//! - [`Prime`] - Simple newtype to validate and hold a prime number
//! - [`Sign`] - The real number sign of a rational adic number
//! - [`AdicValuation`] - Adic valuation, used for sizing
//! - [`LazyDiv`] - An intermediate struct to hold a number division until precision is selected


mod adic_approximate;
mod adic_composite;
mod adic_fraction;
mod adic_integer;
mod lazy_div;
mod traits;
mod valuation;

pub (crate) use adic_integer::ExactIntegerVariant;

pub use adic_approximate::ZAdic;
pub use adic_composite::{AdicComposite, AdicPower};
pub use adic_fraction::QAdic;
pub use adic_integer::{IAdic, RAdic, UAdic};
pub use lazy_div::LazyDiv;
pub use traits::{
    AdicApproximate, AdicFraction, AdicInteger, AdicNumber, AdicSized,
    HasDigits, RationalAdicNumber, SignedAdicNumber,
};
pub use valuation::{AdicValuation, AdicValuationRing, QAdicValuation, ZAdicValuation};

#[cfg(test)]
pub mod test_util;
