use std::{rc::Rc, sync::Arc};

use syn::{Block, ImplItem, ImplItemFn, Signature, Type};

use crate::common::{context::CodegenContext, syn::ext::ImplItemFnConstructExt};

pub trait AcceptsBuilder {
    fn build_accept_signature(&self, ctx: &CodegenContext, accepts_t_type: Type) -> Signature;

    fn build_accept_impl_item(
        &self,
        ctx: &CodegenContext,
        accepts_t_type: Type,
        accept_block: Block,
    ) -> ImplItem {
        ImplItem::Fn(ImplItemFn::from_vis_sig_block(
            syn::Visibility::Inherited,
            self.build_accept_signature(ctx, accepts_t_type),
            accept_block,
        ))
    }
}

impl<T: AcceptsBuilder + ?Sized> AcceptsBuilder for &T {
    fn build_accept_signature(&self, ctx: &CodegenContext, accepts_t_type: Type) -> Signature {
        (**self).build_accept_signature(ctx, accepts_t_type)
    }
}

impl<T: AcceptsBuilder + ?Sized> AcceptsBuilder for &mut T {
    fn build_accept_signature(&self, ctx: &CodegenContext, accepts_t_type: Type) -> Signature {
        (**self).build_accept_signature(ctx, accepts_t_type)
    }
}

impl<T: AcceptsBuilder + ?Sized> AcceptsBuilder for Box<T> {
    fn build_accept_signature(&self, ctx: &CodegenContext, accepts_t_type: Type) -> Signature {
        (**self).build_accept_signature(ctx, accepts_t_type)
    }
}

impl<T: AcceptsBuilder + ?Sized> AcceptsBuilder for Rc<T> {
    fn build_accept_signature(&self, ctx: &CodegenContext, accepts_t_type: Type) -> Signature {
        (**self).build_accept_signature(ctx, accepts_t_type)
    }
}

impl<T: AcceptsBuilder + ?Sized> AcceptsBuilder for Arc<T> {
    fn build_accept_signature(&self, ctx: &CodegenContext, accepts_t_type: Type) -> Signature {
        (**self).build_accept_signature(ctx, accepts_t_type)
    }
}
