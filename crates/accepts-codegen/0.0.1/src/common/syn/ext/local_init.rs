use syn::{
    Expr, LocalInit,
    token::{Else, Eq},
};

pub trait LocalInitConstructExt {
    fn from_parts(eq_token: Eq, expr: Box<Expr>, diverge: Option<(Else, Box<Expr>)>) -> LocalInit;

    fn from_expr(expr: Box<Expr>) -> LocalInit;

    fn from_expr_diverge(expr: Box<Expr>, diverge: Option<(Else, Box<Expr>)>) -> LocalInit;
}

impl LocalInitConstructExt for LocalInit {
    fn from_parts(eq_token: Eq, expr: Box<Expr>, diverge: Option<(Else, Box<Expr>)>) -> LocalInit {
        LocalInit {
            eq_token,
            expr,
            diverge,
        }
    }

    fn from_expr(expr: Box<Expr>) -> LocalInit {
        Self::from_parts(Eq::default(), expr, None)
    }

    fn from_expr_diverge(expr: Box<Expr>, diverge: Option<(Else, Box<Expr>)>) -> LocalInit {
        Self::from_parts(Eq::default(), expr, diverge)
    }
}
