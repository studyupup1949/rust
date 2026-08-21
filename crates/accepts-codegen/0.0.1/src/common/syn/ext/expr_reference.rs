use syn::{
    Attribute, Expr, ExprReference,
    token::{And, Mut},
};

pub trait ExprReferenceConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        and_token: And,
        mutability: Option<Mut>,
        expr: Box<Expr>,
    ) -> ExprReference;

    fn from_mutability_expr(mutability: Option<Mut>, expr: Box<Expr>) -> ExprReference;

    fn from_expr(expr: Box<Expr>) -> ExprReference;

    fn mut_from_expr(expr: Box<Expr>) -> ExprReference;
}

impl ExprReferenceConstructExt for ExprReference {
    fn from_parts(
        attrs: Vec<Attribute>,
        and_token: And,
        mutability: Option<Mut>,
        expr: Box<Expr>,
    ) -> ExprReference {
        ExprReference {
            attrs,
            and_token,
            mutability,
            expr,
        }
    }

    fn from_mutability_expr(mutability: Option<Mut>, expr: Box<Expr>) -> ExprReference {
        Self::from_parts(Vec::new(), And::default(), mutability, expr)
    }

    fn from_expr(expr: Box<Expr>) -> ExprReference {
        Self::from_mutability_expr(None, expr)
    }

    fn mut_from_expr(expr: Box<Expr>) -> ExprReference {
        Self::from_mutability_expr(Some(Mut::default()), expr)
    }
}
