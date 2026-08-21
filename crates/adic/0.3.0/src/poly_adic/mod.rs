//! Adic polynomials
//!
//! ### Polynomial structs
//!
//! - [`AdicPolynomial`] - Struct representing an adic polynomial with [`AdicInteger`](crate::AdicInteger) coefficients
//!
//! ### Root/variety structs
//!
//! - [`ZAdicVariety`] - Collection of [`ZAdics`](crate::ZAdic) often representing roots of an `AdicPolynomial`
//!
//! ### Rootfinding methods
//!
//! - [`polynomial_variety`] - Find the roots of an `AdicPolynomial` using the Hensel lemma
//! - [`nth_root`] - Calculate the n-th root of an `AdicInteger` using the Hensel lemma
//! - [`roots_of_unity`] - Calculate the p-1 roots of unity in Z_p (2 roots in Z_2)

mod adic_polynomial;
mod h_lift;
mod variety;

pub use adic_polynomial::AdicPolynomial;
pub use h_lift::{nth_root, polynomial_variety, roots_of_unity};
pub use variety::ZAdicVariety;

#[cfg(test)]
mod test_h_lift;
