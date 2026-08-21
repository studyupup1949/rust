use num::{integer::binomial, Integer, Unsigned};
use crate::{AdicNumber, Prime, UAdic};

/// Binomial coefficients in `UAdic` form.
/// Accepts unsigned integers.
///
/// ```
/// # use adic::{special_function::adic_binomial, AdicNumber, UAdic};
/// assert_eq!(UAdic::from_u32(3, 10), adic_binomial(3, 5u32, 2u32));
/// assert_eq!("101._3", adic_binomial(3, 5u32, 2u32).to_string());
/// assert_eq!(UAdic::from_u32(5, 10), adic_binomial(5, 5u32, 2u32));
/// assert_eq!("20._5", adic_binomial(5, 5u32, 2u32).to_string());
/// ```
pub fn adic_binomial<P, I>(p: P, n: I, k: I) -> UAdic
where P: Into<Prime>, I: Clone + Integer + Unsigned, I: Into<u32> {
    let b = binomial(n, k);
    UAdic::from_u32(p.into(), b.into())
}
