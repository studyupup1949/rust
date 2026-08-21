use syn::{
    Arm, AttrStyle, Attribute, Expr, Meta, Pat, Token,
    token::{Bracket, Comma, FatArrow, Pound},
};

pub trait ArmConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        pat: Pat,
        guard: Option<(Token![if], Box<Expr>)>,
        fat_arrow_token: Token![=>],
        body: Box<Expr>,
        comma: Option<Token![,]>,
    ) -> Arm;

    fn from_attrs_pat_guard_body_comma(
        attrs: Vec<Attribute>,
        pat: Pat,
        guard: Option<(Token![if], Box<Expr>)>,
        body: Box<Expr>,
        comma: Option<Token![,]>,
    ) -> Arm;

    fn from_attrs_pat_guard_body(
        attrs: Vec<Attribute>,
        pat: Pat,
        guard: Option<(Token![if], Box<Expr>)>,
        body: Box<Expr>,
    ) -> Arm;

    fn from_pat_guard_body_comma(
        pat: Pat,
        guard: Option<(Token![if], Box<Expr>)>,
        body: Box<Expr>,
        comma: Option<Token![,]>,
    ) -> Arm;

    fn from_pat_guard_body(
        pat: Pat,
        guard: Option<(Token![if], Box<Expr>)>,
        body: Box<Expr>,
    ) -> Arm;
}

impl ArmConstructExt for Arm {
    fn from_parts(
        attrs: Vec<Attribute>,
        pat: Pat,
        guard: Option<(Token![if], Box<Expr>)>,
        fat_arrow_token: Token![=>],
        body: Box<Expr>,
        comma: Option<Token![,]>,
    ) -> Arm {
        Arm {
            attrs,
            pat,
            guard,
            fat_arrow_token,
            body,
            comma,
        }
    }

    fn from_attrs_pat_guard_body_comma(
        attrs: Vec<Attribute>,
        pat: Pat,
        guard: Option<(Token![if], Box<Expr>)>,
        body: Box<Expr>,
        comma: Option<Token![,]>,
    ) -> Arm {
        <Self as ArmConstructExt>::from_parts(attrs, pat, guard, FatArrow::default(), body, comma)
    }

    fn from_attrs_pat_guard_body(
        attrs: Vec<Attribute>,
        pat: Pat,
        guard: Option<(Token![if], Box<Expr>)>,
        body: Box<Expr>,
    ) -> Arm {
        <Self as ArmConstructExt>::from_attrs_pat_guard_body_comma(
            attrs,
            pat,
            guard,
            body,
            Some(Comma::default()),
        )
    }

    fn from_pat_guard_body_comma(
        pat: Pat,
        guard: Option<(Token![if], Box<Expr>)>,
        body: Box<Expr>,
        comma: Option<Token![,]>,
    ) -> Arm {
        <Self as ArmConstructExt>::from_attrs_pat_guard_body_comma(
            Vec::new(),
            pat,
            guard,
            body,
            comma,
        )
    }

    fn from_pat_guard_body(
        pat: Pat,
        guard: Option<(Token![if], Box<Expr>)>,
        body: Box<Expr>,
    ) -> Arm {
        <Self as ArmConstructExt>::from_pat_guard_body_comma(
            pat,
            guard,
            body,
            Some(Comma::default()),
        )
    }
}
