use std::hash::RandomState;

use syn::{
    AngleBracketedGenericArguments, Attribute, Generics, ImplItem, ItemImpl, PathArguments, Token,
    Type, TypePath,
    token::{Brace, PathSep},
};

use crate::common::syn::{
    ast::{function::merge_generics, tokens::PathSplitLastArgs, util::generic::param_to_argument},
    ext::AngleBracketedGenericArgumentsConstructExt,
};

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct PartialInherentImpl {
    pub attrs: Vec<Attribute>,
    pub impl_token: Token![impl],
    pub generics: Generics,
    pub brace_token: Brace,
    pub items: Vec<ImplItem>,
}

#[allow(dead_code)]
impl PartialInherentImpl {
    pub fn from_parts(
        attrs: Vec<Attribute>,
        impl_token: Token![impl],
        generics: Generics,
        brace_token: Brace,
        items: Vec<ImplItem>,
    ) -> Self {
        Self {
            attrs,
            impl_token,
            generics,
            brace_token,
            items,
        }
    }

    pub fn into_item_impl(
        self,
        self_ty_path: PathSplitLastArgs,
        self_ty_generics: Generics,
    ) -> ItemImpl {
        let self_ty_generics_params = self_ty_generics.params.clone();
        let self_ty_generics_lt_token = self_ty_generics.lt_token.clone();
        let self_ty_generics_gt_token = self_ty_generics.gt_token.clone();

        ItemImpl {
            attrs: self.attrs,
            defaultness: None,
            unsafety: None,
            impl_token: self.impl_token,
            generics: merge_generics::<RandomState>(self_ty_generics, self.generics),
            trait_: None,
            self_ty: Box::new(Type::Path(TypePath {
                qself: None,
                path: {
                    self_ty_path.into_path(if self_ty_generics_params.is_empty() {
                        PathArguments::None
                    } else {
                        PathArguments::AngleBracketed(AngleBracketedGenericArguments::from_parts(
                            Some(PathSep::default()),
                            self_ty_generics_lt_token.unwrap_or_default(),
                            self_ty_generics_params
                                .into_iter()
                                .map(param_to_argument)
                                .collect(),
                            self_ty_generics_gt_token.unwrap_or_default(),
                        ))
                    })
                },
            })),
            brace_token: self.brace_token,
            items: self.items,
        }
    }

    pub fn decompose_and_restore_mut<Mutator>(
        self,
        self_ty_path: PathSplitLastArgs,
        self_ty_generics: Generics,
        mutator: Mutator,
    ) -> Self
    where
        Mutator: FnOnce(&mut ItemImpl),
    {
        let generics_backup = self.generics.clone();

        let mut item_impl = self.into_item_impl(self_ty_path, self_ty_generics);

        (mutator)(&mut item_impl);

        Self::from_parts(
            item_impl.attrs,
            item_impl.impl_token,
            generics_backup,
            item_impl.brace_token,
            item_impl.items,
        )
    }

    pub fn decompose_and_restore<Inspector>(
        self,
        self_ty_path: PathSplitLastArgs,
        self_ty_generics: Generics,
        inspector: Inspector,
    ) -> Self
    where
        Inspector: FnOnce(&ItemImpl),
    {
        self.decompose_and_restore_mut(self_ty_path, self_ty_generics, |item_impl_mut| {
            inspector(item_impl_mut)
        })
    }
}
