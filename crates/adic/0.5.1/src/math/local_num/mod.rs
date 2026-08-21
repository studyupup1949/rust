//! Traits for "local" numbers
//!
//! Sometimes, a given type `T` only has the concept of `zero` and `one` local to `self`.
//! E.g. the 5-adic space ZZ_5 has `zero` in it, but it is not the same `zero` as in ZZ_7.
//! Same with `one`, shared concepts but in a "subspace" of `T`.
//!
//! So a function to create `zero` or `one` needs extra information than the traits
//!  [Zero](<https://docs.rs/num/latest/num/trait.Zero.html>)
//!  and [One](<https://docs.rs/num/latest/num/trait.One.html>)
//!  in [num](<https://docs.rs/num/latest/num/>).
//! The obvious "extra" information is `self`.
//!
//! Built on top of this are the constructed traits of `LocalNum` and `LocalInteger`.
//! These follow similarly to the `num` crate with two major differences:
//! - They only require `LocalZero` and `LocalOne` instead of `Zero` and `One`
//! - They don't rely on information about number ordering, such as floor and ceil
//!
//! Traits:
//!
//! - [`LocalZero`] asks for `zero(Self)` rather than `zero()`
//! - [`LocalOne`] asks for `one(Self)` rather than `one()`
//! - [`LocalNum`] the base type for a ring-like structure, with local zero/one and ring ops
//! - [`LocalInteger`] a `LocalNum` that has an idea of GCD/LCM

mod local_num;
mod trait_impl;

#[cfg(test)]
mod test;

pub use local_num::{LocalInteger, LocalNum, LocalOne, LocalZero};
