use std::hash::BuildHasher;
use syn::{
    ConstParam, GenericParam, Ident, LifetimeParam, TypeParam, punctuated::Punctuated, token::Colon,
};

use crate::common::collection::MergeMap;

use super::{internal::extend_unique_by_key, merge_type_param_bounds};

pub fn merge_generic_params<P, S>(
    generic_params: Punctuated<GenericParam, P>,
    other_generic_params: impl IntoIterator<Item = Punctuated<GenericParam, P>>,
) -> Punctuated<GenericParam, P>
where
    P: Default,
    S: BuildHasher + Default,
{
    let mut lifetime_map = MergeMap::<Ident, LifetimeParam, _>::with_default(|existing, value| {
        existing.attrs.extend(value.attrs);
        extend_unique_by_key(&mut existing.bounds, value.bounds, |lifetime| {
            lifetime.ident.clone()
        });
    });

    let mut type_map = MergeMap::<Ident, TypeParam, _>::with_default(|existing, value| {
        existing.attrs.extend(value.attrs);

        existing.bounds =
            merge_type_param_bounds::<_, S>(std::mem::take(&mut existing.bounds), [value.bounds]);

        if existing.eq_token.is_none() {
            existing.eq_token = value.eq_token;
        }

        if existing.default.is_none() {
            existing.default = value.default;
        }
    });

    let mut const_map = MergeMap::<Ident, ConstParam, _>::with_default(|existing, value| {
        existing.attrs.extend(value.attrs);

        if existing.eq_token.is_none() {
            existing.eq_token = value.eq_token;
        }

        if existing.default.is_none() {
            existing.default = value.default;
        }
    });

    for param in generic_params {
        match param {
            GenericParam::Lifetime(lifetime_param) => {
                lifetime_map.upsert(lifetime_param.lifetime.ident.clone(), lifetime_param);
            }
            GenericParam::Type(type_param) => {
                type_map.upsert(type_param.ident.clone(), type_param);
            }
            GenericParam::Const(const_param) => {
                const_map.upsert(const_param.ident.clone(), const_param);
            }
        }
    }

    for params in other_generic_params {
        for param in params {
            match param {
                GenericParam::Lifetime(lifetime_param) => {
                    lifetime_map.upsert(lifetime_param.lifetime.ident.clone(), lifetime_param);
                }
                GenericParam::Type(type_param) => {
                    type_map.upsert(type_param.ident.clone(), type_param);
                }
                GenericParam::Const(const_param) => {
                    const_map.upsert(const_param.ident.clone(), const_param);
                }
            }
        }
    }

    let mut new_params = Punctuated::new();

    for (_, lifetime_param) in lifetime_map.into_inner() {
        new_params.push(GenericParam::Lifetime(lifetime_param));
    }
    for (_, mut type_param) in type_map.into_inner() {
        if type_param.bounds.is_empty() {
            type_param.colon_token = None;
        } else {
            type_param.colon_token = type_param.colon_token.or(Some(Colon::default()));
        }
        new_params.push(GenericParam::Type(type_param));
    }
    for (_, const_param) in const_map.into_inner() {
        new_params.push(GenericParam::Const(const_param));
    }

    new_params
}
