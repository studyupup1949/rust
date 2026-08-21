//! Adic traits
//!
//! ### Adic number traits
//! - [`AdicInteger`]
//! - [`AdicPrimitive`]
//! ### Digital number traits
//! - [`HasDigits`]
//! - [`HasApproximateDigits`]
//! - [`CanTruncate`]
//! - [`CanApproximate`]
//! ### Regular number + prime -> adic
//! - [`PrimedFrom`]
//! - [`PrimedInto`]
//! - [`TryPrimedFrom`]
//! - [`TryPrimedInto`]

mod adic_integer;
mod adic_primitive;
mod digit_display;
mod has_digits;
mod primed_from;
mod truncation;

pub (crate) use digit_display::HasDigitDisplay;

pub use adic_integer::AdicInteger;
pub use adic_primitive::AdicPrimitive;
pub use has_digits::{HasApproximateDigits, HasDigits};
pub use primed_from::{PrimedFrom, PrimedInto, TryPrimedFrom, TryPrimedInto};
pub use truncation::{CanApproximate, CanTruncate};
