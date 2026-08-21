use syn::{
    Attribute, Expr, ExprLet, Pat,
    token::{Eq, Let},
};

pub trait ExprLetConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        let_token: Let,
        pat: Box<Pat>,
        eq_token: Eq,
        expr: Box<Expr>,
    ) -> ExprLet;

    fn from_pat_expr(pat: Box<Pat>, expr: Box<Expr>) -> ExprLet;
}

impl ExprLetConstructExt for ExprLet {
    fn from_parts(
        attrs: Vec<Attribute>,
        let_token: Let,
        pat: Box<Pat>,
        eq_token: Eq,
        expr: Box<Expr>,
    ) -> ExprLet {
        ExprLet {
            attrs,
            let_token,
            pat,
            eq_token,
            expr,
        }
    }

    fn from_pat_expr(pat: Box<Pat>, expr: Box<Expr>) -> ExprLet {
        Self::from_parts(Vec::new(), Let::default(), pat, Eq::default(), expr)
    }
}
