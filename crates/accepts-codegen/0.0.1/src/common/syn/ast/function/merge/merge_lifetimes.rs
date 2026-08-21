use std::hash::BuildHasher;
use syn::{Lifetime, punctuated::Punctuated};

pub fn merge_lifetimes<P, S>(
    captured_params: Punctuated<Lifetime, P>,
    other_captured_params: impl IntoIterator<Item = Punctuated<Lifetime, P>>,
) -> Punctuated<Lifetime, P>
where
    P: Default,
    S: BuildHasher + Default,
{
    super::internal::merge_unique_punctuated::<_, _, S>(captured_params, other_captured_params)
}
