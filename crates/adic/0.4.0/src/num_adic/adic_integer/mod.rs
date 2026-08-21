//! Adic integer structs
//!
//! - [`UAdic`] - Finite set of digits to represent non-negative integers
//! - [`IAdic`] - Finite digits and trailing either zeros or p-1, to represent any integers
//! - [`RAdic`] - Infinite repeating digits to represent all integers and most rationals
//! - [`ZAdic`] - Infinite digits to represent all p-adic integers Z_p

mod conversion;
mod i_adic;
mod r_adic;
mod ops;
mod trait_impl;
mod u_adic;
mod z_adic;


pub use i_adic::IAdic;
pub use r_adic::RAdic;
pub use u_adic::UAdic;
pub use z_adic::ZAdic;

use z_adic::ZAdicVariant;


#[cfg(test)]
mod test_adic;
#[cfg(test)]
mod test_ops;
