use syn::{
    Attribute, Expr, ExprAwait,
    token::{Await, Dot},
};

pub trait ExprAwaitConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        base: Box<Expr>,
        dot_token: Dot,
        await_token: Await,
    ) -> ExprAwait;

    fn from_base(base: Box<Expr>) -> ExprAwait;
}

impl ExprAwaitConstructExt for ExprAwait {
    fn from_parts(
        attrs: Vec<Attribute>,
        base: Box<Expr>,
        dot_token: Dot,
        await_token: Await,
    ) -> ExprAwait {
        ExprAwait {
            attrs,
            base,
            dot_token,
            await_token,
        }
    }

    fn from_base(base: Box<Expr>) -> ExprAwait {
        Self::from_parts(Vec::new(), base, Dot::default(), Await::default())
    }
}
