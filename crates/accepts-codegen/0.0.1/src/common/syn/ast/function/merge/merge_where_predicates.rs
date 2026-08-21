use std::hash::BuildHasher;
use syn::{Ident, PredicateLifetime, PredicateType, Type, WherePredicate, punctuated::Punctuated};

use crate::common::collection::MergeMap;

use super::{merge_generic_params, merge_lifetimes, merge_type_param_bounds};

pub fn merge_where_predicates<P, S>(
    where_predicates: Punctuated<WherePredicate, P>,
    other_where_predicates: impl IntoIterator<Item = Punctuated<WherePredicate, P>>,
) -> Punctuated<WherePredicate, P>
where
    P: Default,
    S: BuildHasher + Default,
{
    let mut lifetime_map = MergeMap::<Ident, PredicateLifetime, _>::with_default(
        |existing: &mut PredicateLifetime, value| {
            existing.bounds =
                merge_lifetimes::<_, S>(std::mem::take(&mut existing.bounds), [value.bounds])
        },
    );

    let mut type_map = MergeMap::<Type, PredicateType, _>::with_default(|existing, value| {
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

        existing.bounds =
            merge_type_param_bounds::<_, S>(std::mem::take(&mut existing.bounds), [value.bounds])
    });

    for where_predicate in where_predicates {
        match where_predicate {
            WherePredicate::Lifetime(lifetime_param) => {
                lifetime_map.upsert(lifetime_param.lifetime.ident.clone(), lifetime_param);
            }
            WherePredicate::Type(type_param) => {
                type_map.upsert(type_param.bounded_ty.clone(), type_param);
            }
            _ => {}
        }
    }

    for where_predicates in other_where_predicates {
        for where_predicate in where_predicates {
            match where_predicate {
                WherePredicate::Lifetime(lifetime_param) => {
                    lifetime_map.upsert(lifetime_param.lifetime.ident.clone(), lifetime_param);
                }
                WherePredicate::Type(type_param) => {
                    type_map.upsert(type_param.bounded_ty.clone(), type_param);
                }
                _ => {}
            }
        }
    }

    let mut new_where_predicates = Punctuated::new();

    for (_, lifetime_param) in lifetime_map.into_inner() {
        new_where_predicates.push(WherePredicate::Lifetime(lifetime_param));
    }
    for (_, type_param) in type_map.into_inner() {
        new_where_predicates.push(WherePredicate::Type(type_param));
    }

    new_where_predicates
}
