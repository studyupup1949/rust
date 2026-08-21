//! Adic number structs of various types
//!
//! - [`AdicInteger`] - Trait representing an adic number without p-fractional digits
//!
//! - [`UAdic`] - Finite set of digits to represent non-negative integers
//! - [`RAdic`] - Infinite repeating digits to represent all integers and most rationals
//! - [`ZAdic`] - Infinite digits to represent all p-adic integers Z_p


mod adic_integer;
mod adic_sign;
mod i_adic;
mod r_adic;
mod u_adic;
mod valuation;
mod z_adic;

pub use adic_integer::AdicInteger;
pub use adic_sign::AdicSign;
pub use i_adic::IAdic;
pub use r_adic::RAdic;
pub use u_adic::UAdic;
pub use valuation::ZAdicValuation;
pub use z_adic::ZAdic;
