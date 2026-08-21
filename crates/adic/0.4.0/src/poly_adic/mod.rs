//! Adic polynomials
//!
//! ### Polynomial structs
//!
//! - [`AdicPolynomial`] - Struct representing an adic polynomial with [`AdicInteger`](crate::AdicInteger) coefficients
//!
//! ### Root/variety structs
//!
//! - [`AdicVariety`] - Collection of approximate [`AdicNumbers`](crate::AdicNumber) often representing roots of an `AdicPolynomial`
//!
//! ### Rootfinding methods
//!
//! - [`polynomial_variety`] - Find the roots of an `AdicPolynomial` using the Hensel lemma
//! - [`nth_root`] - Calculate the n-th root of an `AdicInteger` using the Hensel lemma
//! - [`roots_of_unity`] - Calculate the p-1 roots of unity in Z_p (2 roots in Z_2)

mod adic_polynomial;
mod hensel;
mod teichmuller;
mod variety;

pub use adic_polynomial::AdicPolynomial;
pub use hensel::{nth_root, polynomial_variety, num_nth_roots, variety_size};
pub use teichmuller::{roots_of_unity, teichmuller};
pub use variety::{AdicVariety, ZAdicVariety};

#[cfg(test)]
mod test_hensel;
