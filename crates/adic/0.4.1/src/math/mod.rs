mod divisible;
mod sign;
pub mod special_function;

pub (crate) use sign::Sign;

pub use divisible::{Composite, Divisible, Natural, Prime, PrimePower};
