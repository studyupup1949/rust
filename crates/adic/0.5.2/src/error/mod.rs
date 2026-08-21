//! adic error module

mod adic_error;
mod valid;


pub (crate) use valid::{validate_digits_mod_p, validate_matching_p};

pub use adic_error::{AdicError, AdicResult};
