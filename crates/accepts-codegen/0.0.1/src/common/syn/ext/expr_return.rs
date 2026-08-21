use syn::{
    Attribute, Block, Expr, ExprReturn, Label,
    token::{Await, Dot, Return},
};

pub trait ExprReturnConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        return_token: Return,
        expr: Option<Box<Expr>>,
    ) -> ExprReturn;

    fn from_attrs_expr(attrs: Vec<Attribute>, expr: Option<Box<Expr>>) -> ExprReturn;

    fn from_expr(expr: Option<Box<Expr>>) -> ExprReturn;

    fn new_none() -> ExprReturn;
}

impl ExprReturnConstructExt for ExprReturn {
    fn from_parts(
        attrs: Vec<Attribute>,
        return_token: Return,
        expr: Option<Box<Expr>>,
    ) -> ExprReturn {
        ExprReturn {
            attrs,
            return_token,
            expr,
        }
    }

    fn from_attrs_expr(attrs: Vec<Attribute>, expr: Option<Box<Expr>>) -> ExprReturn {
        <Self as ExprReturnConstructExt>::from_parts(attrs, Return::default(), expr)
    }

    fn from_expr(expr: Option<Box<Expr>>) -> ExprReturn {
        <Self as ExprReturnConstructExt>::from_attrs_expr(Vec::new(), expr)
    }

    fn new_none() -> ExprReturn {
        <Self as ExprReturnConstructExt>::from_expr(None)
    }
}
