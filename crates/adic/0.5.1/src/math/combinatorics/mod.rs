//! Combinatoric functions

#![allow(dead_code, unused_imports)]

mod factorial;
mod totient;

pub use factorial::factorial;
pub use totient::{carmichael, prime_power_factors, totient};
