use syn::{
    AngleBracketedGenericArguments, Attribute, Expr, ExprMethodCall, Ident, Token,
    punctuated::Punctuated,
    token::{Dot, Paren},
};

pub trait ExprMethodCallConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        receiver: Box<Expr>,
        dot_token: Token![.],
        method: Ident,
        turbofish: Option<AngleBracketedGenericArguments>,
        paren_token: Paren,
        args: Punctuated<Expr, Token![,]>,
    ) -> ExprMethodCall;

    fn from_receiver_method_turbofish_args(
        receiver: Box<Expr>,
        method: Ident,
        turbofish: Option<AngleBracketedGenericArguments>,
        args: Punctuated<Expr, Token![,]>,
    ) -> ExprMethodCall;

    fn from_receiver_method_args(
        receiver: Box<Expr>,
        method: Ident,
        args: Punctuated<Expr, Token![,]>,
    ) -> ExprMethodCall;
}

impl ExprMethodCallConstructExt for ExprMethodCall {
    fn from_parts(
        attrs: Vec<Attribute>,
        receiver: Box<Expr>,
        dot_token: Token![.],
        method: Ident,
        turbofish: Option<AngleBracketedGenericArguments>,
        paren_token: Paren,
        args: Punctuated<Expr, Token![,]>,
    ) -> ExprMethodCall {
        ExprMethodCall {
            attrs,
            receiver,
            dot_token,
            method,
            turbofish,
            paren_token,
            args,
        }
    }

    fn from_receiver_method_turbofish_args(
        receiver: Box<Expr>,
        method: Ident,
        turbofish: Option<AngleBracketedGenericArguments>,
        args: Punctuated<Expr, Token![,]>,
    ) -> ExprMethodCall {
        Self::from_parts(
            Vec::new(),
            receiver,
            Dot::default(),
            method,
            turbofish,
            Paren::default(),
            args,
        )
    }

    fn from_receiver_method_args(
        receiver: Box<Expr>,
        method: Ident,
        args: Punctuated<Expr, Token![,]>,
    ) -> ExprMethodCall {
        Self::from_receiver_method_turbofish_args(receiver, method, None, args)
    }
}
