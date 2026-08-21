//! Adic traits
//!
//! - [`AdicNumber`] - An adic number
//! - [`SignedAdicNumber`] - An adic number that includes signed integers
//! - [`AdicInteger`] - An adic number without p-fractional digits
//! - [`AdicFraction`] - An adic number with p-fractional digits
//! - [`AdicSized`] - An adic number that's size can be measured
//! - [`AdicApproximate`] - An adic number whose certainty can be measured

mod adic_approximate;
mod adic_fraction;
mod adic_integer;
mod adic_number;
mod adic_sized;
mod has_digits;

pub use adic_approximate::AdicApproximate;
pub use adic_fraction::AdicFraction;
pub use adic_integer::AdicInteger;
pub use adic_number::{AdicNumber, SignedAdicNumber, RationalAdicNumber};
pub use adic_sized::AdicSized;
pub use has_digits::HasDigits;
