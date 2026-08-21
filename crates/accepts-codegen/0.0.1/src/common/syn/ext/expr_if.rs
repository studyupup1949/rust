use syn::{
    Attribute, Block, Expr, ExprIf,
    token::{Else, If},
};

pub trait ExprIfConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        if_token: If,
        cond: Box<Expr>,
        then_branch: Block,
        else_branch: Option<(Else, Box<Expr>)>,
    ) -> ExprIf;

    fn from_cond_then_else(
        cond: Box<Expr>,
        then_branch: Block,
        else_branch: Option<(Else, Box<Expr>)>,
    ) -> ExprIf;

    fn from_cond_then(cond: Box<Expr>, then_branch: Block) -> ExprIf;
}

impl ExprIfConstructExt for ExprIf {
    fn from_parts(
        attrs: Vec<Attribute>,
        if_token: If,
        cond: Box<Expr>,
        then_branch: Block,
        else_branch: Option<(Else, Box<Expr>)>,
    ) -> ExprIf {
        ExprIf {
            attrs,
            if_token,
            cond,
            then_branch,
            else_branch,
        }
    }

    fn from_cond_then_else(
        cond: Box<Expr>,
        then_branch: Block,
        else_branch: Option<(Else, Box<Expr>)>,
    ) -> ExprIf {
        Self::from_parts(Vec::new(), If::default(), cond, then_branch, else_branch)
    }

    fn from_cond_then(cond: Box<Expr>, then_branch: Block) -> ExprIf {
        Self::from_cond_then_else(cond, then_branch, None)
    }
}
