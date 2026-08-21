mod conversion;
mod e_adic;
mod integer_variant;
mod ops;
mod trait_impl;


pub (crate) use integer_variant::IntegerVariant;
pub use e_adic::EAdic;


#[cfg(test)]
mod test_ops;
