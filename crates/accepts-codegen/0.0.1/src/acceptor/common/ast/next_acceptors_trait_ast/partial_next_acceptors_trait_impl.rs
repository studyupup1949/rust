use std::hash::RandomState;

use syn::{
    AngleBracketedGenericArguments, Attribute, Block, Generics, ImplItem, ImplItemFn, ItemImpl,
    Lifetime, PathArguments, Type, TypePath,
    token::{self, Brace, For, Impl, PathSep},
};

use crate::{
    acceptor::common::function::generate::{next_acceptors_path, next_acceptors_path_mut},
    common::{
        context::CodegenContext,
        syn::{
            ast::{
                function::merge_generics,
                tokens::PathSplitLastArgs,
                util::generic::{generics_without_defaults, param_to_argument},
            },
            ext::{
                AngleBracketedGenericArgumentsConstructExt, ImplItemFnConstructExt,
                ItemImplConstructExt, TypePathConstructExt,
            },
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialNextAcceptorsTraitImpl {
    pub attrs: Vec<Attribute>,
    pub impl_token: token::Impl,
    pub generics: Generics,
    pub acceptor_type: Type,
    pub iter_lifetime: Lifetime,
    pub iter_type: Type,
    pub mut_: bool,
    pub trait_for_token: token::For,
    pub brace_token: token::Brace,
    pub next_acceptors_block: Block,
}

#[allow(dead_code)]
impl PartialNextAcceptorsTraitImpl {
    pub fn from_parts(
        attrs: Vec<Attribute>,
        impl_token: token::Impl,
        generics: Generics,
        acceptor_type: Type,
        iter_lifetime: Lifetime,
        iter_type: Type,
        mut_: bool,
        trait_for_token: token::For,
        brace_token: token::Brace,
        next_acceptors_block: Block,
    ) -> Self {
        Self {
            attrs,
            impl_token,
            generics,
            acceptor_type,
            iter_lifetime,
            iter_type,
            mut_,
            trait_for_token,
            brace_token,
            next_acceptors_block,
        }
    }

    pub fn from_types(
        acceptor_type: Type,
        iter_lifetime: Lifetime,
        iter_type: Type,
        mut_: bool,
        next_acceptors_block: Block,
    ) -> Self {
        Self::from_parts(
            Vec::new(),
            Impl::default(),
            Generics::default(),
            acceptor_type,
            iter_lifetime,
            iter_type,
            mut_,
            For::default(),
            Brace::default(),
            next_acceptors_block,
        )
    }

    pub fn into_item_impl_from_type(
        self,
        ctx: &CodegenContext,
        self_ty: Type,
        self_ty_generics: Generics,
    ) -> ItemImpl {
        let acceptor_type = self.acceptor_type;
        let iter_lifetime = self.iter_lifetime;
        let iter_type = self.iter_type;
        let mut_ = self.mut_;
        ItemImpl::from_parts(
            self.attrs,
            None,
            None,
            self.impl_token,
            merge_generics::<RandomState>(
                generics_without_defaults(self_ty_generics),
                self.generics,
            ),
            Some((
                None,
                if mut_ {
                    next_acceptors_path_mut(ctx, None, None)
                } else {
                    next_acceptors_path(ctx, None, None)
                },
                self.trait_for_token,
            )),
            Box::new(self_ty),
            self.brace_token,
            vec![
                syn::parse_quote!(type Acceptor<#iter_lifetime> = #acceptor_type where Self: 'a;),
                syn::parse_quote!(type Iter<#iter_lifetime> = #iter_type where Self: 'a;),
                ImplItem::Fn(ImplItemFn::from_vis_sig_block(
                    syn::Visibility::Inherited,
                    if mut_ {
                        syn::parse_quote!(fn next_acceptors_mut(&mut self) -> Self::Iter<'_>)
                    } else {
                        syn::parse_quote!(fn next_acceptors(&self) -> Self::Iter<'_>)
                    },
                    self.next_acceptors_block,
                )),
            ],
        )
    }

    pub fn into_item_impl_from_path(
        self,
        ctx: &CodegenContext,
        self_ty_path: PathSplitLastArgs,
        self_ty_generics: Generics,
    ) -> ItemImpl {
        let self_ty_generics_params = self_ty_generics.params.clone();
        let self_ty_generics_lt_token = self_ty_generics.lt_token.clone();
        let self_ty_generics_gt_token = self_ty_generics.gt_token.clone();

        self.into_item_impl_from_type(
            ctx,
            Type::Path(TypePath::from_path(self_ty_path.into_path(
                if self_ty_generics_params.is_empty() {
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
                },
            ))),
            self_ty_generics,
        )
    }

    pub fn to_item_impl_from_path(
        &self,
        ctx: &CodegenContext,
        self_ty_path: PathSplitLastArgs,
        self_ty_generics: Generics,
    ) -> ItemImpl {
        self.clone()
            .into_item_impl_from_path(ctx, self_ty_path, self_ty_generics)
    }

    pub fn to_item_impl_from_type(
        &self,
        ctx: &CodegenContext,
        self_ty: Type,
        self_ty_generics: Generics,
    ) -> ItemImpl {
        self.clone()
            .into_item_impl_from_type(ctx, self_ty, self_ty_generics)
    }
}
