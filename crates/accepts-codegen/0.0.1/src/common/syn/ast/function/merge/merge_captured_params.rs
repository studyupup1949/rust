use std::hash::BuildHasher;
use syn::{CapturedParam, punctuated::Punctuated};

pub fn merge_captured_params<P, S>(
    captured_params: Punctuated<CapturedParam, P>,
    other_captured_params: impl IntoIterator<Item = Punctuated<CapturedParam, P>>,
) -> Punctuated<CapturedParam, P>
where
    P: Default,
    S: BuildHasher + Default,
{
    super::internal::merge_unique_punctuated::<_, _, S>(captured_params, other_captured_params)
}
