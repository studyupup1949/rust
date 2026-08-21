//! Abridgment

use std::cmp::max;

/// Abridge
pub fn abridge<const DIGITS: usize>(value: f64, abridge: Abridging, rounding: Rounding) -> f64 {
    let power = 10.0f64.powi(trailing_digits::<DIGITS>(value, abridge) as _);
    round(value * power, rounding) / power
}

fn round(value: f64, rounding: Rounding) -> f64 {
    match rounding {
        // Returns the largest integer less than or equal to value.
        Rounding::Floor => value.floor(),
        // Returns the smallest integer greater than or equal to value.
        Rounding::Ceil => value.ceil(),
        // Returns the nearest integer to value. If a value is half-way between two integers, round away from 0.0.
        Rounding::Round => value.round(),
    }
}

fn trailing_digits<const DIGITS: usize>(value: f64, abridge: Abridging) -> usize {
    let leading_digits = value.abs().log10().ceil();
    match abridge {
        // Treat fractions as negative leading digits so we have the correct total number of significant digits.
        Abridging::Significant => max(0, DIGITS as isize - leading_digits as isize) as _,
        Abridging::Mantissa => DIGITS,
        // Negative logs become 0, never print more than 3 digits after the decimal.
        Abridging::Total => DIGITS.saturating_sub(leading_digits as usize),
    }
}

/// Abridging kind
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Abridging {
    /// Significant digits
    Significant,
    /// Mantissa digits
    Mantissa,
    /// Total digits
    Total,
}

/// Rounding mode
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rounding {
    // Returns the largest integer less than or equal to value.
    Floor,
    // Returns the smallest integer greater than or equal to value.
    Ceil,
    // Returns the nearest integer to value. If a value is half-way between two integers, round away from 0.0.
    Round,
}

pub mod prelude {
    pub use crate::{
        abridge,
        Abridging::{Mantissa, Significant, Total},
        Rounding::{Ceil, Floor, Round},
    };
}
