use syn::{
    Attribute, Expr, ExprCall,
    punctuated::Punctuated,
    token::{Comma, Paren},
};

pub trait ExprCallConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        func: Box<Expr>,
        paren_token: Paren,
        args: Punctuated<Expr, Comma>,
    ) -> ExprCall;

    fn from_func_args(func: Box<Expr>, args: Punctuated<Expr, Comma>) -> ExprCall;

    fn from_func(func: Box<Expr>) -> ExprCall;
}

impl ExprCallConstructExt for ExprCall {
    fn from_parts(
        attrs: Vec<Attribute>,
        func: Box<Expr>,
        paren_token: Paren,
        args: Punctuated<Expr, Comma>,
    ) -> ExprCall {
        ExprCall {
            attrs,
            func,
            paren_token,
            args,
        }
    }

    fn from_func_args(func: Box<Expr>, args: Punctuated<Expr, Comma>) -> ExprCall {
        Self::from_parts(Vec::new(), func, Paren::default(), args)
    }

    fn from_func(func: Box<Expr>) -> ExprCall {
        Self::from_func_args(func, Punctuated::new())
    }
}
