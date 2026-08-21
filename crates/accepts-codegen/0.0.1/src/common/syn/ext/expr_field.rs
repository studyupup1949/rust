use syn::{Attribute, Expr, ExprField, Member, Token, token::Dot};

pub trait ExprFieldConstructExt {
    fn from_parts(
        attrs: Vec<Attribute>,
        base: Box<Expr>,
        dot_token: Token![.],
        member: Member,
    ) -> ExprField;

    fn from_base_member(base: Box<Expr>, member: Member) -> ExprField;
}

impl ExprFieldConstructExt for ExprField {
    fn from_parts(
        attrs: Vec<Attribute>,
        base: Box<Expr>,
        dot_token: Token![.],
        member: Member,
    ) -> ExprField {
        ExprField {
            attrs,
            base,
            dot_token,
            member,
        }
    }

    fn from_base_member(base: Box<Expr>, member: Member) -> ExprField {
        Self::from_parts(Vec::new(), base, Dot::default(), member)
    }
}
