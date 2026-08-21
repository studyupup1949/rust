use proc_macro2::TokenStream;
use quote::ToTokens;
use syn::{ItemStruct, punctuated::Punctuated};

use crate::{
    acceptor::common::ast::accepts_trait_ast::{
        AcceptsBuilder, AcceptsInfo, PartialAcceptsTraitImpl,
    },
    common::{
        collection::HybridDeck,
        context::CodegenContext,
        syn::ast::{
            partial_impl::{PartialInherentImpl, PartialTraitImpl},
            tokens::PathSplitLastArgs,
        },
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructAcceptorSpec<
    A: AcceptsInfo + AcceptsBuilder,
    const INHERENT_IMPL_INLINE_CAP: usize,
    const ACCEPTS_TRAIT_INLINE_CAP: usize,
    const TRAIT_IMPL_INLINE_CAP: usize,
> {
    pub struct_: ItemStruct,
    pub inherent_impls: HybridDeck<PartialInherentImpl, INHERENT_IMPL_INLINE_CAP>,
    pub accepts_trait_impls: HybridDeck<PartialAcceptsTraitImpl<A>, ACCEPTS_TRAIT_INLINE_CAP>,
    pub other_trait_impls: HybridDeck<PartialTraitImpl, TRAIT_IMPL_INLINE_CAP>,
}

impl<
    A: AcceptsInfo + AcceptsBuilder,
    const INHERENT_IMPL_INLINE_CAP: usize,
    const ACCEPTS_TRAIT_INLINE_CAP: usize,
    const TRAIT_IMPL_INLINE_CAP: usize,
> StructAcceptorSpec<A, INHERENT_IMPL_INLINE_CAP, ACCEPTS_TRAIT_INLINE_CAP, TRAIT_IMPL_INLINE_CAP>
{
    pub fn from_parts(
        struct_: ItemStruct,
        inherent_impls: HybridDeck<PartialInherentImpl, INHERENT_IMPL_INLINE_CAP>,
        accepts_traits: HybridDeck<PartialAcceptsTraitImpl<A>, ACCEPTS_TRAIT_INLINE_CAP>,
        other_trait_impls: HybridDeck<PartialTraitImpl, TRAIT_IMPL_INLINE_CAP>,
    ) -> Self {
        Self {
            struct_,
            inherent_impls,
            accepts_trait_impls: accepts_traits,
            other_trait_impls,
        }
    }

    pub fn into_tokens(self, ctx: &CodegenContext, tokens: &mut TokenStream) {
        self.struct_.to_tokens(tokens);

        let self_ty_path =
            PathSplitLastArgs::from_parts(None, Punctuated::new(), self.struct_.ident);

        for inherent_impl in self.inherent_impls {
            inherent_impl
                .into_item_impl(self_ty_path.clone(), self.struct_.generics.clone())
                .to_tokens(tokens);
        }

        for accepts_trait_impl in self.accepts_trait_impls.into_iter() {
            accepts_trait_impl
                .into_item_impl_from_path(ctx, self_ty_path.clone(), self.struct_.generics.clone())
                .to_tokens(tokens);
        }

        for other_trait_impl in self.other_trait_impls {
            other_trait_impl
                .into_item_impl(self_ty_path.clone(), self.struct_.generics.clone())
                .to_tokens(tokens);
        }
    }
}
