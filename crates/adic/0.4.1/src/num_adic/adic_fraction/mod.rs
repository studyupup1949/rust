//! Adic fraction (non-integer) structs
//!
//! - [`QAdic`] - Generic that can hold any of the above `AdicInteger`s and represent and `AdicNumber`

mod q_adic;
mod q_adic_ops;
mod trait_impl;

pub use q_adic::QAdic;

#[cfg(test)]
mod test_ops;
