use syn::{
    Arm, Attribute, Expr, ExprMatch,
    token::{Brace, Match},
};

pub trait ExprMatchConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        match_token: Match,
        expr: Box<Expr>,
        brace_token: Brace,
        arms: Vec<Arm>,
    ) -> ExprMatch;

    fn from_attrs_expr_arms(attrs: Vec<Attribute>, expr: Box<Expr>, arms: Vec<Arm>) -> ExprMatch;

    fn from_expr_arms(expr: Box<Expr>, arms: Vec<Arm>) -> ExprMatch;
}

impl ExprMatchConstructExt for ExprMatch {
    fn from_parts(
        attrs: Vec<Attribute>,
        match_token: Match,
        expr: Box<Expr>,
        brace_token: Brace,
        arms: Vec<Arm>,
    ) -> ExprMatch {
        ExprMatch {
            attrs,
            match_token,
            expr,
            brace_token,
            arms,
        }
    }

    fn from_attrs_expr_arms(attrs: Vec<Attribute>, expr: Box<Expr>, arms: Vec<Arm>) -> ExprMatch {
        <Self as ExprMatchConstructExt>::from_parts(
            attrs,
            Match::default(),
            expr,
            Brace::default(),
            arms,
        )
    }

    fn from_expr_arms(expr: Box<Expr>, arms: Vec<Arm>) -> ExprMatch {
        <Self as ExprMatchConstructExt>::from_attrs_expr_arms(Vec::new(), expr, arms)
    }
}
