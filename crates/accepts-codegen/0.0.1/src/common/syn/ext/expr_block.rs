use syn::{
    Attribute, Block, Expr, ExprBlock, Label,
    token::{Await, Dot},
};

pub trait ExprBlockConstructExt {
    fn from_parts(attrs: Vec<Attribute>, label: Option<Label>, block: Block) -> ExprBlock;

    fn from_label_block(label: Option<Label>, block: Block) -> ExprBlock;

    fn from_block(block: Block) -> ExprBlock;
}

impl ExprBlockConstructExt for ExprBlock {
    fn from_parts(attrs: Vec<Attribute>, label: Option<Label>, block: Block) -> ExprBlock {
        ExprBlock {
            attrs,
            label,
            block,
        }
    }

    fn from_label_block(label: Option<Label>, block: Block) -> ExprBlock {
        <Self as ExprBlockConstructExt>::from_parts(Vec::new(), label, block)
    }

    fn from_block(block: Block) -> ExprBlock {
        <Self as ExprBlockConstructExt>::from_label_block(None, block)
    }
}
