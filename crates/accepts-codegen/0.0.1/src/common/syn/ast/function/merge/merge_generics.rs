use std::hash::BuildHasher;
use syn::Generics;

use crate::common::syn::ext::GenericsConstructExt;

use super::{merge_generic_params, merge_where_predicates};

pub fn merge_generics<S>(mut base: Generics, patch: Generics) -> Generics
where
    S: BuildHasher + Default,
{
    let params = {
        base.params = merge_generic_params::<_, S>(base.params, [patch.params]);
        base.params
    };

    let lt_token = {
        if params.is_empty() {
            None
        } else {
            Some(base.lt_token.or(patch.lt_token).unwrap_or_default())
        }
    };

    let gt_token = {
        if params.is_empty() {
            None
        } else {
            Some(base.gt_token.or(patch.gt_token).unwrap_or_default())
        }
    };

    let where_clause = {
        match (base.where_clause, patch.where_clause) {
            (Some(mut base_wc), Some(patch_wc)) => {
                base_wc.predicates =
                    merge_where_predicates::<_, S>(base_wc.predicates, [patch_wc.predicates]);

                Some(base_wc)
            }
            (Some(mut wc), None) | (None, Some(mut wc)) => {
                wc.predicates = merge_where_predicates::<_, S>(wc.predicates, []);
                Some(wc)
            }
            (None, None) => None,
        }
    };

    Generics::from_parts(lt_token, params, gt_token, where_clause)
}
