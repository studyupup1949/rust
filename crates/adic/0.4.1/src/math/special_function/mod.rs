//! Special functions for both divisibles and adics

mod binomial;
mod totient;

pub use binomial::adic_binomial;
pub use totient::{carmichael, carmichael_iter, totient};
