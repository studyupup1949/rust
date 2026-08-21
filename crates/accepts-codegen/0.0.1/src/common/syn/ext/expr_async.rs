use syn::{
    Attribute, Block, ExprAsync,
    token::{Async, Move},
};

pub trait ExprAsyncConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        async_token: Async,
        capture: Option<Move>,
        block: Block,
    ) -> ExprAsync;

    fn from_capture_block(capture: Option<Move>, block: Block) -> ExprAsync;

    fn from_block(block: Block) -> ExprAsync;

    fn async_move_from_block(block: Block) -> ExprAsync;
}

impl ExprAsyncConstructExt for ExprAsync {
    fn from_parts(
        attrs: Vec<Attribute>,
        async_token: Async,
        capture: Option<Move>,
        block: Block,
    ) -> ExprAsync {
        ExprAsync {
            attrs,
            async_token,
            capture,
            block,
        }
    }

    fn from_capture_block(capture: Option<Move>, block: Block) -> ExprAsync {
        Self::from_parts(Vec::new(), Async::default(), capture, block)
    }

    fn from_block(block: Block) -> ExprAsync {
        Self::from_capture_block(None, block)
    }

    fn async_move_from_block(block: Block) -> ExprAsync {
        Self::from_capture_block(Some(Move::default()), block)
    }
}
