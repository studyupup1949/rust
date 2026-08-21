//! Variety structs
//!
//! [`Variety`] - Collection of numbers, often representing roots of a `Polynomial`

mod adic_variety;
mod variety;

#[cfg(test)]
mod test;


pub use variety::Variety;
