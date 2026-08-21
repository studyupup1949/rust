use syn::{
    Attribute, Block, ImplItemFn, Signature, Visibility,
    token::{self},
};

pub trait ImplItemFnConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        vis: Visibility,
        defaultness: Option<token::Default>,
        sig: Signature,
        block: Block,
    ) -> ImplItemFn;

    fn from_vis_sig_block(vis: Visibility, sig: Signature, block: Block) -> ImplItemFn;
}

impl ImplItemFnConstructExt for ImplItemFn {
    fn from_parts(
        attrs: Vec<Attribute>,
        vis: Visibility,
        defaultness: Option<token::Default>,
        sig: Signature,
        block: Block,
    ) -> ImplItemFn {
        ImplItemFn {
            attrs,
            vis,
            defaultness,
            sig,
            block,
        }
    }

    fn from_vis_sig_block(vis: Visibility, sig: Signature, block: Block) -> ImplItemFn {
        Self::from_parts(Vec::new(), vis, None, sig, block)
    }
}
