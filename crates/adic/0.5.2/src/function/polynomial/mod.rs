//! Polynomial structs
//!
//! [`Polynomial`] - Struct representing a polynomial, enhanced with [`AdicInteger`](crate::AdicInteger) or [`QAdic`](crate::QAdic) coefficients

mod euclid;
mod factorization;
mod newton_polygon;
mod ops;
mod polynomial;
mod polynomial_adic;
mod zadic_wrapper;

pub use polynomial::Polynomial;

pub (crate) use newton_polygon::NewtonPolygon;


#[cfg(test)]
mod test;
#[cfg(test)]
mod test_adic;
