use std::hash::RandomState;

use syn::{
    AngleBracketedGenericArguments, Attribute, Block, Generics, ItemImpl, Path, PathArguments,
    Type, TypePath,
    token::{self, Brace, For, Impl, PathSep},
};

use crate::{
    acceptor::common::function::generate::accepts_path,
    common::{
        context::CodegenContext,
        syn::{
            ast::{
                function::merge_generics, tokens::PathSplitLastArgs,
                util::generic::param_to_argument,
            },
            ext::{
                AngleBracketedGenericArgumentsConstructExt, ItemImplConstructExt,
                TypePathConstructExt,
            },
        },
    },
};

use super::{AcceptsBuilder, AcceptsGenerics, AcceptsInfo, AcceptsT};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PartialAcceptsTraitImpl<A: AcceptsInfo + AcceptsBuilder> {
    pub attrs: Vec<Attribute>,
    pub impl_token: token::Impl,
    pub generics: AcceptsGenerics,
    pub accepts: A,
    pub accepts_t: AcceptsT,
    pub trait_for_token: token::For,
    pub brace_token: token::Brace,
    pub accept_block: Block,
}

#[allow(dead_code)]
impl<A: AcceptsInfo + AcceptsBuilder> PartialAcceptsTraitImpl<A> {
    pub fn from_parts(
        attrs: Vec<Attribute>,
        impl_token: token::Impl,
        generics: AcceptsGenerics,
        accepts: A,
        accepts_t: AcceptsT,
        trait_for_token: token::For,
        brace_token: token::Brace,
        accept_block: Block,
    ) -> Self {
        Self {
            attrs,
            impl_token,
            accepts,
            accepts_t,
            generics,
            trait_for_token,
            brace_token,
            accept_block,
        }
    }

    pub fn from_accepts(
        generics: AcceptsGenerics,
        accepts: A,
        accepts_t: AcceptsT,
        accept_block: Block,
    ) -> Self {
        Self::from_parts(
            Vec::new(),
            Impl::default(),
            generics,
            accepts,
            accepts_t,
            For::default(),
            Brace::default(),
            accept_block,
        )
    }

    pub fn into_item_impl_from_type(
        self,
        ctx: &CodegenContext,
        self_ty: Type,
        self_ty_generics: Generics,
    ) -> ItemImpl {
        let (generics, t_type) = match self.accepts_t.clone() {
            AcceptsT::Type(t_type) => (
                merge_generics::<RandomState>(
                    self_ty_generics,
                    self.generics.into_generics(self.accepts_t),
                ),
                t_type,
            ),
            AcceptsT::Generics(t_type_param) => {
                let t_type_param_ident = t_type_param.ident.clone();
                (
                    merge_generics::<RandomState>(
                        self_ty_generics,
                        self.generics.into_generics(self.accepts_t),
                    ),
                    Type::Path(TypePath::from_path(Path::from(t_type_param_ident))),
                )
            }
        };

        ItemImpl::from_parts(
            self.attrs,
            None,
            None,
            self.impl_token,
            generics,
            Some((
                None,
                accepts_path(ctx, self.accepts.clone(), t_type.clone()),
                self.trait_for_token,
            )),
            Box::new(self_ty),
            self.brace_token,
            vec![
                self.accepts
                    .build_accept_impl_item(ctx, t_type, self.accept_block),
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
