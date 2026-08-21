use indexmap::IndexSet;
use proc_macro2::TokenStream;
use std::hash::BuildHasher;
use syn::{
    Path, PreciseCapture, TraitBound, TraitBoundModifier, TypeParamBound, punctuated::Punctuated,
};

use crate::common::collection::MergeMap;

use super::{merge_captured_params, merge_generic_params};

pub fn merge_type_param_bounds<P, S>(
    type_param_bounds: Punctuated<TypeParamBound, P>,
    other_type_param_bounds: impl IntoIterator<Item = Punctuated<TypeParamBound, P>>,
) -> Punctuated<TypeParamBound, P>
where
    P: Default,
    S: BuildHasher + Default,
{
    let mut trait_map =
        MergeMap::<(Path, TraitBoundModifier), TraitBound, _>::with_default(|existing, value| {
            if let Some(mut bound_lifetimes) = existing.lifetimes.take() {
                bound_lifetimes.lifetimes = if let Some(s) = value.lifetimes {
                    merge_generic_params::<_, S>(bound_lifetimes.lifetimes, [s.lifetimes])
                } else {
                    merge_generic_params::<_, S>(bound_lifetimes.lifetimes, [])
                };
                existing.lifetimes = Some(bound_lifetimes);
            } else {
                existing.lifetimes = value.lifetimes;
            }
        });

    let mut lifetime_indexset = IndexSet::new();

    let mut precise_capture_map =
        MergeMap::<(), PreciseCapture, _>::with_default(|existing, value| {
            existing.params =
                merge_captured_params::<_, S>(std::mem::take(&mut existing.params), [value.params])
        });

    let mut verbatim_map = MergeMap::<(), TokenStream, _>::with_default(|existing, value| {
        existing.extend(value.into_iter());
    });

    let mut insert_bound = |type_param_bound: TypeParamBound| match type_param_bound {
        TypeParamBound::Trait(type_bound) => {
            trait_map.upsert(
                (type_bound.path.clone(), type_bound.modifier.clone()),
                type_bound,
            );
        }
        TypeParamBound::Lifetime(lifetime) => {
            lifetime_indexset.insert(lifetime);
        }
        TypeParamBound::PreciseCapture(precise_capture) => {
            precise_capture_map.upsert((), precise_capture);
        }
        TypeParamBound::Verbatim(token_stream) => {
            verbatim_map.upsert((), token_stream);
        }
        _ => {}
    };

    for type_param_bound in type_param_bounds {
        insert_bound(type_param_bound)
    }

    for type_param_bounds in other_type_param_bounds {
        for type_param_bound in type_param_bounds {
            insert_bound(type_param_bound)
        }
    }

    let mut params = Punctuated::new();

    for (_, trait_bound) in trait_map.into_inner().into_iter() {
        params.push(TypeParamBound::Trait(trait_bound));
    }

    for type_param in lifetime_indexset.into_iter() {
        params.push(TypeParamBound::Lifetime(type_param));
    }

    for (_, precise_capture) in precise_capture_map.into_inner().into_iter() {
        params.push(TypeParamBound::PreciseCapture(precise_capture));
    }

    for (_, token_stream) in verbatim_map.into_inner().into_iter() {
        params.push(TypeParamBound::Verbatim(token_stream));
    }

    params
}
