use std::hash::RandomState;

use syn::{
    AngleBracketedGenericArguments, Attribute, Generics, ImplItem, ItemImpl, Path, PathArguments,
    Token, Type, TypePath,
    token::{self, Brace, For, Impl, PathSep},
};

use crate::common::syn::{
    ast::{function::merge_generics, tokens::PathSplitLastArgs, util::generic::param_to_argument},
    ext::AngleBracketedGenericArgumentsConstructExt,
};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PartialTraitImpl {
    pub attrs: Vec<Attribute>,
    pub defaultness: Option<Token![default]>,
    pub unsafety: Option<Token![unsafe]>,
    pub impl_token: Token![impl],
    pub generics: Generics,
    pub trait_: (Option<Token![!]>, Path, Token![for]),
    pub brace_token: token::Brace,
    pub items: Vec<ImplItem>,
}

#[allow(dead_code)]
impl PartialTraitImpl {
    pub fn from_parts(
        attrs: Vec<Attribute>,
        defaultness: Option<Token![default]>,
        unsafety: Option<Token![unsafe]>,
        impl_token: Token![impl],
        generics: Generics,
        trait_: (Option<Token![!]>, Path, Token![for]),
        brace_token: token::Brace,
        items: Vec<ImplItem>,
    ) -> Self {
        Self {
            attrs,
            defaultness,
            unsafety,
            impl_token,
            generics,
            trait_,
            brace_token,
            items,
        }
    }

    pub fn from_trait_path(trait_path: Path) -> Self {
        Self::from_parts(
            Vec::new(),
            None,
            None,
            Impl::default(),
            Generics::default(),
            (None, trait_path, For::default()),
            Brace::default(),
            Vec::new(),
        )
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
            defaultness: self.defaultness,
            unsafety: self.unsafety,
            impl_token: self.impl_token,
            generics: merge_generics::<RandomState>(self_ty_generics, self.generics),
            trait_: Some(self.trait_),
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

    pub fn decompose_and_restore<Inspector>(
        self,
        self_ty_path: PathSplitLastArgs,
        self_ty_generics: Generics,
        inspector: Inspector,
    ) -> Self
    where
        Inspector: FnOnce(&ItemImpl),
    {
        let generics_backup = self.generics.clone();

        let item_impl = self.into_item_impl(self_ty_path, self_ty_generics);

        (inspector)(&item_impl);

        Self::from_parts(
            item_impl.attrs,
            item_impl.defaultness,
            item_impl.unsafety,
            item_impl.impl_token,
            generics_backup,
            item_impl.trait_.unwrap(),
            item_impl.brace_token,
            item_impl.items,
        )
    }
}
